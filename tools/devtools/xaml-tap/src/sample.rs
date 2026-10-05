//! The UI thread's stack, looked at about a thousand times a second while
//! devtools have sampling on.
//!
//! A thread of the tap's own suspends the UI thread, reads its registers,
//! walks its stack and lets it go. While the UI thread is suspended the
//! sampler waits on nothing the UI thread could be holding - the heap, the
//! loader's lock - so in that window it allocates nothing and finds unwind
//! data in a table of the process's modules read beforehand, never through
//! `RtlLookupFunctionEntry`, which takes the loader's lock. Addresses are
//! named afterwards, when devtools take them, through dbghelp.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;

use guinea_devtools_protocol::native::{Sample, Stacks};

use crate::bindings::{
    CloseHandle, CreateToolhelp32Snapshot, HANDLE, MODULEENTRY32W, Module32FirstW, Module32NextW,
    OpenThread, TH32CS_SNAPMODULE, THREAD_GET_CONTEXT, THREAD_QUERY_INFORMATION,
    THREAD_SUSPEND_RESUME,
};

/// The deepest stack walked; deeper calls are left out.
const DEPTH: usize = 128;
/// How many samples wait to be taken at most, the newest kept: a minute of
/// them, for while nobody takes them.
const KEPT: usize = 60_000;
/// How many samples go by before the table of modules is read again.
const MODULES_EVERY: u32 = 1_000;

static SAMPLER: Mutex<Option<Running>> = Mutex::new(None);
static TAKEN: Mutex<VecDeque<(u64, Vec<u64>)>> = Mutex::new(VecDeque::new());
static NAMES: Mutex<Option<Names>> = Mutex::new(None);

struct Running {
    thread: u32,
    stop: Arc<AtomicBool>,
    handle: JoinHandle<()>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Starts sampling the stack of `thread`, unless it already is.
pub fn start(thread: u32) -> Result<(), String> {
    if !cfg!(target_arch = "x86_64") {
        return Err("stacks are sampled on x86-64 only".to_string());
    }

    let mut running = lock(&SAMPLER);
    if running
        .as_ref()
        .is_some_and(|running| running.thread == thread && !running.handle.is_finished())
    {
        return Ok(());
    }
    if let Some(old) = running.take() {
        old.stop.store(true, Ordering::SeqCst);
        let _ = old.handle.join();
    }

    let access = (THREAD_SUSPEND_RESUME | THREAD_GET_CONTEXT | THREAD_QUERY_INFORMATION) as u32;
    let target = unsafe { OpenThread(access, false.into(), thread) };
    if target.is_null() {
        return Err(format!(
            "thread {thread} cannot be opened: {}",
            std::io::Error::last_os_error()
        ));
    }

    let stop = Arc::new(AtomicBool::new(false));
    let handle = std::thread::Builder::new()
        .name("guinea-xaml-tap-sampler".into())
        .spawn({
            let stop = Arc::clone(&stop);
            let target = target as usize;
            move || sampling(target as HANDLE, &stop)
        })
        .map_err(|error| {
            let _ = unsafe { CloseHandle(target) };
            format!("the sampler did not start: {error}")
        })?;

    *running = Some(Running {
        thread,
        stop,
        handle,
    });
    Ok(())
}

/// Stops sampling; what was sampled stays until it is taken.
pub fn stop() {
    if let Some(running) = lock(&SAMPLER).take() {
        running.stop.store(true, Ordering::SeqCst);
        let _ = running.handle.join();
    }
}

/// What was sampled since the last take, oldest first, every address named.
pub fn take() -> Stacks {
    let taken = std::mem::take(&mut *lock(&TAKEN));
    if taken.is_empty() {
        return Stacks::default();
    }

    let mut names = lock(&NAMES);
    let names = names.get_or_insert_with(Names::new);
    names.refresh();

    let mut interned = Interned::default();
    let mut samples = Vec::with_capacity(taken.len());
    let called_from = |depth: usize, address: u64| if depth == 0 { address } else { address - 1 };
    for (qpc, frames) in &taken {
        for (depth, address) in frames.iter().enumerate() {
            names.learn(called_from(depth, *address));
        }
        let named: Vec<(&str, Option<&str>)> = frames
            .iter()
            .enumerate()
            .map(|(depth, address)| names.name(called_from(depth, *address)))
            .collect();
        samples.push(Sample {
            qpc: *qpc,
            stack: interned.stack(&named),
        });
    }

    interned.into_stacks(samples)
}

fn sampling(target: HANDLE, stop: &AtomicBool) {
    use crate::bindings::{GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_TIME_CRITICAL};

    let _ = unsafe { SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_TIME_CRITICAL) };
    let timer = Timer::new();
    let mut modules = Modules::read();
    let mut frames = [0u64; DEPTH];
    let mut since_modules = 0;

