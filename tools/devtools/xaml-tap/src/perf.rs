//! The UI thread's frames, from WinUI's own performance events.
//!
//! WinUI 3 writes frames, measure and arrange to the `Microsoft-Windows-XAML`
//! ETW provider. An ordinary session needs an administrator; a private one
//! does not, but only sees its own process - which is why this lives in the
//! tap. While devtools are connected the session writes into a ring file of
//! a few megabytes, so it always holds the last seconds. A capture stops it,
//! reads the ring into frames, and starts it again.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use guinea_devtools_protocol::native::{Frame, Pass};
use windows_core::{GUID, PCWSTR, PWSTR};

use crate::bindings::{
    CONTROLTRACE_ID, CloseTrace, ControlTraceW, EVENT_CONTROL_CODE_ENABLE_PROVIDER, EVENT_RECORD,
    EVENT_TRACE_CONTROL_STOP, EVENT_TRACE_FILE_MODE_CIRCULAR, EVENT_TRACE_LOGFILEW,
    EVENT_TRACE_PRIVATE_IN_PROC, EVENT_TRACE_PRIVATE_LOGGER_MODE, EVENT_TRACE_PROPERTIES,
    EnableTraceEx2, OpenTraceW, PROCESS_TRACE_MODE_EVENT_RECORD, PROPERTY_DATA_DESCRIPTOR,
    ProcessTrace, StartTraceW, TRACE_EVENT_INFO, TRACE_LEVEL_VERBOSE, TdhGetEventInformation,
    TdhGetProperty, TdhGetPropertySize, WNODE_FLAG_TRACED_GUID,
};

const XAML: GUID = GUID::from_u128(0x531a35ab_63ce_4bcf_aa98_f88c7a89e455);
const RING_MB: u32 = 32;
const KEPT_FRAMES: usize = 600;
const KEPT_PASSES: usize = 12;
const INSUFFICIENT_BUFFER: u32 = 122;
const NAME_ROOM: usize = 1024;

struct Session {
    id: CONTROLTRACE_ID,
    name: Vec<u16>,
}

static SESSION: Mutex<Option<Session>> = Mutex::new(None);

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain([0]).collect()
}

fn ring() -> PathBuf {
    std::env::temp_dir().join(format!("guinea-xaml-perf-{}.etl", std::process::id()))
}

fn properties(name: &[u16], file: Option<&[u16]>) -> Vec<u8> {
    let header = size_of::<EVENT_TRACE_PROPERTIES>();
    let mut buffer = vec![0u8; header + NAME_ROOM * 2 + file.map_or(0, |file| file.len() * 2)];
    let size = buffer.len();
    let properties = buffer.as_mut_ptr() as *mut EVENT_TRACE_PROPERTIES;

    unsafe {
        (*properties).Wnode.BufferSize = size as u32;
        (*properties).Wnode.Flags = WNODE_FLAG_TRACED_GUID as u32;
        (*properties).Wnode.ClientContext = 1;
        (*properties).Wnode.Guid = GUID::new().unwrap_or_default();
        (*properties).LogFileMode =
            (EVENT_TRACE_PRIVATE_LOGGER_MODE | EVENT_TRACE_PRIVATE_IN_PROC | EVENT_TRACE_FILE_MODE_CIRCULAR) as u32;
        (*properties).MaximumFileSize = RING_MB;
        (*properties).LoggerNameOffset = header as u32;
        std::ptr::copy_nonoverlapping(name.as_ptr(), buffer.as_mut_ptr().add(header) as *mut u16, name.len().min(NAME_ROOM));

        if let Some(file) = file {
            (*properties).LogFileNameOffset = (header + NAME_ROOM * 2) as u32;
            std::ptr::copy_nonoverlapping(
                file.as_ptr(),
                buffer.as_mut_ptr().add(header + NAME_ROOM * 2) as *mut u16,
                file.len(),
            );
        }
    }
    buffer
}

