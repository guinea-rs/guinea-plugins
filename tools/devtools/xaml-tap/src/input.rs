//! Clicks and keystrokes, aimed by the mark an element carries.
//!
//! The element is found in the XAML tree, where every element is - a border
//! with a click handler included, which UI Automation never sees. What
//! happens next is the caller's choice. [`Input::Pointer`] is the real
//! pointer and keyboard at the element's place on the screen: the window
//! comes to the front, the cursor moves there and back, and hit testing and
//! routing are WinUI's own. [`Input::Automation`] is the control's automation
//! pattern, found by the same mark: no pointer and no focus change, and only
//! for a control that has a pattern to use.

use guinea_devtools_protocol::native::{Bounds, Input, Target};
use std::mem::ManuallyDrop;

use windows_core::{BSTR, IUnknown, Interface};

use crate::bindings::{
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CUIAutomation, CoCreateInstance, CoInitializeEx,
    GetCursorPos, GetSystemMetrics, HWND, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE,
    IUIAutomation, IUIAutomationElement, IUIAutomationInvokePattern, IUIAutomationTogglePattern,
    IUIAutomationValuePattern, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE,
    MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE,
    MOUSEEVENTF_VIRTUALDESK, MOUSEINPUT, POINT, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
    SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SendInput, SetForegroundWindow, VARIANT, VARIANT_0,
    VARIANT_0_0, VARIANT_0_0_0,
};
use crate::inspect::{self, Inspector};
use crate::{tree, ui};

/// UI Automation's `AutomationId` property.
const AUTOMATION_ID: i32 = 30011;
/// UI Automation's patterns, by id.
const INVOKE: i32 = 10000;
const VALUE: i32 = 10002;
const TOGGLE: i32 = 10015;
/// Every element below, not only the children.
const DESCENDANTS: i32 = 4;
/// A `VARIANT` holding a `BSTR`.
const VT_BSTR: u16 = 8;

/// The elements below `under`, in tree order, that carry `mark`.
fn marked<'a>(
    inspector: &'a Inspector,
    under: Option<u64>,
    mark: &'a str,
) -> impl Iterator<Item = u64> + 'a {
    tree::in_order(under)
        .into_iter()
        .filter(move |&element| inspector.mark(element).as_deref() == Some(mark))
}

/// The element `target` names, and where it is on the screen.
pub fn find(target: &Target) -> Result<(u64, Option<(Bounds, HWND)>), String> {
    inspect::with(|inspector| {
        let under = match &target.within {
            Some(within) => Some(
                marked(inspector, None, within)
                    .next()
                    .ok_or_else(|| format!("nothing is marked {within:?}"))?,
            ),
            None => None,
        };

        let element = marked(inspector, under, &target.mark)
            .nth(target.nth)
            .ok_or_else(|| {
                let count = marked(inspector, under, &target.mark).count();
                format!(
                    "there is no element {} marked {:?} - {count} carry that mark",
                    target.nth, target.mark
                )
            })?;

        Ok((element, inspector.bounds(element)))
    })
}

pub fn click(target: &Target, input: Input) -> Result<(), String> {
    match input {
        Input::Pointer => {
            let (x, y) = aim(target)?;
            press(x, y)
        }
        Input::Automation => {
            let element = located(target)?;

            if let Some(invoke) = pattern::<IUIAutomationInvokePattern>(&element, INVOKE) {
                return unsafe { invoke.Invoke() }
                    .ok()
                    .map_err(|error| format!("invoking it: {error}"));
            }
            if let Some(toggle) = pattern::<IUIAutomationTogglePattern>(&element, TOGGLE) {
                return unsafe { toggle.Toggle() }
                    .ok()
                    .map_err(|error| format!("toggling it: {error}"));
            }

            Err(format!(
                "{:?} has no pattern to click with - use the pointer instead",
                target.mark
            ))
        }
    }
}

pub fn type_text(target: &Target, text: &str, input: Input) -> Result<(), String> {
    match input {
        Input::Pointer => {
            let (x, y) = aim(target)?;
            press(x, y)?;
            keys(text)
        }
        Input::Automation => {
            let element = located(target)?;
            let value = pattern::<IUIAutomationValuePattern>(&element, VALUE).ok_or_else(|| {
                format!(
                    "{:?} takes no value - use the pointer and the keyboard instead",
                    target.mark
                )
            })?;

            unsafe { value.SetValue(&BSTR::from(text)) }
                .ok()
                .map_err(|error| format!("setting the value: {error}"))
        }
    }
}

/// The middle of the element `target` names, with its window in front and
/// nothing else over that point.
fn aim(target: &Target) -> Result<(i32, i32), String> {
    let (_, placed) = find(target)?;
    let (bounds, window) =
        placed.ok_or_else(|| format!("{:?} is not on the screen", target.mark))?;

    if bounds.width <= 0 || bounds.height <= 0 {
        return Err(format!(
            "{:?} has no size: collapsed, or not laid out",
            target.mark
        ));
    }

    let (x, y) = (bounds.x + bounds.width / 2, bounds.y + bounds.height / 2);

    let _ = unsafe { SetForegroundWindow(window) };
    if ui::window_at(POINT { x, y }) != Some(window) {
        return Err(format!(
            "something else covers {:?} at ({x}, {y}) - a click there would land on it",
            target.mark
        ));
    }

    Ok((x, y))
}