    while !stop.load(Ordering::SeqCst) {
        timer.wait();

        let Some((qpc, depth)) = (unsafe { walk::look(target, &modules, &mut frames) }) else {
            if gone(target) {
                break;
            }
            continue;
        };
        if depth > 0 {
            keep(qpc, &frames[..depth]);
        }

        since_modules += 1;
        if since_modules >= MODULES_EVERY {
            modules = Modules::read();
            since_modules = 0;
        }
    }

    let _ = unsafe { CloseHandle(target) };
}

/// Keeps a sample until it is taken.
fn keep(qpc: u64, frames: &[u64]) {
    let mut taken = lock(&TAKEN);
    if taken.len() >= KEPT {
        taken.pop_front();
    }
    taken.push_back((qpc, frames.to_vec()));
}

/// Whether the thread `target` names has exited.
fn gone(target: HANDLE) -> bool {
    use crate::bindings::WaitForSingleObject;

    let waited = unsafe { WaitForSingleObject(target, 0) };
    waited == 0
}

/// A wait of about a millisecond: a high-resolution waitable timer where
/// there is one, a sleep where there is not.
struct Timer(HANDLE);

impl Timer {
    fn new() -> Self {
        use crate::bindings::{
            CREATE_WAITABLE_TIMER_HIGH_RESOLUTION, CreateWaitableTimerExW, TIMER_ALL_ACCESS,
        };

        Self(unsafe {
            CreateWaitableTimerExW(
                std::ptr::null(),
                windows_core::PCWSTR::null(),
                CREATE_WAITABLE_TIMER_HIGH_RESOLUTION as u32,
                TIMER_ALL_ACCESS as u32,
            )
        })
    }

    fn wait(&self) {
        use crate::bindings::{SetWaitableTimer, WaitForSingleObject};

        let in_a_millisecond: i64 = -10_000;
        let armed = !self.0.is_null()
            && unsafe {
                SetWaitableTimer(
                    self.0,
                    &in_a_millisecond,
                    0,
                    None,
                    std::ptr::null(),
                    false.into(),
                )
            }
            .as_bool();
        if armed {
            let _ = unsafe { WaitForSingleObject(self.0, 1_000) };
        } else {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        if !self.0.is_null() {
            let _ = unsafe { CloseHandle(self.0) };
        }
    }
}

/// A loaded image and its table of functions, as the unwinder needs it.
struct Module {
    base: u64,
    end: u64,
    name: String,
    path: String,
    functions: usize,
    count: usize,
}

/// Every module of the process, by where it is loaded.
#[derive(Default)]
struct Modules(Vec<Module>);

impl Modules {
    fn read() -> Self {
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE as u32, 0) };
        if snapshot.is_null() || snapshot as isize == -1 {
            return Self::default();
        }

        let mut entry = MODULEENTRY32W {
            dwSize: size_of::<MODULEENTRY32W>() as u32,
            ..Default::default()
        };
        let mut modules = Vec::new();
        let mut more = unsafe { Module32FirstW(snapshot, &mut entry) }.as_bool();
        while more {
            let base = entry.modBaseAddr as u64;
            let (functions, count) = unsafe { walk::exceptions(base) }.unwrap_or((0, 0));
            modules.push(Module {
                base,
                end: base + u64::from(entry.modBaseSize),
                name: wide(&entry.szModule),
                path: wide(&entry.szExePath),
                functions,
                count,
            });
            more = unsafe { Module32NextW(snapshot, &mut entry) }.as_bool();
        }
        let _ = unsafe { CloseHandle(snapshot) };

