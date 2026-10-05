//! The UI thread's sampled stacks, kept across captures.
//!
//! Each capture names its functions and stacks by its own indices; kept
//! here they are named by one table, so a stack is the same number however
//! many captures it came in.

use std::collections::HashMap;

use guinea_devtools_protocol::native::{Sample, Stacks};

/// How many samples are kept: at a thousand a second, two minutes of them.
pub const KEPT_SAMPLES: usize = 120_000;

/// Whose code a sampled function is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Origin {
    #[default]
    App,
    /// Rust's own `std`, `core` and `alloc`, compiled into the application.
    Rust,
    /// WinUI and the rest of the Windows App SDK, installed or shipped
    /// beside the application.
    WinUi,
    /// The operating system's own modules.
    Windows,
    /// An address in no module the sampler knew.
    Unknown,
}

impl Origin {
    /// From the function's name and the path of its module.
    pub fn of(function: &str, module: Option<&str>) -> Origin {
        let Some(module) = module else {
            return Origin::Unknown;
        };
        let path = module.to_ascii_lowercase();
        let file = path.rsplit(['\\', '/']).next().unwrap_or(&path);

        if path.get(1..).is_some_and(|path| path.starts_with(r":\windows\")) {
            Origin::Windows
        } else if path.contains(r"\windowsapps\microsoft.windowsappruntime")
            || file.starts_with("microsoft.")
            || WINDOWS_APP_SDK.contains(&file)
        {
            Origin::WinUi
        } else if ["std::", "core::", "alloc::"]
            .iter()
            .any(|path| function.trim_start_matches('<').starts_with(path))
        {
            Origin::Rust
        } else {
            Origin::App
        }
    }
}

/// The Windows App SDK's modules that are not named `Microsoft.*`, as an
/// application that ships it beside itself has them.
const WINDOWS_APP_SDK: &[&str] = &[
    "coremessagingxp.dll",
    "dcompi.dll",
    "dwmcorei.dll",
    "dwmscenei.dll",
    "marshal.dll",
    "mrm.dll",
    "wuceffectsi.dll",
];

/// The system calls a thread waits in, with neither `Nt` nor `Zw`.
const WAITS: &[&str] = &[
    "UserGetMessage",
    "UserMsgWaitForMultipleObjectsEx",
    "UserWaitMessage",
    "WaitForSingleObject",
    "WaitForMultipleObjects",
    "WaitForAlertByThreadId",
    "SignalAndWaitForSingleObject",
    "DelayExecution",
    "RemoveIoCompletion",
    "RemoveIoCompletionEx",
    "WaitForWorkViaWorkerFactory",
];

/// Whether a function the thread was sampled innermost in is the operating
/// system waiting for something: a message, an object, time.
pub fn waits(function: &str) -> bool {
    let name = function.rsplit_once('!').map_or(function, |(_, name)| name);
    let name = name.split_once("+0x").map_or(name, |(name, _)| name);
    name.strip_prefix("Nt")
        .or_else(|| name.strip_prefix("Zw"))
        .is_some_and(|call| WAITS.contains(&call))
}

/// Every sample since the inspector came, oldest first: the newest
/// [`KEPT_SAMPLES`] of them.
#[derive(Clone, Debug, Default)]
pub struct Sampled {
    /// Every function a kept stack names.
    pub functions: Vec<String>,
    /// Indices into `functions`, the innermost call first.
    pub stacks: Vec<Vec<u32>>,
    /// By `qpc`, each naming an index into `stacks`.
    pub samples: Vec<Sample>,
    /// For each of `functions`, an index into `modules`.
    origins: Vec<Option<u32>>,
    /// For each of `functions`, whose code it is.
    kinds: Vec<Origin>,
    modules: Vec<String>,
    function_ids: HashMap<String, u32>,
    module_ids: HashMap<String, u32>,
    stack_ids: HashMap<Vec<u32>, u32>,
}

impl Sampled {
    /// Takes in a capture's stacks: each sample once, however many captures
    /// it was in.
    pub fn absorb(&mut self, stacks: Stacks) {
        if stacks.samples.is_empty() {
            return;
        }

        let modules: Vec<u32> = stacks
            .modules
            .into_iter()
            .map(|path| self.module_id(path))
            .collect();
        let functions: Vec<u32> = stacks
            .functions
            .into_iter()
            .enumerate()
            .map(|(local, name)| {
                let id = self.function_id(name);
                let origin = stacks
                    .origins
                    .get(local)
                    .copied()
                    .flatten()
                    .and_then(|module| modules.get(module as usize).copied());
                if let Some(kept) = self.origins.get_mut(id as usize)
                    && kept.is_none()
                {
                    *kept = origin;
                    self.kinds[id as usize] =
                        Origin::of(&self.functions[id as usize], self.module(id));
                }
                id
            })
            .collect();
        let stack_ids: Vec<u32> = stacks
            .stacks
            .into_iter()
            .map(|stack| {
                let named = stack
                    .iter()
                    .filter_map(|local| functions.get(*local as usize).copied())
                    .collect();
                self.stack_id(named)
            })
            .collect();

        for sample in stacks.samples {
            let Some(&stack) = stack_ids.get(sample.stack as usize) else {
                continue;
            };
            if let Err(at) = self
                .samples
                .binary_search_by_key(&sample.qpc, |kept| kept.qpc)
            {
                self.samples.insert(
                    at,
                    Sample {
                        qpc: sample.qpc,
                        stack,
                    },
                );
            }
        }

        let over = self.samples.len().saturating_sub(KEPT_SAMPLES);
        self.samples.drain(..over);
    }