/// Starts recording, unless it already is.
pub fn start() -> Result<(), String> {
    let mut session = SESSION.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if session.is_some() {
        return Ok(());
    }

    let name = wide(&format!("guinea-xaml-perf-{}", std::process::id()));
    let file = wide(&ring().to_string_lossy());
    let mut buffer = properties(&name, Some(&file));

    let mut id: CONTROLTRACE_ID = 0;
    let started = unsafe { StartTraceW(&mut id, PCWSTR(name.as_ptr()), buffer.as_mut_ptr().cast()) };
    if started != 0 {
        return Err(format!("the ETW session did not start: {started}"));
    }

    let enabled = unsafe {
        EnableTraceEx2(
            id,
            &XAML,
            EVENT_CONTROL_CODE_ENABLE_PROVIDER as u32,
            TRACE_LEVEL_VERBOSE as u8,
            u64::MAX,
            0,
            0,
            std::ptr::null(),
        )
    };
    if enabled != 0 {
        stop_session(id, &name);
        return Err(format!("the XAML provider was not enabled: {enabled}"));
    }

    *session = Some(Session { id, name });
    Ok(())
}

fn stop_session(id: CONTROLTRACE_ID, name: &[u16]) {
    let mut buffer = properties(name, None);
    unsafe { ControlTraceW(id, PCWSTR(name.as_ptr()), buffer.as_mut_ptr().cast(), EVENT_TRACE_CONTROL_STOP as u32) };
}

/// Stops recording, when devtools go away.
pub fn stop() {
    if let Some(session) = SESSION.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).take() {
        stop_session(session.id, &session.name);
    }
}

/// What the ring holds now, as frames; recording goes on afterwards.
pub fn capture() -> Result<Vec<Frame>, String> {
    stop();
    let frames = read(&ring());
    start()?;
    frames
}

#[derive(Default)]
struct Reader {
    first: Option<i64>,
    frame: Option<Open>,
    frames: Vec<Frame>,
    names: HashMap<(u16, u8), (String, String)>,
}

struct Open {
    at: i64,
    open_passes: Vec<(u64, &'static str, i64)>,
    measure: i64,
    arrange: i64,
    passes: Vec<Pass>,
}

fn read(path: &std::path::Path) -> Result<Vec<Frame>, String> {
    let mut file = wide(&path.to_string_lossy());
    let mut reader = Reader::default();

    let mut log = EVENT_TRACE_LOGFILEW {
        LogFileName: PWSTR(file.as_mut_ptr()),
        Context: (&mut reader as *mut Reader).cast(),
        ..Default::default()
    };
    log.Anonymous.ProcessTraceMode = PROCESS_TRACE_MODE_EVENT_RECORD as u32;
    log.Anonymous2.EventRecordCallback = Some(on_event);

    let handle = unsafe { OpenTraceW(&mut log) };
    if handle == u64::MAX {
        return Err(format!("cannot open {}", path.display()));
    }

    let processed = unsafe { ProcessTrace(&handle, 1, std::ptr::null(), std::ptr::null()) };
    unsafe { CloseTrace(handle) };
    if processed != 0 {
        return Err(format!("reading the recording failed: {processed}"));
    }

    let kept = reader.frames.len().saturating_sub(KEPT_FRAMES);
    Ok(reader.frames.split_off(kept))
}

fn names(record: &EVENT_RECORD) -> Option<(String, String)> {
    let mut size = 0u32;
    let first = unsafe { TdhGetEventInformation(record, 0, std::ptr::null(), std::ptr::null_mut(), &mut size) };
    if first != INSUFFICIENT_BUFFER {
        return None;
    }

    let mut buffer = vec![0u8; size as usize];
    let info = buffer.as_mut_ptr() as *mut TRACE_EVENT_INFO;
    if unsafe { TdhGetEventInformation(record, 0, std::ptr::null(), info, &mut size) } != 0 {
        return None;
    }

    let at = |offset: u32| {
        if offset == 0 {
            return String::new();
        }
        let start = unsafe { buffer.as_ptr().add(offset as usize) as *const u16 };
        let mut len = 0;
        while unsafe { *start.add(len) } != 0 {
            len += 1;
        }
        String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(start, len) })
    };

    let (task, opcode, event) = unsafe {
        (
            at((*info).TaskNameOffset),
            at((*info).OpcodeNameOffset),
            at((*info).Anonymous.EventNameOffset),
        )
    };
    let task = if task.is_empty() { event } else { task };
    Some((task.trim().to_string(), opcode.trim().to_string()))
}