        modules.sort_by_key(|module| module.base);
        Self(modules)
    }

    fn containing(&self, address: u64) -> Option<&Module> {
        let after = self.0.partition_point(|module| module.base <= address);
        self.0
            .get(after.checked_sub(1)?)
            .filter(|module| address < module.end)
    }

    /// Every folder a module was loaded from, each once, as a symbol search
    /// path: rustc writes only the file name of a PDB into the image, and
    /// dbghelp does not look beside the image on its own.
    fn folders(&self) -> String {
        let mut folders: Vec<&str> = self
            .0
            .iter()
            .filter_map(|module| module.path.rsplit_once('\\').map(|(folder, _)| folder))
            .collect();
        folders.sort_unstable();
        folders.dedup();
        folders.join(";")
    }
}

fn wide(units: &[u16]) -> String {
    let length = units
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(units.len());
    String::from_utf16_lossy(&units[..length])
}

/// The names dbghelp gives addresses, each looked up once.
struct Names {
    by_address: HashMap<u64, (String, Option<String>)>,
    modules: Modules,
    searched: String,
}

impl Names {
    fn new() -> Self {
        use crate::bindings::{
            GetCurrentProcess, SYMOPT_DEFERRED_LOADS, SYMOPT_FAIL_CRITICAL_ERRORS, SYMOPT_UNDNAME,
            SymInitializeW, SymSetOptions,
        };

        unsafe {
            SymSetOptions(
                (SYMOPT_UNDNAME | SYMOPT_DEFERRED_LOADS | SYMOPT_FAIL_CRITICAL_ERRORS) as u32,
            );
            let _ = SymInitializeW(GetCurrentProcess(), windows_core::PCWSTR::null(), true.into());
        }

        Self {
            by_address: HashMap::new(),
            modules: Modules::default(),
            searched: String::new(),
        }
    }

    /// Learns of the modules loaded since the last time, and where their
    /// symbols may be.
    fn refresh(&mut self) {
        use crate::bindings::{GetCurrentProcess, SymRefreshModuleList, SymSetSearchPathW};

        self.modules = Modules::read();
        let folders = self.modules.folders();
        if folders != self.searched {
            let path: Vec<u16> = folders.encode_utf16().chain([0]).collect();
            let _ = unsafe {
                SymSetSearchPathW(GetCurrentProcess(), windows_core::PCWSTR(path.as_ptr()))
            };
            self.searched = folders;
        }
        let _ = unsafe { SymRefreshModuleList(GetCurrentProcess()) };
    }

    /// Looks `address` up, unless it was before.
    fn learn(&mut self, address: u64) {
        if self.by_address.contains_key(&address) {
            return;
        }

        let module = self.modules.containing(address);
        let name = match (symbol(address), module) {
            (Some(found), Some(module)) if found.exported => match found.past {
                0 => format!("{}!{}", module.name, found.name),
                past => format!("{}!{}+{past:#x}", module.name, found.name),
            },
            (Some(found), _) => found.name,
            (None, Some(module)) => format!("{}+{:#x}", module.name, address - module.base),
            (None, None) => format!("{address:#x}"),
        };
        let path = module.map(|module| module.path.clone());
        self.by_address.insert(address, (name, path));
    }

    /// What [`Names::learn`] found for `address`: its name, and the path of
    /// the module it is in.
    fn name(&self, address: u64) -> (&str, Option<&str>) {
        self.by_address
            .get(&address)
            .map_or(("(not looked up)", None), |(name, path)| {
                (name.as_str(), path.as_deref())
            })
    }
}

/// What dbghelp calls the code at an address.
struct Symbol {
    name: String,
    /// How far past the symbol's start the address is.
    past: u64,
    /// Whether the name is only the nearest export before the address, the
    /// module having no debug information: then the address may be in some
    /// other function entirely.
    exported: bool,
}

/// The function `address` is in, as the debug information names it.
fn symbol(address: u64) -> Option<Symbol> {
    use crate::bindings::{GetCurrentProcess, SYMBOL_INFOW, SYMFLAG_EXPORT, SymFromAddrW};

    const LONGEST: usize = 1_024;
    let words = (size_of::<SYMBOL_INFOW>() + LONGEST * 2).div_ceil(8);
    let mut buffer = vec![0u64; words];
    let info = buffer.as_mut_ptr().cast::<SYMBOL_INFOW>();
    unsafe {
        (*info).SizeOfStruct = size_of::<SYMBOL_INFOW>() as u32;
        (*info).MaxNameLen = LONGEST as u32;
    }

    let mut displacement = 0u64;
    let found =
        unsafe { SymFromAddrW(GetCurrentProcess(), address, &mut displacement, info) }.as_bool();
    if !found {
        return None;
    }

    let length = unsafe { (*info).NameLen as usize }.min(LONGEST);
    let name = unsafe { std::slice::from_raw_parts((*info).Name.as_ptr(), length) };
    Some(Symbol {
        name: String::from_utf16_lossy(name),
        past: displacement,
        exported: unsafe { (*info).Flags } & SYMFLAG_EXPORT as u32 != 0,
    })
}