    fn function_id(&mut self, name: String) -> u32 {
        if let Some(id) = self.function_ids.get(&name) {
            return *id;
        }
        let id = self.functions.len() as u32;
        self.kinds.push(Origin::of(&name, None));
        self.functions.push(name.clone());
        self.origins.push(None);
        self.function_ids.insert(name, id);
        id
    }

    fn module_id(&mut self, path: String) -> u32 {
        if let Some(id) = self.module_ids.get(&path) {
            return *id;
        }
        let id = self.modules.len() as u32;
        self.modules.push(path.clone());
        self.module_ids.insert(path, id);
        id
    }

    fn stack_id(&mut self, stack: Vec<u32>) -> u32 {
        if let Some(id) = self.stack_ids.get(&stack) {
            return *id;
        }
        let id = self.stacks.len() as u32;
        self.stacks.push(stack.clone());
        self.stack_ids.insert(stack, id);
        id
    }

    /// The function `id` names.
    pub fn function(&self, id: u32) -> &str {
        self.functions.get(id as usize).map_or("", String::as_str)
    }

    /// Whose code function `id` is.
    pub fn origin(&self, id: u32) -> Origin {
        self.kinds
            .get(id as usize)
            .copied()
            .unwrap_or(Origin::Unknown)
    }

    /// The path of the module function `id` is in, when it is in one.
    pub fn module(&self, id: u32) -> Option<&str> {
        let module = self.origins.get(id as usize).copied().flatten()?;
        self.modules.get(module as usize).map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capture(functions: &[&str], stacks: &[&[u32]], samples: &[(u64, u32)]) -> Stacks {
        Stacks {
            functions: functions.iter().map(|name| name.to_string()).collect(),
            stacks: stacks.iter().map(|stack| stack.to_vec()).collect(),
            samples: samples
                .iter()
                .map(|&(qpc, stack)| Sample { qpc, stack })
                .collect(),
            ..Stacks::default()
        }
    }

    fn named(sampled: &Sampled) -> Vec<(u64, Vec<&str>)> {
        sampled
            .samples
            .iter()
            .map(|sample| {
                let stack = &sampled.stacks[sample.stack as usize];
                (
                    sample.qpc,
                    stack.iter().map(|id| sampled.function(*id)).collect(),
                )
            })
            .collect()
    }

    #[test]
    fn captures_that_number_their_functions_differently_land_on_one_table() {
        let mut sampled = Sampled::default();

        sampled.absorb(capture(
            &["main", "draw"],
            &[&[1, 0]],
            &[(10, 0), (20, 0)],
        ));
        sampled.absorb(capture(
            &["measure", "draw", "main"],
            &[&[1, 2], &[0, 1, 2]],
            &[(20, 0), (30, 1), (5, 0)],
        ));

        assert_eq!(
            named(&sampled),
            [
                (5, vec!["draw", "main"]),
                (10, vec!["draw", "main"]),
                (20, vec!["draw", "main"]),
                (30, vec!["measure", "draw", "main"]),
            ]
        );
        assert_eq!(sampled.functions, ["main", "draw", "measure"]);
        assert_eq!(sampled.stacks.len(), 2, "draw under main is one stack");
    }

    #[test]
    fn a_function_knows_its_module_whichever_capture_named_it() {
        let mut sampled = Sampled::default();
        sampled.absorb(Stacks {
            modules: vec![r"C:\app\app.exe".into(), r"C:\Windows\System32\ntdll.dll".into()],
            origins: vec![Some(1), Some(0), None],
            ..capture(&["NtWait", "main", "0x7ff0"], &[&[0, 1, 2]], &[(1, 0)])
        });
        sampled.absorb(Stacks {
            modules: vec![r"C:\app\app.exe".into()],
            origins: vec![Some(0)],
            ..capture(&["draw"], &[&[0]], &[(2, 0)])
        });

        let modules: Vec<(&str, Option<&str>)> = (0..sampled.functions.len() as u32)
            .map(|id| (sampled.function(id), sampled.module(id)))
            .collect();
        assert_eq!(
            modules,
            [
                ("NtWait", Some(r"C:\Windows\System32\ntdll.dll")),
                ("main", Some(r"C:\app\app.exe")),
                ("0x7ff0", None),
                ("draw", Some(r"C:\app\app.exe")),
            ]
        );
        let origins: Vec<Origin> = (0..sampled.functions.len() as u32)
            .map(|id| sampled.origin(id))
            .collect();
        assert_eq!(
            origins,
            [Origin::Windows, Origin::App, Origin::Unknown, Origin::App]
        );
    }

    #[test]
    fn whose_code_a_function_is_comes_from_its_module_and_its_name() {
        const APP: &str = r"C:\dev\uniproc\target\release\uniproc.exe";
        let cases = [
            ("uniproc::ui::render", Some(APP), Origin::App),
            ("core::ops::function::FnOnce::call_once", Some(APP), Origin::Rust),
            ("std::sys::backtrace::__rust_begin_short_backtrace", Some(APP), Origin::Rust),
            ("<alloc::vec::Vec<u8> as core::ops::drop::Drop>::drop", Some(APP), Origin::Rust),
            ("<uniproc::Row as core::fmt::Debug>::fmt", Some(APP), Origin::App),
            (
                "win32u.dll!NtUserGetMessage+0x14",
                Some(r"C:\WINDOWS\System32\win32u.dll"),
                Origin::Windows,
            ),
            (
                "Microsoft.UI.Xaml.dll+0x1a2b",
                Some(
                    r"C:\Program Files\WindowsApps\Microsoft.WindowsAppRuntime.1.6_6000.311.13.0_x64__8wekyb3d8bbwe\Microsoft.UI.Xaml.dll",
                ),
                Origin::WinUi,
            ),
            (
                "Microsoft.ui.xaml.dll+0x10",
                Some(r"C:\dev\uniproc\target\release\Microsoft.ui.xaml.dll"),
                Origin::WinUi,
            ),
            (
                "CoreMessagingXP.dll!CreateDispatcherQueue+0x40",
                Some(r"C:\dev\uniproc\target\release\CoreMessagingXP.dll"),
                Origin::WinUi,
            ),
            ("0x7ff6a0001000", None, Origin::Unknown),
        ];

        for (function, module, origin) in cases {
            assert_eq!(Origin::of(function, module), origin, "{function}");
        }
    }

    #[test]
    fn waiting_is_the_system_call_a_thread_waits_in() {
        for waiting in [
            "win32u.dll!NtUserGetMessage+0x14",
            "win32u.dll!NtUserMsgWaitForMultipleObjectsEx+0x14",
            "ntdll.dll!NtWaitForSingleObject+0x14",
            "ZwWaitForMultipleObjects",
            "ntdll.dll!NtDelayExecution+0x14",
        ] {
            assert!(waits(waiting), "{waiting}");
        }
        for working in [
            "ntdll.dll!RtlAllocateHeap+0x2f",
            "uniproc::wait_for_message",
            "win32u.dll!NtUserPeekMessage+0x14",
        ] {
            assert!(!waits(working), "{working}");
        }
    }

    #[test]
    fn only_the_newest_samples_are_kept() {
        let mut sampled = Sampled::default();
        let samples: Vec<(u64, u32)> = (0..KEPT_SAMPLES as u64 + 10).map(|qpc| (qpc, 0)).collect();

        sampled.absorb(capture(&["main"], &[&[0]], &samples));

        assert_eq!(sampled.samples.len(), KEPT_SAMPLES);
        assert_eq!(sampled.samples[0].qpc, 10);
    }
}
