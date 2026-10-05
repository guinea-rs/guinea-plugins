//! Pointing at an element, the way Chrome's inspector does. This half runs in
//! devtools.
//!
//! While a [`Picker`] lives, a low-level mouse hook on a thread of its own
//! notes where the pointer is, and swallows a left click over the target
//! process's windows: the click picks the element under it instead of
//! pressing whatever that is.

use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::bindings::{
    CallNextHookEx, GA_ROOT, GetAncestor, GetCurrentThreadId, GetCursorPos, GetMessageW,
    GetModuleHandleW, GetWindowThreadProcessId, LPARAM, LRESULT, MSG, MSLLHOOKSTRUCT, POINT,
    PostThreadMessageW, SetWindowsHookExW, UnhookWindowsHookEx, WH_MOUSE_LL, WM_LBUTTONDOWN,
    WM_LBUTTONUP, WM_MOUSEMOVE, WM_QUIT, WPARAM, WindowFromPoint,
};

#[derive(Default)]
struct Seen {
    pid: u32,
    at: Option<(i32, i32)>,
    clicked: bool,
}

static SEEN: Mutex<Seen> = Mutex::new(Seen {
    pid: 0,
    at: None,
    clicked: false,
});
static THREAD: AtomicU32 = AtomicU32::new(0);

fn seen<R>(job: impl FnOnce(&mut Seen) -> R) -> R {
    job(&mut SEEN.lock().unwrap_or_else(|poisoned| poisoned.into_inner()))
}

fn owner(x: i32, y: i32) -> u32 {
    let under = unsafe { WindowFromPoint(POINT { x, y }) };
    if under.is_null() {
        return 0;
    }

    let root = unsafe { GetAncestor(under, GA_ROOT as u32) };
    let mut pid = 0;
    unsafe { GetWindowThreadProcessId(root, &mut pid) };
    pid
}

unsafe extern "system" fn hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let info = unsafe { &*(lparam as *const MSLLHOOKSTRUCT) };
        let (x, y) = (info.pt.x, info.pt.y);
        let message = wparam as u32;

        if message == WM_MOUSEMOVE as u32 {
            seen(|seen| seen.at = Some((x, y)));
        } else if message == WM_LBUTTONDOWN as u32 || message == WM_LBUTTONUP as u32 {
            let target = seen(|seen| seen.pid);
            if target != 0 && owner(x, y) == target {
                if message == WM_LBUTTONDOWN as u32 {
                    seen(|seen| {
                        seen.at = Some((x, y));
                        seen.clicked = true;
                    });
                }
                return 1;
            }
        }
    }

    unsafe { CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam) }
}

/// A pick in progress; dropping it gives the mouse back.
pub struct Picker {
    _private: (),
}

impl Picker {
    /// Starts watching the pointer for process `pid`. One at a time: a second
    /// picker replaces the first one's target.
    pub fn start(pid: u32) -> Picker {
        seen(|seen| {
            *seen = Seen {
                pid,
                ..Seen::default()
            }
        });

        if THREAD.load(Ordering::SeqCst) == 0 {
            std::thread::Builder::new()
                .name("devtools-pick".into())
                .spawn(|| {
                    THREAD.store(unsafe { GetCurrentThreadId() }, Ordering::SeqCst);

                    let module = unsafe { GetModuleHandleW(windows_core::PCWSTR::null()) };
                    let hooked = unsafe { SetWindowsHookExW(WH_MOUSE_LL, Some(hook), module, 0) };

                    let mut message = MSG::default();
                    while unsafe { GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) }.as_bool()
                    {
                    }

                    if !hooked.is_null() {
                        let _ = unsafe { UnhookWindowsHookEx(hooked) };
                    }
                    THREAD.store(0, Ordering::SeqCst);
                })
                .expect("spawning the pick thread");
        }

        Picker { _private: () }
    }

    /// Where the pointer is, in screen pixels.
    ///
    /// Read from the cursor rather than the hook: a cursor moved by
    /// `SetCursorPos` - a pen driver, an automation tool - sends no mouse
    /// input for the hook to see.
    pub fn at(&self) -> Option<(i32, i32)> {
        let mut cursor = POINT::default();
        if unsafe { GetCursorPos(&mut cursor) }.as_bool() {
            return Some((cursor.x, cursor.y));
        }
        seen(|seen| seen.at)
    }

    /// Whether a click over the target landed since the last call.
    pub fn take_click(&self) -> bool {
        seen(|seen| std::mem::take(&mut seen.clicked))
    }
}

impl Drop for Picker {
    fn drop(&mut self) {
        seen(|seen| seen.pid = 0);

        let thread = THREAD.load(Ordering::SeqCst);
        if thread != 0 {
            let _ = unsafe { PostThreadMessageW(thread, WM_QUIT as u32, 0, 0) };
        }
    }
}