/// A left click at `(x, y)` on the screen, and the cursor back where it was.
fn press(x: i32, y: i32) -> Result<(), String> {
    let mut was = POINT { x: 0, y: 0 };
    let _ = unsafe { GetCursorPos(&mut was) };

    send(&[
        mouse(Some((x, y)), 0),
        mouse(None, MOUSEEVENTF_LEFTDOWN as u32),
        mouse(None, MOUSEEVENTF_LEFTUP as u32),
        mouse(Some((was.x, was.y)), 0),
    ])
}

/// `text` as keystrokes, one UTF-16 unit each, to whatever has focus.
fn keys(text: &str) -> Result<(), String> {
    let strokes: Vec<INPUT> = text
        .encode_utf16()
        .flat_map(|unit| {
            [0, KEYEVENTF_KEYUP as u32].map(|up| INPUT {
                r#type: INPUT_KEYBOARD as u32,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: 0,
                        wScan: unit,
                        dwFlags: KEYEVENTF_UNICODE as u32 | up,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            })
        })
        .collect();

    send(&strokes)
}

/// A mouse event; with a place, a move there first.
fn mouse(to: Option<(i32, i32)>, flags: u32) -> INPUT {
    let (dx, dy, moved) = match to {
        Some((x, y)) => {
            let (left, top, width, height) = unsafe {
                (
                    GetSystemMetrics(SM_XVIRTUALSCREEN),
                    GetSystemMetrics(SM_YVIRTUALSCREEN),
                    GetSystemMetrics(SM_CXVIRTUALSCREEN).max(1),
                    GetSystemMetrics(SM_CYVIRTUALSCREEN).max(1),
                )
            };
            (
                ((x - left) * 65535) / width,
                ((y - top) * 65535) / height,
                (MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK) as u32,
            )
        }
        None => (0, 0, 0),
    };

    INPUT {
        r#type: INPUT_MOUSE as u32,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: 0,
                dwFlags: moved | flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn send(inputs: &[INPUT]) -> Result<(), String> {
    let sent = unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        )
    };

    if sent as usize == inputs.len() {
        Ok(())
    } else {
        Err("the input was blocked - by another desktop, or by a window of higher integrity".into())
    }
}

/// The element `target` names, as UI Automation has it.
fn located(target: &Target) -> Result<IUIAutomationElement, String> {
    let automation = automation()?;
    let marked = |from: &IUIAutomationElement, mark: &str| -> Vec<IUIAutomationElement> {
        let mut value = text(mark);
        let found = unsafe {
            automation
                .CreatePropertyCondition(AUTOMATION_ID, &value)
                .and_then(|condition| from.FindAll(DESCENDANTS, &condition))
        };
        unsafe { ManuallyDrop::drop(&mut (*value.Anonymous.Anonymous).Anonymous.bstrVal) };
        let Ok(found) = found else {
            return Vec::new();
        };

        let count = unsafe { found.Length() }.unwrap_or(0);
        (0..count)
            .filter_map(|at| unsafe { found.GetElement(at) }.ok())
            .collect()
    };

    for window in ui::windows() {
        let Ok(root) = (unsafe { automation.ElementFromHandle(window) }) else {
            continue;
        };

        let under = match &target.within {
            Some(within) => match marked(&root, within).into_iter().next() {
                Some(under) => under,
                None => continue,
            },
            None => root,
        };

        if let Some(element) = marked(&under, &target.mark).into_iter().nth(target.nth) {
            return Ok(element);
        }
    }

    Err(format!(
        "UI Automation has no element {} marked {:?} - an element without an automation \
         peer, a border say, is only reachable with the pointer",
        target.nth, target.mark
    ))
}

/// `mark` as the `VARIANT` a property condition compares with. The string is
/// the caller's to free once the condition is made.
fn text(mark: &str) -> VARIANT {
    VARIANT {
        Anonymous: VARIANT_0 {
            Anonymous: ManuallyDrop::new(VARIANT_0_0 {
                vt: VT_BSTR,
                wReserved1: 0,
                wReserved2: 0,
                wReserved3: 0,
                Anonymous: VARIANT_0_0_0 {
                    bstrVal: ManuallyDrop::new(BSTR::from(mark)),
                },
            }),
        },
    }
}

fn automation() -> Result<IUIAutomation, String> {
    unsafe {
        let _ = CoInitializeEx(std::ptr::null(), COINIT_MULTITHREADED as u32);

        let mut raw = std::ptr::null_mut();
        CoCreateInstance(
            &CUIAutomation,
            std::ptr::null_mut(),
            CLSCTX_INPROC_SERVER,
            &IUIAutomation::IID,
            &mut raw,
        )
        .ok()
        .map_err(|error| format!("starting UI Automation: {error}"))?;

        Ok(IUIAutomation::from_raw(raw))
    }
}

fn pattern<P: Interface>(element: &IUIAutomationElement, id: i32) -> Option<P> {
    let found: IUnknown = unsafe { element.GetCurrentPattern(id) }.ok()?;
    found.cast().ok()
}
