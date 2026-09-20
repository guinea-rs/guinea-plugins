//! The UI thread, reached from anywhere.
//!
//! Everything in XAML diagnostics but the tree subscription answers only on
//! the thread that owns the tree - from any other it is `RPC_E_WRONG_THREAD`,
//! or a crash. `SetSite` arrives on that thread, so a message-only window made
//! there is a door into it: a job is handed through with `SendMessage`.

use std::sync::atomic::{AtomicIsize, AtomicU32, Ordering};

use windows_core::{BOOL, w};

use crate::bindings::{
    CreateWindowExW, DefWindowProcW, EnumThreadWindows, GA_ROOT, GWL_EXSTYLE, GetAncestor,
    GetCurrentThreadId, GetModuleHandleW, GetWindowLongW, GetWindowThreadProcessId, HWND,
    HWND_MESSAGE, IsWindowVisible, LPARAM, LRESULT, POINT, RegisterClassW, SendMessageW, WM_APP,
    WNDCLASSW, WPARAM, WS_EX_TOOLWINDOW, WindowFromPoint,
};

const RUN: u32 = WM_APP as u32 + 1;

static DOOR: AtomicIsize = AtomicIsize::new(0);
static THREAD: AtomicU32 = AtomicU32::new(0);

unsafe extern "system" fn door(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if message == RUN {
        let job = unsafe { &mut *(lparam as *mut &mut dyn FnMut()) };
        job();
        return 0;
    }

    unsafe { DefWindowProcW(window, message, wparam, lparam) }
}

/// Makes the door. Called once, on the UI thread.
pub fn install() {
    THREAD.store(unsafe { GetCurrentThreadId() }, Ordering::SeqCst);

    let class = w!("guinea-xaml-tap");
    let instance = unsafe { GetModuleHandleW(windows_core::PCWSTR::null()) };
    let registration = WNDCLASSW {
        lpfnWndProc: Some(door),
        hInstance: instance,
        lpszClassName: class,
        ..Default::default()
    };
    unsafe { RegisterClassW(&registration) };

    let window = unsafe {
        CreateWindowExW(
            0,
            class,
            w!(""),
            0,
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            std::ptr::null_mut(),
            instance,
            std::ptr::null(),
        )
    };

    if !window.is_null() {
        DOOR.store(window as isize, Ordering::SeqCst);
    }
}

/// Runs `job` on the UI thread and waits for it; `None` before [`install`].
pub fn on_ui<R>(job: impl FnOnce() -> R) -> Option<R> {
    let window = DOOR.load(Ordering::SeqCst);
    if window == 0 {
        return None;
    }

    let mut job = Some(job);
    let mut result = None;
    let mut run = || result = job.take().map(|job| job());
    let mut run: &mut dyn FnMut() = &mut run;

    unsafe { SendMessageW(window as HWND, RUN, 0, &mut run as *mut _ as LPARAM) };
    result
}

fn is_ours(window: HWND) -> bool {
    let thread = unsafe { GetWindowThreadProcessId(window, std::ptr::null_mut()) };
    thread != 0 && thread == THREAD.load(Ordering::SeqCst)
}

/// The UI thread's top-level window at `point`, in screen pixels.
pub fn window_at(point: POINT) -> Option<HWND> {
    let under = unsafe { WindowFromPoint(point) };
    if under.is_null() {
        return None;
    }

    let root = unsafe { GetAncestor(under, GA_ROOT as u32) };
    is_ours(root).then_some(root)
}

windows_core::link!("user32.dll" "system" fn GetClientRect(window : HWND, rect : *mut crate::diag::RECT) -> BOOL);

/// `window`'s client area in screen pixels; `None` when the window is gone.
pub fn client_area(window: HWND) -> Option<crate::diag::RECT> {
    let mut client = crate::diag::RECT::default();
    if unsafe { GetClientRect(window, &mut client) }.ok().is_err() {
        return None;
    }

    let mut corner = POINT { x: 0, y: 0 };
    let _ = unsafe { crate::bindings::ClientToScreen(window, &mut corner) };

    Some(crate::diag::RECT {
        left: corner.x,
        top: corner.y,
        right: corner.x + (client.right - client.left),
        bottom: corner.y + (client.bottom - client.top),
    })
}

/// The UI thread's visible top-level windows, tool windows left out.
pub fn windows() -> Vec<HWND> {
    unsafe extern "system" fn collect(window: HWND, found: LPARAM) -> BOOL {
        let found = unsafe { &mut *(found as *mut Vec<HWND>) };
        let tool = unsafe { GetWindowLongW(window, GWL_EXSTYLE) } & WS_EX_TOOLWINDOW != 0;

        if unsafe { IsWindowVisible(window) }.as_bool() && !tool {
            found.push(window);
        }
        BOOL(1)
    }

    let mut found: Vec<HWND> = Vec::new();
    let _ = unsafe {
        EnumThreadWindows(
            THREAD.load(Ordering::SeqCst),
            Some(collect),
            &mut found as *mut _ as LPARAM,
        )
    };
    found
}