/// Names, modules and stacks, each kept once, in the order first seen.
#[derive(Default)]
struct Interned {
    functions: Vec<String>,
    function_ids: HashMap<String, u32>,
    origins: Vec<Option<u32>>,
    modules: Vec<String>,
    module_ids: HashMap<String, u32>,
    stacks: Vec<Vec<u32>>,
    stack_ids: HashMap<Vec<u32>, u32>,
}

impl Interned {
    /// The index of the stack that names `functions` - each with the module
    /// it is in - innermost first.
    fn stack(&mut self, functions: &[(&str, Option<&str>)]) -> u32 {
        let ids: Vec<u32> = functions
            .iter()
            .map(|(function, module)| self.function(function, *module))
            .collect();
        if let Some(id) = self.stack_ids.get(&ids) {
            return *id;
        }

        let id = self.stacks.len() as u32;
        self.stacks.push(ids.clone());
        self.stack_ids.insert(ids, id);
        id
    }

    fn function(&mut self, name: &str, module: Option<&str>) -> u32 {
        if let Some(id) = self.function_ids.get(name) {
            return *id;
        }

        let origin = module.map(|path| self.module(path));
        let id = self.functions.len() as u32;
        self.functions.push(name.to_string());
        self.origins.push(origin);
        self.function_ids.insert(name.to_string(), id);
        id
    }

    fn module(&mut self, path: &str) -> u32 {
        if let Some(id) = self.module_ids.get(path) {
            return *id;
        }

        let id = self.modules.len() as u32;
        self.modules.push(path.to_string());
        self.module_ids.insert(path.to_string(), id);
        id
    }

    fn into_stacks(self, samples: Vec<Sample>) -> Stacks {
        Stacks {
            functions: self.functions,
            stacks: self.stacks,
            samples,
            modules: self.modules,
            origins: self.origins,
        }
    }
}

/// What runs while the UI thread is suspended, and what it reads: nothing
/// in here allocates or takes a lock.
#[cfg(target_arch = "x86_64")]
mod walk {
    use super::{DEPTH, Modules};
    use crate::bindings::{
        CONTEXT, GetThreadContext, HANDLE, IMAGE_DIRECTORY_ENTRY_EXCEPTION, IMAGE_DOS_HEADER,
        IMAGE_DOS_SIGNATURE, IMAGE_NT_HEADERS64, IMAGE_NT_OPTIONAL_HDR64_MAGIC, IMAGE_NT_SIGNATURE,
        QueryPerformanceCounter, RUNTIME_FUNCTION, ResumeThread, RtlVirtualUnwind, SuspendThread,
        UNW_FLAG_NHANDLER,
    };

    /// `CONTEXT_CONTROL | CONTEXT_INTEGER` on x86-64, which the metadata
    /// does not carry: the instruction and stack pointers and the integer
    /// registers, all that unwinding reads.
    const CONTROL_AND_INTEGER: u32 = 0x0010_0003;

    /// Suspends `target`, walks its stack into `frames` and resumes it; when
    /// the stack was read, and how deep it went. `None` once the thread is
    /// gone.
    ///
    /// # Safety
    /// `target` is a thread handle with suspend and context access, not the
    /// calling thread's.
    pub unsafe fn look(
        target: HANDLE,
        modules: &Modules,
        frames: &mut [u64; DEPTH],
    ) -> Option<(u64, usize)> {
        if unsafe { SuspendThread(target) } == u32::MAX {
            return None;
        }

        let mut context = CONTEXT {
            ContextFlags: CONTROL_AND_INTEGER,
            ..Default::default()
        };
        let read = unsafe { GetThreadContext(target, &mut context) }.as_bool();
        let mut qpc = 0i64;
        let _ = unsafe { QueryPerformanceCounter(&mut qpc) };
        let depth = if read {
            unsafe { unwind(&mut context, modules, frames) }
        } else {
            0
        };

        unsafe { ResumeThread(target) };
        Some((qpc as u64, depth))
    }