fn unsigned(record: &EVENT_RECORD, name: &str) -> Option<u64> {
    let name = wide(name);
    let descriptor = PROPERTY_DATA_DESCRIPTOR {
        PropertyName: name.as_ptr() as u64,
        ArrayIndex: u32::MAX,
        Reserved: 0,
    };

    let mut size = 0u32;
    if unsafe { TdhGetPropertySize(record, 0, std::ptr::null(), 1, &descriptor, &mut size) } != 0 || size > 8 {
        return None;
    }

    let mut value = [0u8; 8];
    if unsafe { TdhGetProperty(record, 0, std::ptr::null(), 1, &descriptor, size, value.as_mut_ptr()) } != 0 {
        return None;
    }
    Some(u64::from_le_bytes(value))
}

fn micros(ticks: i64) -> u64 {
    (ticks.max(0) / 10) as u64
}

unsafe extern "system" fn on_event(record: *mut EVENT_RECORD) {
    let record = unsafe { &*record };
    if record.EventHeader.ProviderId != XAML {
        return;
    }

    let reader = unsafe { &mut *(record.UserContext as *mut Reader) };
    let now = record.EventHeader.TimeStamp;
    let first = *reader.first.get_or_insert(now);

    let descriptor = record.EventHeader.EventDescriptor;
    let key = (descriptor.Id, descriptor.Version);
    let (task, opcode) = if descriptor.Id != 0 {
        match reader.names.get(&key) {
            Some(names) => names.clone(),
            None => {
                let Some(found) = names(record) else { return };
                reader.names.insert(key, found.clone());
                found
            }
        }
    } else {
        return;
    };

    match (task.as_str(), opcode.as_str()) {
        ("Frame", "Start") => {
            reader.frame = Some(Open {
                at: now,
                open_passes: Vec::new(),
                measure: 0,
                arrange: 0,
                passes: Vec::new(),
            });
        }
        ("Frame", "Stop") => {
            if let Some(mut open) = reader.frame.take() {
                open.passes.sort_by_key(|pass| std::cmp::Reverse(pass.took_us));
                open.passes.truncate(KEPT_PASSES);

                reader.frames.push(Frame {
                    at_us: micros(open.at - first),
                    took_us: micros(now - open.at),
                    measure_us: micros(open.measure),
                    arrange_us: micros(open.arrange),
                    passes: open.passes,
                });
            }
        }
        ("MeasureElement" | "ArrangeElement", "Start") => {
            let Some(open) = reader.frame.as_mut() else { return };
            let kind = if task == "MeasureElement" { "measure" } else { "arrange" };
            let element = unsigned(record, "ElementId").unwrap_or(0);
            open.open_passes.push((element, kind, now));
        }
        ("MeasureElement" | "ArrangeElement", "Stop") => {
            let Some(open) = reader.frame.as_mut() else { return };
            let kind = if task == "MeasureElement" { "measure" } else { "arrange" };
            let element = unsigned(record, "ElementId").unwrap_or(0);

            let Some(at) = open
                .open_passes
                .iter()
                .rposition(|(open_element, open_kind, _)| *open_element == element && *open_kind == kind)
            else {
                return;
            };
            let (_, _, started) = open.open_passes.remove(at);
            let took = now - started;

            let outermost = !open.open_passes.iter().any(|(_, open_kind, _)| *open_kind == kind);
            if outermost {
                if kind == "measure" {
                    open.measure += took;
                } else {
                    open.arrange += took;
                }
            }

            open.passes.push(Pass {
                element,
                kind: kind.to_string(),
                took_us: micros(took),
            });
        }
        _ => {}
    }
}
