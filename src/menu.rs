use std::mem::forget;
use std::ops::{Deref, DerefMut};
use std::ptr::{null, null_mut};
use winapi::shared::windef::{HMENU, HWND};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winuser::*;

use Error;
use wide::ToWide;


pub enum MenuItem<'a> {
    String(&'a str),
    Separator,
}
pub enum MenuStatus {
    Enabled,
    Disabled,
    Grayed,
}
pub enum MenuCheck {
    Checked,
    Unchecked,
}
pub enum MenuAction {
    Id(u16),
    ChildMenu(PopupMenu),
}
pub struct Menu {
    handle: HMENU,
}
impl Menu {
    unsafe fn from_raw(handle: HMENU) -> Menu {
        Menu {
            handle: handle,
        }
    }
    fn as_raw(&self) -> HMENU {
        self.handle
    }
    fn into_raw(self) -> HMENU {
        let handle = self.handle;
        forget(self);
        handle
    }
    pub fn append<'a>(
        &mut self, item: MenuItem<'a>, action: MenuAction, status: MenuStatus, check: MenuCheck,
    ) -> Result<(), Error> {
        let mut flags = 0;
        let action = match action {
            MenuAction::Id(n) => n as usize,
            MenuAction::ChildMenu(menu) => {
                flags |= MF_POPUP;
                menu.into_inner().into_raw() as usize
            },
        };
        match status {
            MenuStatus::Enabled => flags |= MF_ENABLED,
            MenuStatus::Disabled => flags |= MF_DISABLED,
            MenuStatus::Grayed => flags |= MF_GRAYED,
        }
        match check {
            MenuCheck::Checked => flags |= MF_CHECKED,
            MenuCheck::Unchecked => flags |= MF_UNCHECKED,
        }
        if unsafe { match item {
            MenuItem::String(string) => AppendMenuW(
                self.handle, flags | MF_STRING, action, string.to_wide_null().as_ptr(),
            ),
            MenuItem::Separator => AppendMenuW(
                self.handle, flags | MF_SEPARATOR, action, null(),
            ),
        }} == 0 {
            return Err(Error::get_last_error());
        }
        Ok(())
    }
}
impl Drop for Menu {
    fn drop(&mut self) {
        println!("Dropping!");
        if unsafe { DestroyMenu(self.handle) } == 0 {
            Error::get_last_error().die("Failed to destroy menu");
        }
    }
}
pub struct PopupMenu(Menu);
impl PopupMenu {
    pub fn new() -> Result<PopupMenu, Error> {
        let menu = unsafe { CreatePopupMenu() };
        if menu.is_null() {
            return Err(Error::get_last_error());
        }
        Ok(PopupMenu(unsafe { Menu::from_raw(menu) }))
    }
    pub fn display(&self, hwnd: HWND, x: i32, y: i32) -> Result<u16, Error> {
        unsafe {
            let ret = SetForegroundWindow(hwnd);
            if ret == 0 {
                return Err(Error::get_last_error());
            }
            let ret = TrackPopupMenuEx(self.handle, TPM_RETURNCMD, x, y, hwnd, null_mut());
            if ret == 0 && GetLastError() != 0 {
                return Err(Error::get_last_error());
            }
            Ok(ret as u16)
        }
    }
    fn into_inner(self) -> Menu {
        self.0
    }
}
impl Deref for PopupMenu {
    type Target = Menu;
    fn deref(&self) -> &Menu {
        &self.0
    }
}
impl DerefMut for PopupMenu {
    fn deref_mut(&mut self) -> &mut Menu {
        &mut self.0
    }
}
pub struct MenuBar(Menu);
impl Deref for MenuBar {
    type Target = Menu;
    fn deref(&self) -> &Menu {
        &self.0
    }
}
impl DerefMut for MenuBar {
    fn deref_mut(&mut self) -> &mut Menu {
        &mut self.0
    }
}