    unsafe fn unwind(context: &mut CONTEXT, modules: &Modules, frames: &mut [u64; DEPTH]) -> usize {
        let mut depth = 0;
        while depth < DEPTH {
            let pc = context.Rip;
            if pc == 0 {
                break;
            }
            frames[depth] = pc;
            depth += 1;

            let Some(module) = modules.containing(pc) else {
                break;
            };
            let below = context.Rsp;
            match unsafe { function(module.base, module.functions, module.count, pc) } {
                Some(entry) => {
                    let mut handler = std::ptr::null_mut();
                    let mut frame = 0u64;
                    unsafe {
                        RtlVirtualUnwind(
                            UNW_FLAG_NHANDLER as u32,
                            module.base,
                            pc,
                            entry,
                            context,
                            &mut handler,
                            &mut frame,
                            std::ptr::null_mut(),
                        )
                    };
                }
                None => {
                    context.Rip = unsafe { *(context.Rsp as *const u64) };
                    context.Rsp += 8;
                }
            }
            if context.Rsp <= below {
                break;
            }
        }
        depth
    }

    /// The entry of the table at `functions` that covers `pc`, following an
    /// entry that only points at another.
    unsafe fn function(
        base: u64,
        functions: usize,
        count: usize,
        pc: u64,
    ) -> Option<*const RUNTIME_FUNCTION> {
        if functions == 0 {
            return None;
        }
        let offset = u32::try_from(pc - base).ok()?;
        let table =
            unsafe { std::slice::from_raw_parts(functions as *const RUNTIME_FUNCTION, count) };
        let at = table.partition_point(|entry| entry.EndAddress <= offset);
        let entry = table.get(at).filter(|entry| entry.BeginAddress <= offset)?;

        let unwind = unsafe { entry.Anonymous.UnwindData };
        if unwind & 1 != 0 {
            return Some((base + u64::from(unwind & !1)) as *const RUNTIME_FUNCTION);
        }
        Some(entry)
    }

    /// Where the image loaded at `base` keeps its table of functions, and
    /// how many it lists.
    ///
    /// # Safety
    /// `base` is where a module of this process is loaded.
    pub unsafe fn exceptions(base: u64) -> Option<(usize, usize)> {
        let dos = unsafe { &*(base as *const IMAGE_DOS_HEADER) };
        if i32::from(dos.e_magic) != IMAGE_DOS_SIGNATURE {
            return None;
        }
        let nt = unsafe { &*((base + dos.e_lfanew as u64) as *const IMAGE_NT_HEADERS64) };
        if nt.Signature as i32 != IMAGE_NT_SIGNATURE
            || nt.OptionalHeader.Magic as i32 != IMAGE_NT_OPTIONAL_HDR64_MAGIC
        {
            return None;
        }

        let directory = nt.OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_EXCEPTION as usize];
        (directory.VirtualAddress != 0 && directory.Size != 0).then(|| {
            (
                (base + u64::from(directory.VirtualAddress)) as usize,
                directory.Size as usize / size_of::<RUNTIME_FUNCTION>(),
            )
        })
    }
}

#[cfg(not(target_arch = "x86_64"))]
mod walk {
    use super::{DEPTH, Modules};
    use crate::bindings::HANDLE;

    pub unsafe fn look(_: HANDLE, _: &Modules, _: &mut [u64; DEPTH]) -> Option<(u64, usize)> {
        None
    }

