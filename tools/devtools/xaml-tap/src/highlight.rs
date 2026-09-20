//! The box drawn over an element, the way Chrome tints what the inspector
//! points at: a translucent window that takes neither the mouse nor the focus.
//!
//! It is owned by the application's window rather than kept on top of
//! everything: Windows then keeps it just above its owner, under whatever
//! covers the owner - devtools included - and hides it with a minimised
//! owner. Created, moved and hidden on the UI thread.

use std::cell::Cell;

use guinea_devtools_protocol::native::Bounds;
use windows_core::w;

use crate::bindings::{
    CreateSolidBrush, CreateWindowExW, DefWindowProcW, DestroyWindow, GetModuleHandleW, HWND,
    LWA_ALPHA, RegisterClassW, SW_HIDE, SWP_NOACTIVATE, SWP_SHOWWINDOW, SetLayeredWindowAttributes,
    SetWindowPos, ShowWindow, WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    WS_EX_TRANSPARENT, WS_POPUP,
};

/// Chrome's inspector blue, as `0x00BBGGRR`.
const TINT: u32 = 0x00_E0_A0_60;
const OPACITY: u8 = 96;

thread_local! {
    /// The box and the window that owns it.
    static BOX: Cell<Option<(HWND, HWND)>> = const { Cell::new(None) };
    static REGISTERED: Cell<bool> = const { Cell::new(false) };
}

fn class() -> windows_core::PCWSTR {
    let class = w!("guinea-xaml-tap-highlight");
    if !REGISTERED.get() {
        let registration = WNDCLASSW {
            lpfnWndProc: Some(DefWindowProcW),
            hInstance: unsafe { GetModuleHandleW(windows_core::PCWSTR::null()) },
            hbrBackground: unsafe { CreateSolidBrush(TINT) },
            lpszClassName: class,
            ..Default::default()
        };
        unsafe { RegisterClassW(&registration) };
        REGISTERED.set(true);
    }
    class
}

/// The box owned by `owner`, made anew when the owner changed.
fn owned_by(owner: HWND) -> Option<HWND> {
    if let Some((window, current)) = BOX.get() {
        if current == owner {
            return Some(window);
        }
        let _ = unsafe { DestroyWindow(window) };
        BOX.set(None);
    }

    let style = WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE;
    let window = unsafe {
        CreateWindowExW(
            style as u32,
            class(),
            w!(""),
            WS_POPUP,
            0,
            0,
            0,
            0,
            owner,
            std::ptr::null_mut(),
            GetModuleHandleW(windows_core::PCWSTR::null()),
            std::ptr::null(),
        )
    };
    if window.is_null() {
        return None;
    }

    let _ = unsafe { SetLayeredWindowAttributes(window, 0, OPACITY, LWA_ALPHA as u32) };
    BOX.set(Some((window, owner)));
    Some(window)
}

/// Puts the box over `bounds`, in the window `owner`.
pub fn show(bounds: Bounds, owner: HWND) {
    if bounds.width <= 0 || bounds.height <= 0 {
        hide();
        return;
    }
    let Some(window) = owned_by(owner) else {
        return;
    };

    let _ = unsafe {
        SetWindowPos(
            window,
            std::ptr::null_mut(),
            bounds.x,
            bounds.y,
            bounds.width,
            bounds.height,
            (SWP_NOACTIVATE | SWP_SHOWWINDOW) as u32,
        )
    };
}

pub fn hide() {
    if let Some((window, _)) = BOX.get() {
        let _ = unsafe { ShowWindow(window, SW_HIDE) };
    }
}