    pub unsafe fn exceptions(_: u64) -> Option<(usize, usize)> {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::time::Duration;

    use super::*;

    #[inline(never)]
    fn spin_in_a_known_place(until: &AtomicBool) -> u64 {
        let mut turns = 0u64;
        while !until.load(Ordering::Relaxed) {
            turns = std::hint::black_box(turns.wrapping_add(1));
        }
        turns
    }

    /// What the sampler keeps is one store for the whole process, so the
    /// tests that fill it take turns.
    static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

    #[test]
    fn a_busy_thread_is_seen_in_the_function_it_is_busy_in() {
        let _alone = lock(&ONE_AT_A_TIME);
        let done = Arc::new(AtomicBool::new(false));
        let id = Arc::new(AtomicU32::new(0));
        let busy = {
            let done = Arc::clone(&done);
            let id = Arc::clone(&id);
            std::thread::spawn(move || {
                id.store(
                    unsafe { crate::bindings::GetCurrentThreadId() },
                    Ordering::SeqCst,
                );
                spin_in_a_known_place(&done)
            })
        };
        while id.load(Ordering::SeqCst) == 0 {
            std::thread::yield_now();
        }

        let started = start(id.load(Ordering::SeqCst));
        std::thread::sleep(Duration::from_millis(300));
        stop();
        done.store(true, Ordering::Relaxed);
        let _ = busy.join();
        let stacks = take();

        assert_eq!(started, Ok(()));
        assert!(
            stacks.samples.len() > 50,
            "{} samples in 300 ms",
            stacks.samples.len()
        );
        let spinning = stacks
            .samples
            .iter()
            .filter(|sample| {
                stacks.stacks[sample.stack as usize].iter().any(|function| {
                    stacks.functions[*function as usize].contains("spin_in_a_known_place")
                })
            })
            .count();
        assert!(
            spinning * 2 > stacks.samples.len(),
            "{spinning} of {} samples in spin_in_a_known_place; functions seen: {:?}",
            stacks.samples.len(),
            stacks.functions
        );

        let exe = std::env::current_exe()
            .ok()
            .and_then(|path| Some(path.file_name()?.to_string_lossy().into_owned()));
        let module = stacks
            .functions
            .iter()
            .position(|function| function.contains("spin_in_a_known_place"))
            .and_then(|function| stacks.origins.get(function).copied().flatten())
            .and_then(|module| stacks.modules.get(module as usize));
        assert!(
            module.is_some_and(|path| exe.as_ref().is_some_and(|exe| path.ends_with(exe.as_str()))),
            "spin_in_a_known_place is in {module:?}, not in {exe:?}"
        );
    }

    #[test]
    fn a_name_from_the_exports_alone_says_how_far_past_the_export_it_is() {
        use crate::bindings::{GetModuleHandleW, GetProcAddress};

        let kernel = unsafe { GetModuleHandleW(windows_core::w!("kernel32.dll")) };
        let export = unsafe { GetProcAddress(kernel, windows_core::s!("GetCurrentProcessId")) }
            .map(|function| function as usize as u64);
        let mut names = Names::new();
        names.refresh();
        let address = export.map(|export| export + 2);

        let name = address.map(|address| {
            names.learn(address);
            names.name(address).0.to_string()
        });

        assert!(
            name.as_deref()
                .is_some_and(|name| name.contains(".dll!") && name.ends_with("+0x2")
                    || name.contains(".DLL!") && name.ends_with("+0x2")),
            "{name:?}"
        );
    }

    #[test]
    fn when_nobody_takes_them_the_newest_samples_are_kept() {
        let _alone = lock(&ONE_AT_A_TIME);
        take();
        for qpc in 0..KEPT as u64 + 5 {
            keep(qpc, &[0x1000]);
        }

        let stacks = take();

        assert_eq!(stacks.samples.len(), KEPT);
        assert_eq!(stacks.samples.first().map(|sample| sample.qpc), Some(5));
    }

    #[test]
    fn a_stack_seen_twice_is_listed_once() {
        let mut interned = Interned::default();

        let app = Some(r"C:\app\app.exe");
        let xaml = Some(r"C:\xaml\xaml.dll");
        let first = interned.stack(&[("main", app), ("draw", app), ("measure", xaml)]);
        let again = interned.stack(&[("main", app), ("draw", app), ("measure", xaml)]);
        let other = interned.stack(&[("main", app), ("draw", app), ("0x10", None)]);

        assert_eq!(first, again);
        assert_ne!(first, other);
        let stacks = interned.into_stacks(Vec::new());
        assert_eq!(stacks.functions, ["main", "draw", "measure", "0x10"]);
        assert_eq!(stacks.stacks, [vec![0, 1, 2], vec![0, 1, 3]]);
        assert_eq!(stacks.modules, [r"C:\app\app.exe", r"C:\xaml\xaml.dll"]);
        assert_eq!(stacks.origins, [Some(0), Some(0), Some(1), None]);
    }
}
