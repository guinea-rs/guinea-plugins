windows_core::link!("user32.dll" "system" fn CallNextHookEx(hhk : HHOOK, ncode : i32, wparam : WPARAM, lparam : LPARAM) -> LRESULT);
windows_core::link!("user32.dll" "system" fn ClientToScreen(hwnd : HWND, lppoint : *mut POINT) -> windows_core::BOOL);
windows_core::link!("kernel32.dll" "system" fn CloseHandle(hobject : HANDLE) -> windows_core::BOOL);
windows_core::link!("ole32.dll" "system" fn CoCreateInstance(rclsid : *const windows_core::GUID, punkouter : *mut core::ffi::c_void, dwclscontext : u32, riid : *const windows_core::GUID, ppv : *mut *mut core::ffi::c_void) -> windows_core::HRESULT);
windows_core::link!("ole32.dll" "system" fn CoInitializeEx(pvreserved : *const core::ffi::c_void, dwcoinit : u32) -> windows_core::HRESULT);
windows_core::link!("ole32.dll" "system" fn CoTaskMemFree(pv : *mut core::ffi::c_void));
windows_core::link!("gdi32.dll" "system" fn CreateSolidBrush(color : COLORREF) -> HBRUSH);
windows_core::link!("kernel32.dll" "system" fn CreateToolhelp32Snapshot(dwflags : u32, th32processid : u32) -> HANDLE);
windows_core::link!("kernel32.dll" "system" fn CreateWaitableTimerExW(lptimerattributes : *const SECURITY_ATTRIBUTES, lptimername : windows_core::PCWSTR, dwflags : u32, dwdesiredaccess : u32) -> HANDLE);
windows_core::link!("user32.dll" "system" fn CreateWindowExW(dwexstyle : u32, lpclassname : windows_core::PCWSTR, lpwindowname : windows_core::PCWSTR, dwstyle : u32, x : i32, y : i32, nwidth : i32, nheight : i32, hwndparent : HWND, hmenu : HMENU, hinstance : HINSTANCE, lpparam : *const core::ffi::c_void) -> HWND);
windows_core::link!("user32.dll" "system" fn DefWindowProcW(hwnd : HWND, msg : u32, wparam : WPARAM, lparam : LPARAM) -> LRESULT);
windows_core::link!("user32.dll" "system" fn DestroyWindow(hwnd : HWND) -> windows_core::BOOL);
windows_core::link!("user32.dll" "system" fn EnumThreadWindows(dwthreadid : u32, lpfn : WNDENUMPROC, lparam : LPARAM) -> windows_core::BOOL);
windows_core::link!("user32.dll" "system" fn GetAncestor(hwnd : HWND, gaflags : u32) -> HWND);
windows_core::link!("kernel32.dll" "system" fn GetCurrentProcess() -> HANDLE);
windows_core::link!("kernel32.dll" "system" fn GetCurrentThread() -> HANDLE);
windows_core::link!("kernel32.dll" "system" fn GetCurrentThreadId() -> u32);
windows_core::link!("user32.dll" "system" fn GetCursorPos(lppoint : *mut POINT) -> windows_core::BOOL);
windows_core::link!("user32.dll" "system" fn GetDpiForWindow(hwnd : HWND) -> u32);
windows_core::link!("user32.dll" "system" fn GetMessageW(lpmsg : *mut MSG, hwnd : HWND, wmsgfiltermin : u32, wmsgfiltermax : u32) -> windows_core::BOOL);
windows_core::link!("kernel32.dll" "system" fn GetModuleHandleW(lpmodulename : windows_core::PCWSTR) -> HMODULE);
windows_core::link!("kernel32.dll" "system" fn GetProcAddress(hmodule : HMODULE, lpprocname : windows_core::PCSTR) -> FARPROC);
windows_core::link!("user32.dll" "system" fn GetSystemMetrics(nindex : i32) -> i32);
windows_core::link!("kernel32.dll" "system" fn GetThreadContext(hthread : HANDLE, lpcontext : LPCONTEXT) -> windows_core::BOOL);
windows_core::link!("user32.dll" "system" fn GetWindowLongW(hwnd : HWND, nindex : i32) -> i32);
windows_core::link!("user32.dll" "system" fn GetWindowThreadProcessId(hwnd : HWND, lpdwprocessid : *mut u32) -> u32);
windows_core::link!("user32.dll" "system" fn IsWindowVisible(hwnd : HWND) -> windows_core::BOOL);
windows_core::link!("kernel32.dll" "system" fn LoadLibraryW(lplibfilename : windows_core::PCWSTR) -> HMODULE);
windows_core::link!("kernel32.dll" "system" fn Module32FirstW(hsnapshot : HANDLE, lpme : *mut MODULEENTRY32W) -> windows_core::BOOL);
windows_core::link!("kernel32.dll" "system" fn Module32NextW(hsnapshot : HANDLE, lpme : *mut MODULEENTRY32W) -> windows_core::BOOL);
windows_core::link!("kernel32.dll" "system" fn OpenThread(dwdesiredaccess : u32, binherithandle : windows_core::BOOL, dwthreadid : u32) -> HANDLE);
windows_core::link!("user32.dll" "system" fn PostThreadMessageW(idthread : u32, msg : u32, wparam : WPARAM, lparam : LPARAM) -> windows_core::BOOL);
windows_core::link!("kernel32.dll" "system" fn QueryPerformanceCounter(lpperformancecount : *mut i64) -> windows_core::BOOL);
windows_core::link!("user32.dll" "system" fn RegisterClassW(lpwndclass : *const WNDCLASSW) -> ATOM);
windows_core::link!("kernel32.dll" "system" fn ResumeThread(hthread : HANDLE) -> u32);
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
windows_core::link!("kernel32.dll" "system" fn RtlVirtualUnwind(handlertype : u32, imagebase : u64, controlpc : u64, functionentry : *const RUNTIME_FUNCTION, contextrecord : *mut CONTEXT, handlerdata : *mut *mut core::ffi::c_void, establisherframe : *mut u64, contextpointers : *mut KNONVOLATILE_CONTEXT_POINTERS) -> PEXCEPTION_ROUTINE);
#[cfg(target_arch = "aarch64")]
windows_core::link!("kernel32.dll" "system" fn RtlVirtualUnwind(handlertype : u32, imagebase : usize, controlpc : usize, functionentry : *const ARM64_RUNTIME_FUNCTION, contextrecord : *mut ARM64_NT_CONTEXT, handlerdata : *mut *mut core::ffi::c_void, establisherframe : *mut u64, contextpointers : *mut KNONVOLATILE_CONTEXT_POINTERS_ARM64) -> PEXCEPTION_ROUTINE);
windows_core::link!("user32.dll" "system" fn ScreenToClient(hwnd : HWND, lppoint : *mut POINT) -> windows_core::BOOL);
windows_core::link!("user32.dll" "system" fn SendInput(cinputs : u32, pinputs : *const INPUT, cbsize : i32) -> u32);
windows_core::link!("user32.dll" "system" fn SendMessageW(hwnd : HWND, msg : u32, wparam : WPARAM, lparam : LPARAM) -> LRESULT);
windows_core::link!("user32.dll" "system" fn SetForegroundWindow(hwnd : HWND) -> windows_core::BOOL);
windows_core::link!("user32.dll" "system" fn SetLayeredWindowAttributes(hwnd : HWND, crkey : COLORREF, balpha : u8, dwflags : u32) -> windows_core::BOOL);
windows_core::link!("kernel32.dll" "system" fn SetThreadPriority(hthread : HANDLE, npriority : i32) -> windows_core::BOOL);
windows_core::link!("kernel32.dll" "system" fn SetWaitableTimer(htimer : HANDLE, lpduetime : *const i64, lperiod : i32, pfncompletionroutine : PTIMERAPCROUTINE, lpargtocompletionroutine : *const core::ffi::c_void, fresume : windows_core::BOOL) -> windows_core::BOOL);
windows_core::link!("user32.dll" "system" fn SetWindowPos(hwnd : HWND, hwndinsertafter : HWND, x : i32, y : i32, cx : i32, cy : i32, uflags : u32) -> windows_core::BOOL);
windows_core::link!("user32.dll" "system" fn SetWindowsHookExW(idhook : i32, lpfn : HOOKPROC, hmod : HINSTANCE, dwthreadid : u32) -> HHOOK);
windows_core::link!("user32.dll" "system" fn ShowWindow(hwnd : HWND, ncmdshow : i32) -> windows_core::BOOL);
windows_core::link!("kernel32.dll" "system" fn SuspendThread(hthread : HANDLE) -> u32);
windows_core::link!("dbghelp.dll" "system" fn SymFromAddrW(hprocess : HANDLE, address : u64, displacement : *mut u64, symbol : *mut SYMBOL_INFOW) -> windows_core::BOOL);
windows_core::link!("dbghelp.dll" "system" fn SymInitializeW(hprocess : HANDLE, usersearchpath : windows_core::PCWSTR, finvadeprocess : windows_core::BOOL) -> windows_core::BOOL);
windows_core::link!("dbghelp.dll" "system" fn SymRefreshModuleList(hprocess : HANDLE) -> windows_core::BOOL);
windows_core::link!("dbghelp.dll" "system" fn SymSetOptions(symoptions : u32) -> u32);
windows_core::link!("dbghelp.dll" "system" fn SymSetSearchPathW(hprocess : HANDLE, searchpatha : windows_core::PCWSTR) -> windows_core::BOOL);
windows_core::link!("user32.dll" "system" fn UnhookWindowsHookEx(hhk : HHOOK) -> windows_core::BOOL);
windows_core::link!("kernel32.dll" "system" fn WaitForSingleObject(hhandle : HANDLE, dwmilliseconds : u32) -> u32);
windows_core::link!("user32.dll" "system" fn WindowFromPoint(point : POINT) -> HWND);
#[repr(C, align(16))]
#[cfg(any(target_arch = "arm64ec", target_arch = "x86", target_arch = "x86_64"))]
#[derive(Clone, Copy)]
pub struct ARM64_NT_CONTEXT {
    pub ContextFlags: u32,
    pub Cpsr: u32,
    pub Anonymous: ARM64_NT_CONTEXT_0,
    pub Sp: u64,
    pub Pc: u64,
    pub V: [ARM64_NT_NEON128; 32],
    pub Fpcr: u32,
    pub Fpsr: u32,
    pub Bcr: [u32; 8],
    pub Bvr: [u64; 8],
    pub Wcr: [u32; 2],
    pub Wvr: [u64; 2],
}
#[cfg(any(target_arch = "arm64ec", target_arch = "x86", target_arch = "x86_64"))]
impl Default for ARM64_NT_CONTEXT {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(any(target_arch = "arm64ec", target_arch = "x86", target_arch = "x86_64"))]
#[derive(Clone, Copy)]
pub union ARM64_NT_CONTEXT_0 {
    pub Anonymous: ARM64_NT_CONTEXT_0_0,
    pub X: [u64; 31],
}
#[cfg(any(target_arch = "arm64ec", target_arch = "x86", target_arch = "x86_64"))]
impl Default for ARM64_NT_CONTEXT_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(any(target_arch = "arm64ec", target_arch = "x86", target_arch = "x86_64"))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ARM64_NT_CONTEXT_0_0 {
    pub X0: u64,
    pub X1: u64,
    pub X2: u64,
    pub X3: u64,
    pub X4: u64,
    pub X5: u64,
    pub X6: u64,
    pub X7: u64,
    pub X8: u64,
    pub X9: u64,
    pub X10: u64,
    pub X11: u64,
    pub X12: u64,
    pub X13: u64,
    pub X14: u64,
    pub X15: u64,
    pub X16: u64,
    pub X17: u64,
    pub X18: u64,
    pub X19: u64,
    pub X20: u64,
    pub X21: u64,
    pub X22: u64,
    pub X23: u64,
    pub X24: u64,
    pub X25: u64,
    pub X26: u64,
    pub X27: u64,
    pub X28: u64,
    pub Fp: u64,
    pub Lr: u64,
}
#[repr(C, align(16))]
#[cfg(target_arch = "aarch64")]
#[derive(Clone, Copy)]
pub struct ARM64_NT_CONTEXT {
    pub ContextFlags: u32,
    pub Cpsr: u32,
    pub Anonymous: ARM64_NT_CONTEXT_0,
    pub Sp: u64,
    pub Pc: u64,
    pub V: [NEON128; 32],
    pub Fpcr: u32,
    pub Fpsr: u32,
    pub Bcr: [u32; 8],
    pub Bvr: [u64; 8],
    pub Wcr: [u32; 2],
    pub Wvr: [u64; 2],
}
#[cfg(target_arch = "aarch64")]
impl Default for ARM64_NT_CONTEXT {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(target_arch = "aarch64")]
#[derive(Clone, Copy)]
pub union ARM64_NT_CONTEXT_0 {
    pub Anonymous: ARM64_NT_CONTEXT_0_0,
    pub X: [u64; 31],
}
#[cfg(target_arch = "aarch64")]
impl Default for ARM64_NT_CONTEXT_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(target_arch = "aarch64")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ARM64_NT_CONTEXT_0_0 {
    pub X0: u64,
    pub X1: u64,
    pub X2: u64,
    pub X3: u64,
    pub X4: u64,
    pub X5: u64,
    pub X6: u64,
    pub X7: u64,
    pub X8: u64,
    pub X9: u64,
    pub X10: u64,
    pub X11: u64,
    pub X12: u64,
    pub X13: u64,
    pub X14: u64,
    pub X15: u64,
    pub X16: u64,
    pub X17: u64,
    pub X18: u64,
    pub X19: u64,
    pub X20: u64,
    pub X21: u64,
    pub X22: u64,
    pub X23: u64,
    pub X24: u64,
    pub X25: u64,
    pub X26: u64,
    pub X27: u64,
    pub X28: u64,
    pub Fp: u64,
    pub Lr: u64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union ARM64_NT_NEON128 {
    pub Anonymous: ARM64_NT_NEON128_0,
    pub D: [f64; 2],
    pub S: [f32; 4],
    pub H: [u16; 8],
    pub B: [u8; 16],
}
impl Default for ARM64_NT_NEON128 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ARM64_NT_NEON128_0 {
    pub Low: u64,
    pub High: i64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ARM64_RUNTIME_FUNCTION {
    pub BeginAddress: u32,
    pub Anonymous: ARM64_RUNTIME_FUNCTION_0,
}
impl Default for ARM64_RUNTIME_FUNCTION {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union ARM64_RUNTIME_FUNCTION_0 {
    pub UnwindData: u32,
    pub Anonymous: ARM64_RUNTIME_FUNCTION_0_0,
}
impl Default for ARM64_RUNTIME_FUNCTION_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ARM64_RUNTIME_FUNCTION_0_0 {
    pub _bitfield: u32,
}
impl ARM64_RUNTIME_FUNCTION_0_0 {
    pub fn Flag(&self) -> u32 {
        (self._bitfield << 30) >> 30
    }
    pub fn set_Flag(&mut self, value: u32) {
        self._bitfield = (self._bitfield & !3) | (value & 3);
    }
    pub fn FunctionLength(&self) -> u32 {
        (self._bitfield << 19) >> 21
    }
    pub fn set_FunctionLength(&mut self, value: u32) {
        self._bitfield = (self._bitfield & !(2047 << 2)) | ((value & 2047) << 2);
    }
    pub fn RegF(&self) -> u32 {
        (self._bitfield << 16) >> 29
    }
    pub fn set_RegF(&mut self, value: u32) {
        self._bitfield = (self._bitfield & !(7 << 13)) | ((value & 7) << 13);
    }
    pub fn RegI(&self) -> u32 {
        (self._bitfield << 12) >> 28
    }
    pub fn set_RegI(&mut self, value: u32) {
        self._bitfield = (self._bitfield & !(15 << 16)) | ((value & 15) << 16);
    }
    pub fn H(&self) -> bool {
        (self._bitfield >> 20) & 1 != 0
    }
    pub fn set_H(&mut self, value: bool) {
        self._bitfield = (self._bitfield & !(1 << 20)) | ((value as u32) << 20);
    }
    pub fn CR(&self) -> u32 {
        (self._bitfield << 9) >> 30
    }
    pub fn set_CR(&mut self, value: u32) {
        self._bitfield = (self._bitfield & !(3 << 21)) | ((value & 3) << 21);
    }
    pub fn FrameSize(&self) -> u32 {
        self._bitfield >> 23
    }
    pub fn set_FrameSize(&mut self, value: u32) {
        self._bitfield = (self._bitfield & !(511 << 23)) | ((value & 511) << 23);
    }
}
pub type ATOM = u16;
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutomationProperties(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    AutomationProperties,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl AutomationProperties {
    pub(crate) fn GetAutomationId<P0>(element: P0) -> windows_core::Result<String>
    where
        P0: windows_core::Param<DependencyObject>,
    {
        Self::IAutomationPropertiesStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).GetAutomationId)(
                windows_core::Interface::as_raw(this),
                element.param().abi(),
                &mut result__,
            )
            .map(|| {
                let hstring: windows_core::HSTRING = core::mem::transmute(result__);
                hstring.to_string_lossy()
            })
        })
    }
    fn IAutomationPropertiesStatics<
        R,
        F: FnOnce(&IAutomationPropertiesStatics) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<
            AutomationProperties,
            IAutomationPropertiesStatics,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for AutomationProperties {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IAutomationProperties>();
}
unsafe impl windows_core::Interface for AutomationProperties {
    type Vtable = <IAutomationProperties as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IAutomationProperties as windows_core::Interface>::IID;
}
impl core::ops::Deref for AutomationProperties {
    type Target = IAutomationProperties;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for AutomationProperties {
    const NAME: &'static str = "Microsoft.UI.Xaml.Automation.AutomationProperties";
}
unsafe impl Send for AutomationProperties {}
unsafe impl Sync for AutomationProperties {}
pub const CLASS_E_CLASSNOTAVAILABLE: windows_core::HRESULT =
    windows_core::HRESULT(0x80040111_u32 as _);
pub type CLSCTX = u32;
pub const CLSCTX_INPROC_SERVER: CLSCTX = 1;
pub type COINIT = i32;
pub const COINIT_MULTITHREADED: COINIT = 0;
pub type COLORREF = u32;
#[repr(C)]
#[cfg(target_arch = "x86")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CONTEXT {
    pub ContextFlags: u32,
    pub Dr0: u32,
    pub Dr1: u32,
    pub Dr2: u32,
    pub Dr3: u32,
    pub Dr6: u32,
    pub Dr7: u32,
    pub FloatSave: FLOATING_SAVE_AREA,
    pub SegGs: u32,
    pub SegFs: u32,
    pub SegEs: u32,
    pub SegDs: u32,
    pub Edi: u32,
    pub Esi: u32,
    pub Ebx: u32,
    pub Edx: u32,
    pub Ecx: u32,
    pub Eax: u32,
    pub Ebp: u32,
    pub Eip: u32,
    pub SegCs: u32,
    pub EFlags: u32,
    pub Esp: u32,
    pub SegSs: u32,
    pub ExtendedRegisters: [u8; 512],
}
#[cfg(target_arch = "x86")]
impl Default for CONTEXT {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
#[derive(Clone, Copy)]
pub struct CONTEXT {
    pub P1Home: u64,
    pub P2Home: u64,
    pub P3Home: u64,
    pub P4Home: u64,
    pub P5Home: u64,
    pub P6Home: u64,
    pub ContextFlags: u32,
    pub MxCsr: u32,
    pub SegCs: u16,
    pub SegDs: u16,
    pub SegEs: u16,
    pub SegFs: u16,
    pub SegGs: u16,
    pub SegSs: u16,
    pub EFlags: u32,
    pub Dr0: u64,
    pub Dr1: u64,
    pub Dr2: u64,
    pub Dr3: u64,
    pub Dr6: u64,
    pub Dr7: u64,
    pub Rax: u64,
    pub Rcx: u64,
    pub Rdx: u64,
    pub Rbx: u64,
    pub Rsp: u64,
    pub Rbp: u64,
    pub Rsi: u64,
    pub Rdi: u64,
    pub R8: u64,
    pub R9: u64,
    pub R10: u64,
    pub R11: u64,
    pub R12: u64,
    pub R13: u64,
    pub R14: u64,
    pub R15: u64,
    pub Rip: u64,
    pub Anonymous: CONTEXT_0,
    pub VectorRegister: [M128A; 26],
    pub VectorControl: u64,
    pub DebugControl: u64,
    pub LastBranchToRip: u64,
    pub LastBranchFromRip: u64,
    pub LastExceptionToRip: u64,
    pub LastExceptionFromRip: u64,
}
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
impl Default for CONTEXT {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
#[derive(Clone, Copy)]
pub union CONTEXT_0 {
    pub FltSave: XMM_SAVE_AREA32,
    pub Anonymous: CONTEXT_0_0,
}
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
impl Default for CONTEXT_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CONTEXT_0_0 {
    pub Header: [M128A; 2],
    pub Legacy: [M128A; 8],
    pub Xmm0: M128A,
    pub Xmm1: M128A,
    pub Xmm2: M128A,
    pub Xmm3: M128A,
    pub Xmm4: M128A,
    pub Xmm5: M128A,
    pub Xmm6: M128A,
    pub Xmm7: M128A,
    pub Xmm8: M128A,
    pub Xmm9: M128A,
    pub Xmm10: M128A,
    pub Xmm11: M128A,
    pub Xmm12: M128A,
    pub Xmm13: M128A,
    pub Xmm14: M128A,
    pub Xmm15: M128A,
}
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
impl Default for CONTEXT_0_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[cfg(target_arch = "aarch64")]
pub type CONTEXT = ARM64_NT_CONTEXT;
pub type CONTROLTYPEID = i32;
pub const CREATE_WAITABLE_TIMER_HIGH_RESOLUTION: i32 = 2;
pub const CUIAutomation: windows_core::GUID =
    windows_core::GUID::from_u128(0xff48dba4_60ef_4201_aa87_54103eef594e);
#[repr(C)]
#[derive(Clone, Copy)]
pub union CY {
    pub Anonymous: CY_0,
    pub int64: i64,
}
impl Default for CY {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CY_0 {
    pub Lo: u32,
    pub Hi: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DECIMAL {
    pub wReserved: u16,
    pub Anonymous: DECIMAL_0,
    pub Hi32: u32,
    pub Anonymous2: DECIMAL_1,
}
impl Default for DECIMAL {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union DECIMAL_0 {
    pub Anonymous: DECIMAL_0_0,
    pub signscale: u16,
}
impl Default for DECIMAL_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DECIMAL_0_0 {
    pub scale: u8,
    pub sign: u8,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union DECIMAL_1 {
    pub Anonymous: DECIMAL_1_0,
    pub Lo64: u64,
}
impl Default for DECIMAL_1 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DECIMAL_1_0 {
    pub Lo32: u32,
    pub Mid32: u32,
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyObject(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    DependencyObject,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for DependencyObject {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IDependencyObject>();
}
unsafe impl windows_core::Interface for DependencyObject {
    type Vtable = <IDependencyObject as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IDependencyObject as windows_core::Interface>::IID;
}
impl core::ops::Deref for DependencyObject {
    type Target = IDependencyObject;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for DependencyObject {
    const NAME: &'static str = "Microsoft.UI.Xaml.DependencyObject";
}
unsafe impl Send for DependencyObject {}
unsafe impl Sync for DependencyObject {}
pub type EVENTID = i32;
pub type EXCEPTION_DISPOSITION = i32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EXCEPTION_RECORD {
    pub ExceptionCode: u32,
    pub ExceptionFlags: u32,
    pub ExceptionRecord: *mut Self,
    pub ExceptionAddress: *mut core::ffi::c_void,
    pub NumberParameters: u32,
    pub ExceptionInformation: [usize; 15],
}
impl Default for EXCEPTION_RECORD {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub const E_NOINTERFACE: windows_core::HRESULT = windows_core::HRESULT(0x80004002_u32 as _);
pub type FARPROC = Option<unsafe extern "system" fn() -> isize>;
#[repr(C)]
#[cfg(target_arch = "x86")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FLOATING_SAVE_AREA {
    pub ControlWord: u32,
    pub StatusWord: u32,
    pub TagWord: u32,
    pub ErrorOffset: u32,
    pub ErrorSelector: u32,
    pub DataOffset: u32,
    pub DataSelector: u32,
    pub RegisterArea: [u8; 80],
    pub Spare0: u32,
}
#[cfg(target_arch = "x86")]
impl Default for FLOATING_SAVE_AREA {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub const GA_ROOT: i32 = 2;
pub const GWL_EXSTYLE: i32 = -20;
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneralTransform(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    GeneralTransform,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(GeneralTransform, DependencyObject);
impl windows_core::RuntimeType for GeneralTransform {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IGeneralTransform>();
}
unsafe impl windows_core::Interface for GeneralTransform {
    type Vtable = <IGeneralTransform as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IGeneralTransform as windows_core::Interface>::IID;
}
impl core::ops::Deref for GeneralTransform {
    type Target = IGeneralTransform;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for GeneralTransform {
    const NAME: &'static str = "Microsoft.UI.Xaml.Media.GeneralTransform";
}
unsafe impl Send for GeneralTransform {}
unsafe impl Sync for GeneralTransform {}
pub type HANDLE = *mut core::ffi::c_void;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HARDWAREINPUT {
    pub uMsg: u32,
    pub wParamL: u16,
    pub wParamH: u16,
}
pub type HBRUSH = *mut core::ffi::c_void;
pub type HCURSOR = HICON;
pub type HHOOK = *mut core::ffi::c_void;
pub type HICON = *mut core::ffi::c_void;
pub type HINSTANCE = *mut core::ffi::c_void;
pub type HMENU = *mut core::ffi::c_void;
pub type HMODULE = HINSTANCE;
pub type HOOKPROC =
    Option<unsafe extern "system" fn(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT>;
pub type HWND = *mut core::ffi::c_void;
pub const HWND_MESSAGE: HWND = -3 as _;
pub const HWND_TOPMOST: HWND = -1 as _;
windows_core::imp::define_interface!(
    IAccessible,
    IAccessible_Vtbl,
    0x618736e0_3c3d_11cf_810c_00aa00389b71
);
impl core::ops::Deref for IAccessible {
    type Target = IDispatch;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(IAccessible, windows_core::IUnknown, IDispatch);
#[repr(C)]
pub struct IAccessible_Vtbl {
    pub base__: IDispatch_Vtbl,
    accParent: usize,
    accChildCount: usize,
    accChild: usize,
    accName: usize,
    accValue: usize,
    accDescription: usize,
    accRole: usize,
    accState: usize,
    accHelp: usize,
    accHelpTopic: usize,
    accKeyboardShortcut: usize,
    accFocus: usize,
    accSelection: usize,
    accDefaultAction: usize,
    accSelect: usize,
    accLocation: usize,
    accNavigate: usize,
    accHitTest: usize,
    accDoDefaultAction: usize,
    SetaccName: usize,
    SetaccValue: usize,
}
impl windows_core::RuntimeName for IAccessible {}
windows_core::imp::define_interface!(
    IAutomationProperties,
    IAutomationProperties_Vtbl,
    0x525c6a71_dd8a_52a0_977b_db1b02f8e896
);
impl windows_core::RuntimeType for IAutomationProperties {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IAutomationProperties_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IAutomationPropertiesStatics,
    IAutomationPropertiesStatics_Vtbl,
    0xb1e3e0f3_112f_5966_87dc_7862d4ad50e5
);
impl windows_core::RuntimeType for IAutomationPropertiesStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IAutomationPropertiesStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    AcceleratorKeyProperty: usize,
    GetAcceleratorKey: usize,
    SetAcceleratorKey: usize,
    AccessKeyProperty: usize,
    GetAccessKey: usize,
    SetAccessKey: usize,
    AutomationIdProperty: usize,
    pub GetAutomationId: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IClassFactory,
    IClassFactory_Vtbl,
    0x00000001_0000_0000_c000_000000000046
);
windows_core::imp::interface_hierarchy!(IClassFactory, windows_core::IUnknown);
impl IClassFactory {
    pub(crate) unsafe fn CreateInstance<P0, T>(&self, punkouter: P0) -> windows_core::Result<T>
    where
        P0: windows_core::Param<windows_core::IUnknown>,
        T: windows_core::Interface,
    {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (windows_core::Interface::vtable(self).CreateInstance)(
                windows_core::Interface::as_raw(self),
                punkouter.param().abi(),
                &T::IID,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn LockServer(&self, flock: bool) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).LockServer)(
                windows_core::Interface::as_raw(self),
                flock.into(),
            )
        }
    }
}
#[repr(C)]
pub struct IClassFactory_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub CreateInstance: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub LockServer: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::BOOL,
    ) -> windows_core::HRESULT,
}
pub trait IClassFactory_Impl: windows_core::IUnknownImpl {
    fn CreateInstance(
        &self,
        punkouter: windows_core::Ref<windows_core::IUnknown>,
        riid: *const windows_core::GUID,
        ppvobject: *mut *mut core::ffi::c_void,
    ) -> windows_core::Result<()>;
    fn LockServer(&self, flock: windows_core::BOOL) -> windows_core::Result<()>;
}
impl IClassFactory_Vtbl {
    pub const fn new<Identity: IClassFactory_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn CreateInstance<
            Identity: IClassFactory_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            punkouter: *mut core::ffi::c_void,
            riid: *const windows_core::GUID,
            ppvobject: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IClassFactory_Impl::CreateInstance(
                    this,
                    core::mem::transmute_copy(&punkouter),
                    core::mem::transmute_copy(&riid),
                    core::mem::transmute_copy(&ppvobject),
                )
                .into()
            }
        }
        unsafe extern "system" fn LockServer<Identity: IClassFactory_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            flock: windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IClassFactory_Impl::LockServer(this, core::mem::transmute_copy(&flock)).into()
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            CreateInstance: CreateInstance::<Identity, OFFSET>,
            LockServer: LockServer::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IClassFactory as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IClassFactory {}
windows_core::imp::define_interface!(
    IDependencyObject,
    IDependencyObject_Vtbl,
    0xe7beaee7_160e_50f7_8789_d63463f979fa
);
impl windows_core::RuntimeType for IDependencyObject {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IDependencyObject_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IDispatch,
    IDispatch_Vtbl,
    0x00020400_0000_0000_c000_000000000046
);
windows_core::imp::interface_hierarchy!(IDispatch, windows_core::IUnknown);
#[repr(C)]
pub struct IDispatch_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetTypeInfoCount: usize,
    GetTypeInfo: usize,
    GetIDsOfNames: usize,
    Invoke: usize,
}
impl windows_core::RuntimeName for IDispatch {}
windows_core::imp::define_interface!(
    IGeneralTransform,
    IGeneralTransform_Vtbl,
    0x04eedeeb_31e5_54c0_ae3f_8bd06645d339
);
impl windows_core::RuntimeType for IGeneralTransform {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IGeneralTransform {
    pub(crate) fn TransformPoint(&self, point: Point) -> windows_core::Result<Point> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TransformPoint)(
                windows_core::Interface::as_raw(self),
                point,
                &mut result__,
            )
            .map(|| result__)
        }
    }
}
#[repr(C)]
pub struct IGeneralTransform_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    Inverse: usize,
    pub TransformPoint: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        Point,
        *mut Point,
    ) -> windows_core::HRESULT,
}
pub type IMAGE_ARM64_RUNTIME_FUNCTION_ENTRY = ARM64_RUNTIME_FUNCTION;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IMAGE_DATA_DIRECTORY {
    pub VirtualAddress: u32,
    pub Size: u32,
}
pub const IMAGE_DIRECTORY_ENTRY_EXCEPTION: i32 = 3;
#[repr(C, packed(2))]
#[derive(Clone, Copy)]
pub struct IMAGE_DOS_HEADER {
    pub e_magic: u16,
    pub e_cblp: u16,
    pub e_cp: u16,
    pub e_crlc: u16,
    pub e_cparhdr: u16,
    pub e_minalloc: u16,
    pub e_maxalloc: u16,
    pub e_ss: u16,
    pub e_sp: u16,
    pub e_csum: u16,
    pub e_ip: u16,
    pub e_cs: u16,
    pub e_lfarlc: u16,
    pub e_ovno: u16,
    pub e_res: [u16; 4],
    pub e_oemid: u16,
    pub e_oeminfo: u16,
    pub e_res2: [u16; 10],
    pub e_lfanew: i32,
}
impl Default for IMAGE_DOS_HEADER {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub const IMAGE_DOS_SIGNATURE: i32 = 23117;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IMAGE_FILE_HEADER {
    pub Machine: u16,
    pub NumberOfSections: u16,
    pub TimeDateStamp: u32,
    pub PointerToSymbolTable: u32,
    pub NumberOfSymbols: u32,
    pub SizeOfOptionalHeader: u16,
    pub Characteristics: u16,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct IMAGE_NT_HEADERS64 {
    pub Signature: u32,
    pub FileHeader: IMAGE_FILE_HEADER,
    pub OptionalHeader: IMAGE_OPTIONAL_HEADER64,
}
pub const IMAGE_NT_OPTIONAL_HDR64_MAGIC: i32 = 523;
pub const IMAGE_NT_SIGNATURE: i32 = 17744;
#[repr(C, packed(4))]
#[derive(Clone, Copy)]
pub struct IMAGE_OPTIONAL_HEADER64 {
    pub Magic: u16,
    pub MajorLinkerVersion: u8,
    pub MinorLinkerVersion: u8,
    pub SizeOfCode: u32,
    pub SizeOfInitializedData: u32,
    pub SizeOfUninitializedData: u32,
    pub AddressOfEntryPoint: u32,
    pub BaseOfCode: u32,
    pub ImageBase: u64,
    pub SectionAlignment: u32,
    pub FileAlignment: u32,
    pub MajorOperatingSystemVersion: u16,
    pub MinorOperatingSystemVersion: u16,
    pub MajorImageVersion: u16,
    pub MinorImageVersion: u16,
    pub MajorSubsystemVersion: u16,
    pub MinorSubsystemVersion: u16,
    pub Win32VersionValue: u32,
    pub SizeOfImage: u32,
    pub SizeOfHeaders: u32,
    pub CheckSum: u32,
    pub Subsystem: u16,
    pub DllCharacteristics: u16,
    pub SizeOfStackReserve: u64,
    pub SizeOfStackCommit: u64,
    pub SizeOfHeapReserve: u64,
    pub SizeOfHeapCommit: u64,
    pub LoaderFlags: u32,
    pub NumberOfRvaAndSizes: u32,
    pub DataDirectory: [IMAGE_DATA_DIRECTORY; 16],
}
impl Default for IMAGE_OPTIONAL_HEADER64 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[cfg(any(target_arch = "arm64ec", target_arch = "x86", target_arch = "x86_64"))]
pub type IMAGE_RUNTIME_FUNCTION_ENTRY = _IMAGE_RUNTIME_FUNCTION_ENTRY;
#[cfg(target_arch = "aarch64")]
pub type IMAGE_RUNTIME_FUNCTION_ENTRY = IMAGE_ARM64_RUNTIME_FUNCTION_ENTRY;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct INPUT {
    pub r#type: u32,
    pub Anonymous: INPUT_0,
}
impl Default for INPUT {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union INPUT_0 {
    pub mi: MOUSEINPUT,
    pub ki: KEYBDINPUT,
    pub hi: HARDWAREINPUT,
}
impl Default for INPUT_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub const INPUT_KEYBOARD: i32 = 1;
pub const INPUT_MOUSE: i32 = 0;
windows_core::imp::define_interface!(
    IObjectWithSite,
    IObjectWithSite_Vtbl,
    0xfc4801a3_2ba9_11cf_a229_00aa003d7352
);
windows_core::imp::interface_hierarchy!(IObjectWithSite, windows_core::IUnknown);
impl IObjectWithSite {
    pub(crate) unsafe fn SetSite<P0>(&self, punksite: P0) -> windows_core::HRESULT
    where
        P0: windows_core::Param<windows_core::IUnknown>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetSite)(
                windows_core::Interface::as_raw(self),
                punksite.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn GetSite<T>(&self) -> windows_core::Result<T>
    where
        T: windows_core::Interface,
    {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (windows_core::Interface::vtable(self).GetSite)(
                windows_core::Interface::as_raw(self),
                &T::IID,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IObjectWithSite_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub SetSite: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetSite: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
pub trait IObjectWithSite_Impl: windows_core::IUnknownImpl {
    fn SetSite(
        &self,
        punksite: windows_core::Ref<windows_core::IUnknown>,
    ) -> windows_core::Result<()>;
    fn GetSite(
        &self,
        riid: *const windows_core::GUID,
        ppvsite: *mut *mut core::ffi::c_void,
    ) -> windows_core::Result<()>;
}
impl IObjectWithSite_Vtbl {
    pub const fn new<Identity: IObjectWithSite_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn SetSite<Identity: IObjectWithSite_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            punksite: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IObjectWithSite_Impl::SetSite(this, core::mem::transmute_copy(&punksite)).into()
            }
        }
        unsafe extern "system" fn GetSite<Identity: IObjectWithSite_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            riid: *const windows_core::GUID,
            ppvsite: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IObjectWithSite_Impl::GetSite(
                    this,
                    core::mem::transmute_copy(&riid),
                    core::mem::transmute_copy(&ppvsite),
                )
                .into()
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            SetSite: SetSite::<Identity, OFFSET>,
            GetSite: GetSite::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IObjectWithSite as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IObjectWithSite {}
windows_core::imp::define_interface!(
    IRecordInfo,
    IRecordInfo_Vtbl,
    0x0000002f_0000_0000_c000_000000000046
);
windows_core::imp::interface_hierarchy!(IRecordInfo, windows_core::IUnknown);
#[repr(C)]
pub struct IRecordInfo_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    RecordInit: usize,
    RecordClear: usize,
    RecordCopy: usize,
    GetGuid: usize,
    GetName: usize,
    GetSize: usize,
    GetTypeInfo: usize,
    GetField: usize,
    GetFieldNoCopy: usize,
    PutField: usize,
    PutFieldNoCopy: usize,
    GetFieldNames: usize,
    IsMatchingType: usize,
    RecordCreate: usize,
    RecordCreateCopy: usize,
    RecordDestroy: usize,
}
impl windows_core::RuntimeName for IRecordInfo {}
windows_core::imp::define_interface!(
    IUIAutomation,
    IUIAutomation_Vtbl,
    0x30cbe57d_d9d0_452a_ab13_7ac5ac4825ee
);
windows_core::imp::interface_hierarchy!(IUIAutomation, windows_core::IUnknown);
impl IUIAutomation {
    pub(crate) unsafe fn CompareElements<P0, P1>(
        &self,
        el1: P0,
        el2: P1,
    ) -> windows_core::Result<windows_core::BOOL>
    where
        P0: windows_core::Param<IUIAutomationElement>,
        P1: windows_core::Param<IUIAutomationElement>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CompareElements)(
                windows_core::Interface::as_raw(self),
                el1.param().abi(),
                el2.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CompareRuntimeIds(
        &self,
        runtimeid1: *const SAFEARRAY,
        runtimeid2: *const SAFEARRAY,
    ) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CompareRuntimeIds)(
                windows_core::Interface::as_raw(self),
                runtimeid1,
                runtimeid2,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn GetRootElement(&self) -> windows_core::Result<IUIAutomationElement> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetRootElement)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn ElementFromHandle(
        &self,
        hwnd: UIA_HWND,
    ) -> windows_core::Result<IUIAutomationElement> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ElementFromHandle)(
                windows_core::Interface::as_raw(self),
                hwnd,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn ElementFromPoint(
        &self,
        pt: POINT,
    ) -> windows_core::Result<IUIAutomationElement> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ElementFromPoint)(
                windows_core::Interface::as_raw(self),
                pt,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetFocusedElement(&self) -> windows_core::Result<IUIAutomationElement> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetFocusedElement)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetRootElementBuildCache<P0>(
        &self,
        cacherequest: P0,
    ) -> windows_core::Result<IUIAutomationElement>
    where
        P0: windows_core::Param<IUIAutomationCacheRequest>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetRootElementBuildCache)(
                windows_core::Interface::as_raw(self),
                cacherequest.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn ElementFromHandleBuildCache<P1>(
        &self,
        hwnd: UIA_HWND,
        cacherequest: P1,
    ) -> windows_core::Result<IUIAutomationElement>
    where
        P1: windows_core::Param<IUIAutomationCacheRequest>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ElementFromHandleBuildCache)(
                windows_core::Interface::as_raw(self),
                hwnd,
                cacherequest.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn ElementFromPointBuildCache<P1>(
        &self,
        pt: POINT,
        cacherequest: P1,
    ) -> windows_core::Result<IUIAutomationElement>
    where
        P1: windows_core::Param<IUIAutomationCacheRequest>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ElementFromPointBuildCache)(
                windows_core::Interface::as_raw(self),
                pt,
                cacherequest.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetFocusedElementBuildCache<P0>(
        &self,
        cacherequest: P0,
    ) -> windows_core::Result<IUIAutomationElement>
    where
        P0: windows_core::Param<IUIAutomationCacheRequest>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetFocusedElementBuildCache)(
                windows_core::Interface::as_raw(self),
                cacherequest.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateTreeWalker<P0>(
        &self,
        pcondition: P0,
    ) -> windows_core::Result<IUIAutomationTreeWalker>
    where
        P0: windows_core::Param<IUIAutomationCondition>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateTreeWalker)(
                windows_core::Interface::as_raw(self),
                pcondition.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn ControlViewWalker(&self) -> windows_core::Result<IUIAutomationTreeWalker> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ControlViewWalker)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn ContentViewWalker(&self) -> windows_core::Result<IUIAutomationTreeWalker> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ContentViewWalker)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn RawViewWalker(&self) -> windows_core::Result<IUIAutomationTreeWalker> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).RawViewWalker)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn RawViewCondition(&self) -> windows_core::Result<IUIAutomationCondition> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).RawViewCondition)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn ControlViewCondition(
        &self,
    ) -> windows_core::Result<IUIAutomationCondition> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ControlViewCondition)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn ContentViewCondition(
        &self,
    ) -> windows_core::Result<IUIAutomationCondition> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ContentViewCondition)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateCacheRequest(
        &self,
    ) -> windows_core::Result<IUIAutomationCacheRequest> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateCacheRequest)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateTrueCondition(
        &self,
    ) -> windows_core::Result<IUIAutomationCondition> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateTrueCondition)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateFalseCondition(
        &self,
    ) -> windows_core::Result<IUIAutomationCondition> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateFalseCondition)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreatePropertyCondition(
        &self,
        propertyid: PROPERTYID,
        value: &VARIANT,
    ) -> windows_core::Result<IUIAutomationCondition> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreatePropertyCondition)(
                windows_core::Interface::as_raw(self),
                propertyid,
                core::mem::transmute_copy(value),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreatePropertyConditionEx(
        &self,
        propertyid: PROPERTYID,
        value: &VARIANT,
        flags: PropertyConditionFlags,
    ) -> windows_core::Result<IUIAutomationCondition> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreatePropertyConditionEx)(
                windows_core::Interface::as_raw(self),
                propertyid,
                core::mem::transmute_copy(value),
                flags,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateAndCondition<P0, P1>(
        &self,
        condition1: P0,
        condition2: P1,
    ) -> windows_core::Result<IUIAutomationCondition>
    where
        P0: windows_core::Param<IUIAutomationCondition>,
        P1: windows_core::Param<IUIAutomationCondition>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateAndCondition)(
                windows_core::Interface::as_raw(self),
                condition1.param().abi(),
                condition2.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateAndConditionFromArray(
        &self,
        conditions: *const SAFEARRAY,
    ) -> windows_core::Result<IUIAutomationCondition> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateAndConditionFromArray)(
                windows_core::Interface::as_raw(self),
                conditions,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateAndConditionFromNativeArray(
        &self,
        conditions: *const Option<IUIAutomationCondition>,
        conditioncount: i32,
    ) -> windows_core::Result<IUIAutomationCondition> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateAndConditionFromNativeArray)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute(conditions),
                conditioncount,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateOrCondition<P0, P1>(
        &self,
        condition1: P0,
        condition2: P1,
    ) -> windows_core::Result<IUIAutomationCondition>
    where
        P0: windows_core::Param<IUIAutomationCondition>,
        P1: windows_core::Param<IUIAutomationCondition>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateOrCondition)(
                windows_core::Interface::as_raw(self),
                condition1.param().abi(),
                condition2.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateOrConditionFromArray(
        &self,
        conditions: *const SAFEARRAY,
    ) -> windows_core::Result<IUIAutomationCondition> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateOrConditionFromArray)(
                windows_core::Interface::as_raw(self),
                conditions,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateOrConditionFromNativeArray(
        &self,
        conditions: *const Option<IUIAutomationCondition>,
        conditioncount: i32,
    ) -> windows_core::Result<IUIAutomationCondition> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateOrConditionFromNativeArray)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute(conditions),
                conditioncount,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateNotCondition<P0>(
        &self,
        condition: P0,
    ) -> windows_core::Result<IUIAutomationCondition>
    where
        P0: windows_core::Param<IUIAutomationCondition>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateNotCondition)(
                windows_core::Interface::as_raw(self),
                condition.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn AddAutomationEventHandler<P1, P3, P4>(
        &self,
        eventid: EVENTID,
        element: P1,
        scope: TreeScope,
        cacherequest: P3,
        handler: P4,
    ) -> windows_core::HRESULT
    where
        P1: windows_core::Param<IUIAutomationElement>,
        P3: windows_core::Param<IUIAutomationCacheRequest>,
        P4: windows_core::Param<IUIAutomationEventHandler>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).AddAutomationEventHandler)(
                windows_core::Interface::as_raw(self),
                eventid,
                element.param().abi(),
                scope,
                cacherequest.param().abi(),
                handler.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn RemoveAutomationEventHandler<P1, P2>(
        &self,
        eventid: EVENTID,
        element: P1,
        handler: P2,
    ) -> windows_core::HRESULT
    where
        P1: windows_core::Param<IUIAutomationElement>,
        P2: windows_core::Param<IUIAutomationEventHandler>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).RemoveAutomationEventHandler)(
                windows_core::Interface::as_raw(self),
                eventid,
                element.param().abi(),
                handler.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn AddPropertyChangedEventHandlerNativeArray<P0, P2, P3>(
        &self,
        element: P0,
        scope: TreeScope,
        cacherequest: P2,
        handler: P3,
        propertyarray: *const PROPERTYID,
        propertycount: i32,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IUIAutomationElement>,
        P2: windows_core::Param<IUIAutomationCacheRequest>,
        P3: windows_core::Param<IUIAutomationPropertyChangedEventHandler>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).AddPropertyChangedEventHandlerNativeArray)(
                windows_core::Interface::as_raw(self),
                element.param().abi(),
                scope,
                cacherequest.param().abi(),
                handler.param().abi(),
                propertyarray,
                propertycount,
            )
        }
    }
    pub(crate) unsafe fn AddPropertyChangedEventHandler<P0, P2, P3>(
        &self,
        element: P0,
        scope: TreeScope,
        cacherequest: P2,
        handler: P3,
        propertyarray: *const SAFEARRAY,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IUIAutomationElement>,
        P2: windows_core::Param<IUIAutomationCacheRequest>,
        P3: windows_core::Param<IUIAutomationPropertyChangedEventHandler>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).AddPropertyChangedEventHandler)(
                windows_core::Interface::as_raw(self),
                element.param().abi(),
                scope,
                cacherequest.param().abi(),
                handler.param().abi(),
                propertyarray,
            )
        }
    }
    pub(crate) unsafe fn RemovePropertyChangedEventHandler<P0, P1>(
        &self,
        element: P0,
        handler: P1,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IUIAutomationElement>,
        P1: windows_core::Param<IUIAutomationPropertyChangedEventHandler>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).RemovePropertyChangedEventHandler)(
                windows_core::Interface::as_raw(self),
                element.param().abi(),
                handler.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn AddStructureChangedEventHandler<P0, P2, P3>(
        &self,
        element: P0,
        scope: TreeScope,
        cacherequest: P2,
        handler: P3,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IUIAutomationElement>,
        P2: windows_core::Param<IUIAutomationCacheRequest>,
        P3: windows_core::Param<IUIAutomationStructureChangedEventHandler>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).AddStructureChangedEventHandler)(
                windows_core::Interface::as_raw(self),
                element.param().abi(),
                scope,
                cacherequest.param().abi(),
                handler.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn RemoveStructureChangedEventHandler<P0, P1>(
        &self,
        element: P0,
        handler: P1,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IUIAutomationElement>,
        P1: windows_core::Param<IUIAutomationStructureChangedEventHandler>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).RemoveStructureChangedEventHandler)(
                windows_core::Interface::as_raw(self),
                element.param().abi(),
                handler.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn AddFocusChangedEventHandler<P0, P1>(
        &self,
        cacherequest: P0,
        handler: P1,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IUIAutomationCacheRequest>,
        P1: windows_core::Param<IUIAutomationFocusChangedEventHandler>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).AddFocusChangedEventHandler)(
                windows_core::Interface::as_raw(self),
                cacherequest.param().abi(),
                handler.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn RemoveFocusChangedEventHandler<P0>(
        &self,
        handler: P0,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IUIAutomationFocusChangedEventHandler>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).RemoveFocusChangedEventHandler)(
                windows_core::Interface::as_raw(self),
                handler.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn RemoveAllEventHandlers(&self) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).RemoveAllEventHandlers)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub(crate) unsafe fn IntNativeArrayToSafeArray(
        &self,
        array: *const i32,
        arraycount: i32,
    ) -> windows_core::Result<*mut SAFEARRAY> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IntNativeArrayToSafeArray)(
                windows_core::Interface::as_raw(self),
                array,
                arraycount,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn IntSafeArrayToNativeArray(
        &self,
        intarray: *const SAFEARRAY,
        array: *mut *mut i32,
    ) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IntSafeArrayToNativeArray)(
                windows_core::Interface::as_raw(self),
                intarray,
                array as _,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn RectToVariant(&self, rc: RECT) -> windows_core::Result<VARIANT> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).RectToVariant)(
                windows_core::Interface::as_raw(self),
                rc,
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn VariantToRect(&self, var: &VARIANT) -> windows_core::Result<RECT> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).VariantToRect)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(var),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn SafeArrayToRectNativeArray(
        &self,
        rects: *const SAFEARRAY,
        rectarray: *mut *mut RECT,
    ) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).SafeArrayToRectNativeArray)(
                windows_core::Interface::as_raw(self),
                rects,
                rectarray as _,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CreateProxyFactoryEntry<P0>(
        &self,
        factory: P0,
    ) -> windows_core::Result<IUIAutomationProxyFactoryEntry>
    where
        P0: windows_core::Param<IUIAutomationProxyFactory>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateProxyFactoryEntry)(
                windows_core::Interface::as_raw(self),
                factory.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn ProxyFactoryMapping(
        &self,
    ) -> windows_core::Result<IUIAutomationProxyFactoryMapping> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ProxyFactoryMapping)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetPropertyProgrammaticName(
        &self,
        property: PROPERTYID,
    ) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetPropertyProgrammaticName)(
                windows_core::Interface::as_raw(self),
                property,
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn GetPatternProgrammaticName(
        &self,
        pattern: PATTERNID,
    ) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetPatternProgrammaticName)(
                windows_core::Interface::as_raw(self),
                pattern,
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn PollForPotentialSupportedPatterns<P0>(
        &self,
        pelement: P0,
        patternids: *mut *mut SAFEARRAY,
        patternnames: *mut *mut SAFEARRAY,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IUIAutomationElement>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).PollForPotentialSupportedPatterns)(
                windows_core::Interface::as_raw(self),
                pelement.param().abi(),
                patternids as _,
                patternnames as _,
            )
        }
    }
    pub(crate) unsafe fn PollForPotentialSupportedProperties<P0>(
        &self,
        pelement: P0,
        propertyids: *mut *mut SAFEARRAY,
        propertynames: *mut *mut SAFEARRAY,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IUIAutomationElement>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).PollForPotentialSupportedProperties)(
                windows_core::Interface::as_raw(self),
                pelement.param().abi(),
                propertyids as _,
                propertynames as _,
            )
        }
    }
    pub(crate) unsafe fn CheckNotSupported(
        &self,
        value: &VARIANT,
    ) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CheckNotSupported)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(value),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn ReservedNotSupportedValue(
        &self,
    ) -> windows_core::Result<windows_core::IUnknown> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ReservedNotSupportedValue)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn ReservedMixedAttributeValue(
        &self,
    ) -> windows_core::Result<windows_core::IUnknown> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ReservedMixedAttributeValue)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn ElementFromIAccessible<P0>(
        &self,
        accessible: P0,
        childid: i32,
    ) -> windows_core::Result<IUIAutomationElement>
    where
        P0: windows_core::Param<IAccessible>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ElementFromIAccessible)(
                windows_core::Interface::as_raw(self),
                accessible.param().abi(),
                childid,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn ElementFromIAccessibleBuildCache<P0, P2>(
        &self,
        accessible: P0,
        childid: i32,
        cacherequest: P2,
    ) -> windows_core::Result<IUIAutomationElement>
    where
        P0: windows_core::Param<IAccessible>,
        P2: windows_core::Param<IUIAutomationCacheRequest>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ElementFromIAccessibleBuildCache)(
                windows_core::Interface::as_raw(self),
                accessible.param().abi(),
                childid,
                cacherequest.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IUIAutomation_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub CompareElements: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CompareRuntimeIds: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const SAFEARRAY,
        *const SAFEARRAY,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub GetRootElement: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ElementFromHandle: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        UIA_HWND,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ElementFromPoint: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        POINT,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetFocusedElement: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetRootElementBuildCache: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ElementFromHandleBuildCache: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        UIA_HWND,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ElementFromPointBuildCache: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        POINT,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetFocusedElementBuildCache: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateTreeWalker: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ControlViewWalker: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ContentViewWalker: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub RawViewWalker: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub RawViewCondition: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ControlViewCondition: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ContentViewCondition: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateCacheRequest: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateTrueCondition: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateFalseCondition: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreatePropertyCondition: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        PROPERTYID,
        VARIANT,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreatePropertyConditionEx: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        PROPERTYID,
        VARIANT,
        PropertyConditionFlags,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateAndCondition: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateAndConditionFromArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const SAFEARRAY,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateAndConditionFromNativeArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const *mut core::ffi::c_void,
        i32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateOrCondition: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateOrConditionFromArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const SAFEARRAY,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateOrConditionFromNativeArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const *mut core::ffi::c_void,
        i32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateNotCondition: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub AddAutomationEventHandler: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        EVENTID,
        *mut core::ffi::c_void,
        TreeScope,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub RemoveAutomationEventHandler: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        EVENTID,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub AddPropertyChangedEventHandlerNativeArray:
        unsafe extern "system" fn(
            *mut core::ffi::c_void,
            *mut core::ffi::c_void,
            TreeScope,
            *mut core::ffi::c_void,
            *mut core::ffi::c_void,
            *const PROPERTYID,
            i32,
        ) -> windows_core::HRESULT,
    pub AddPropertyChangedEventHandler: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        TreeScope,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const SAFEARRAY,
    ) -> windows_core::HRESULT,
    pub RemovePropertyChangedEventHandler: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub AddStructureChangedEventHandler: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        TreeScope,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub RemoveStructureChangedEventHandler: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub AddFocusChangedEventHandler: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub RemoveFocusChangedEventHandler: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub RemoveAllEventHandlers:
        unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub IntNativeArrayToSafeArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const i32,
        i32,
        *mut *mut SAFEARRAY,
    ) -> windows_core::HRESULT,
    pub IntSafeArrayToNativeArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const SAFEARRAY,
        *mut *mut i32,
        *mut i32,
    ) -> windows_core::HRESULT,
    pub RectToVariant: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        RECT,
        *mut VARIANT,
    ) -> windows_core::HRESULT,
    pub VariantToRect: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        VARIANT,
        *mut RECT,
    ) -> windows_core::HRESULT,
    pub SafeArrayToRectNativeArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const SAFEARRAY,
        *mut *mut RECT,
        *mut i32,
    ) -> windows_core::HRESULT,
    pub CreateProxyFactoryEntry: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ProxyFactoryMapping: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetPropertyProgrammaticName: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        PROPERTYID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetPatternProgrammaticName: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        PATTERNID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub PollForPotentialSupportedPatterns: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut SAFEARRAY,
        *mut *mut SAFEARRAY,
    ) -> windows_core::HRESULT,
    pub PollForPotentialSupportedProperties: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut SAFEARRAY,
        *mut *mut SAFEARRAY,
    )
        -> windows_core::HRESULT,
    pub CheckNotSupported: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        VARIANT,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub ReservedNotSupportedValue: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ReservedMixedAttributeValue: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ElementFromIAccessible: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        i32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ElementFromIAccessibleBuildCache: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        i32,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
pub trait IUIAutomation_Impl: windows_core::IUnknownImpl {
    fn CompareElements(
        &self,
        el1: windows_core::Ref<IUIAutomationElement>,
        el2: windows_core::Ref<IUIAutomationElement>,
    ) -> windows_core::Result<windows_core::BOOL>;
    fn CompareRuntimeIds(
        &self,
        runtimeid1: *const SAFEARRAY,
        runtimeid2: *const SAFEARRAY,
    ) -> windows_core::Result<windows_core::BOOL>;
    fn GetRootElement(&self) -> windows_core::Result<IUIAutomationElement>;
    fn ElementFromHandle(&self, hwnd: UIA_HWND) -> windows_core::Result<IUIAutomationElement>;
    fn ElementFromPoint(&self, pt: &POINT) -> windows_core::Result<IUIAutomationElement>;
    fn GetFocusedElement(&self) -> windows_core::Result<IUIAutomationElement>;
    fn GetRootElementBuildCache(
        &self,
        cacherequest: windows_core::Ref<IUIAutomationCacheRequest>,
    ) -> windows_core::Result<IUIAutomationElement>;
    fn ElementFromHandleBuildCache(
        &self,
        hwnd: UIA_HWND,
        cacherequest: windows_core::Ref<IUIAutomationCacheRequest>,
    ) -> windows_core::Result<IUIAutomationElement>;
    fn ElementFromPointBuildCache(
        &self,
        pt: &POINT,
        cacherequest: windows_core::Ref<IUIAutomationCacheRequest>,
    ) -> windows_core::Result<IUIAutomationElement>;
    fn GetFocusedElementBuildCache(
        &self,
        cacherequest: windows_core::Ref<IUIAutomationCacheRequest>,
    ) -> windows_core::Result<IUIAutomationElement>;
    fn CreateTreeWalker(
        &self,
        pcondition: windows_core::Ref<IUIAutomationCondition>,
    ) -> windows_core::Result<IUIAutomationTreeWalker>;
    fn ControlViewWalker(&self) -> windows_core::Result<IUIAutomationTreeWalker>;
    fn ContentViewWalker(&self) -> windows_core::Result<IUIAutomationTreeWalker>;
    fn RawViewWalker(&self) -> windows_core::Result<IUIAutomationTreeWalker>;
    fn RawViewCondition(&self) -> windows_core::Result<IUIAutomationCondition>;
    fn ControlViewCondition(&self) -> windows_core::Result<IUIAutomationCondition>;
    fn ContentViewCondition(&self) -> windows_core::Result<IUIAutomationCondition>;
    fn CreateCacheRequest(&self) -> windows_core::Result<IUIAutomationCacheRequest>;
    fn CreateTrueCondition(&self) -> windows_core::Result<IUIAutomationCondition>;
    fn CreateFalseCondition(&self) -> windows_core::Result<IUIAutomationCondition>;
    fn CreatePropertyCondition(
        &self,
        propertyid: PROPERTYID,
        value: &VARIANT,
    ) -> windows_core::Result<IUIAutomationCondition>;
    fn CreatePropertyConditionEx(
        &self,
        propertyid: PROPERTYID,
        value: &VARIANT,
        flags: PropertyConditionFlags,
    ) -> windows_core::Result<IUIAutomationCondition>;
    fn CreateAndCondition(
        &self,
        condition1: windows_core::Ref<IUIAutomationCondition>,
        condition2: windows_core::Ref<IUIAutomationCondition>,
    ) -> windows_core::Result<IUIAutomationCondition>;
    fn CreateAndConditionFromArray(
        &self,
        conditions: *const SAFEARRAY,
    ) -> windows_core::Result<IUIAutomationCondition>;
    fn CreateAndConditionFromNativeArray(
        &self,
        conditions: *const Option<IUIAutomationCondition>,
        conditioncount: i32,
    ) -> windows_core::Result<IUIAutomationCondition>;
    fn CreateOrCondition(
        &self,
        condition1: windows_core::Ref<IUIAutomationCondition>,
        condition2: windows_core::Ref<IUIAutomationCondition>,
    ) -> windows_core::Result<IUIAutomationCondition>;
    fn CreateOrConditionFromArray(
        &self,
        conditions: *const SAFEARRAY,
    ) -> windows_core::Result<IUIAutomationCondition>;
    fn CreateOrConditionFromNativeArray(
        &self,
        conditions: *const Option<IUIAutomationCondition>,
        conditioncount: i32,
    ) -> windows_core::Result<IUIAutomationCondition>;
    fn CreateNotCondition(
        &self,
        condition: windows_core::Ref<IUIAutomationCondition>,
    ) -> windows_core::Result<IUIAutomationCondition>;
    fn AddAutomationEventHandler(
        &self,
        eventid: EVENTID,
        element: windows_core::Ref<IUIAutomationElement>,
        scope: TreeScope,
        cacherequest: windows_core::Ref<IUIAutomationCacheRequest>,
        handler: windows_core::Ref<IUIAutomationEventHandler>,
    ) -> windows_core::Result<()>;
    fn RemoveAutomationEventHandler(
        &self,
        eventid: EVENTID,
        element: windows_core::Ref<IUIAutomationElement>,
        handler: windows_core::Ref<IUIAutomationEventHandler>,
    ) -> windows_core::Result<()>;
    fn AddPropertyChangedEventHandlerNativeArray(
        &self,
        element: windows_core::Ref<IUIAutomationElement>,
        scope: TreeScope,
        cacherequest: windows_core::Ref<IUIAutomationCacheRequest>,
        handler: windows_core::Ref<IUIAutomationPropertyChangedEventHandler>,
        propertyarray: *const PROPERTYID,
        propertycount: i32,
    ) -> windows_core::Result<()>;
    fn AddPropertyChangedEventHandler(
        &self,
        element: windows_core::Ref<IUIAutomationElement>,
        scope: TreeScope,
        cacherequest: windows_core::Ref<IUIAutomationCacheRequest>,
        handler: windows_core::Ref<IUIAutomationPropertyChangedEventHandler>,
        propertyarray: *const SAFEARRAY,
    ) -> windows_core::Result<()>;
    fn RemovePropertyChangedEventHandler(
        &self,
        element: windows_core::Ref<IUIAutomationElement>,
        handler: windows_core::Ref<IUIAutomationPropertyChangedEventHandler>,
    ) -> windows_core::Result<()>;
    fn AddStructureChangedEventHandler(
        &self,
        element: windows_core::Ref<IUIAutomationElement>,
        scope: TreeScope,
        cacherequest: windows_core::Ref<IUIAutomationCacheRequest>,
        handler: windows_core::Ref<IUIAutomationStructureChangedEventHandler>,
    ) -> windows_core::Result<()>;
    fn RemoveStructureChangedEventHandler(
        &self,
        element: windows_core::Ref<IUIAutomationElement>,
        handler: windows_core::Ref<IUIAutomationStructureChangedEventHandler>,
    ) -> windows_core::Result<()>;
    fn AddFocusChangedEventHandler(
        &self,
        cacherequest: windows_core::Ref<IUIAutomationCacheRequest>,
        handler: windows_core::Ref<IUIAutomationFocusChangedEventHandler>,
    ) -> windows_core::Result<()>;
    fn RemoveFocusChangedEventHandler(
        &self,
        handler: windows_core::Ref<IUIAutomationFocusChangedEventHandler>,
    ) -> windows_core::Result<()>;
    fn RemoveAllEventHandlers(&self) -> windows_core::Result<()>;
    fn IntNativeArrayToSafeArray(
        &self,
        array: *const i32,
        arraycount: i32,
    ) -> windows_core::Result<*mut SAFEARRAY>;
    fn IntSafeArrayToNativeArray(
        &self,
        intarray: *const SAFEARRAY,
        array: *mut *mut i32,
    ) -> windows_core::Result<i32>;
    fn RectToVariant(&self, rc: &RECT) -> windows_core::Result<VARIANT>;
    fn VariantToRect(&self, var: &VARIANT) -> windows_core::Result<RECT>;
    fn SafeArrayToRectNativeArray(
        &self,
        rects: *const SAFEARRAY,
        rectarray: *mut *mut RECT,
    ) -> windows_core::Result<i32>;
    fn CreateProxyFactoryEntry(
        &self,
        factory: windows_core::Ref<IUIAutomationProxyFactory>,
    ) -> windows_core::Result<IUIAutomationProxyFactoryEntry>;
    fn ProxyFactoryMapping(&self) -> windows_core::Result<IUIAutomationProxyFactoryMapping>;
    fn GetPropertyProgrammaticName(
        &self,
        property: PROPERTYID,
    ) -> windows_core::Result<windows_core::BSTR>;
    fn GetPatternProgrammaticName(
        &self,
        pattern: PATTERNID,
    ) -> windows_core::Result<windows_core::BSTR>;
    fn PollForPotentialSupportedPatterns(
        &self,
        pelement: windows_core::Ref<IUIAutomationElement>,
        patternids: *mut *mut SAFEARRAY,
        patternnames: *mut *mut SAFEARRAY,
    ) -> windows_core::Result<()>;
    fn PollForPotentialSupportedProperties(
        &self,
        pelement: windows_core::Ref<IUIAutomationElement>,
        propertyids: *mut *mut SAFEARRAY,
        propertynames: *mut *mut SAFEARRAY,
    ) -> windows_core::Result<()>;
    fn CheckNotSupported(&self, value: &VARIANT) -> windows_core::Result<windows_core::BOOL>;
    fn ReservedNotSupportedValue(&self) -> windows_core::Result<windows_core::IUnknown>;
    fn ReservedMixedAttributeValue(&self) -> windows_core::Result<windows_core::IUnknown>;
    fn ElementFromIAccessible(
        &self,
        accessible: windows_core::Ref<IAccessible>,
        childid: i32,
    ) -> windows_core::Result<IUIAutomationElement>;
    fn ElementFromIAccessibleBuildCache(
        &self,
        accessible: windows_core::Ref<IAccessible>,
        childid: i32,
        cacherequest: windows_core::Ref<IUIAutomationCacheRequest>,
    ) -> windows_core::Result<IUIAutomationElement>;
}
impl IUIAutomation_Vtbl {
    pub const fn new<Identity: IUIAutomation_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn CompareElements<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            el1: *mut core::ffi::c_void,
            el2: *mut core::ffi::c_void,
            aresame: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CompareElements(
                    this,
                    core::mem::transmute_copy(&el1),
                    core::mem::transmute_copy(&el2),
                ) {
                    Ok(ok__) => {
                        aresame.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CompareRuntimeIds<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            runtimeid1: *const SAFEARRAY,
            runtimeid2: *const SAFEARRAY,
            aresame: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CompareRuntimeIds(
                    this,
                    core::mem::transmute_copy(&runtimeid1),
                    core::mem::transmute_copy(&runtimeid2),
                ) {
                    Ok(ok__) => {
                        aresame.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetRootElement<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            root: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::GetRootElement(this) {
                    Ok(ok__) => {
                        root.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn ElementFromHandle<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            hwnd: UIA_HWND,
            element: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::ElementFromHandle(this, core::mem::transmute_copy(&hwnd))
                {
                    Ok(ok__) => {
                        element.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn ElementFromPoint<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pt: POINT,
            element: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::ElementFromPoint(this, core::mem::transmute(&pt)) {
                    Ok(ok__) => {
                        element.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetFocusedElement<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            element: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::GetFocusedElement(this) {
                    Ok(ok__) => {
                        element.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetRootElementBuildCache<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            cacherequest: *mut core::ffi::c_void,
            root: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::GetRootElementBuildCache(
                    this,
                    core::mem::transmute_copy(&cacherequest),
                ) {
                    Ok(ok__) => {
                        root.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn ElementFromHandleBuildCache<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            hwnd: UIA_HWND,
            cacherequest: *mut core::ffi::c_void,
            element: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::ElementFromHandleBuildCache(
                    this,
                    core::mem::transmute_copy(&hwnd),
                    core::mem::transmute_copy(&cacherequest),
                ) {
                    Ok(ok__) => {
                        element.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn ElementFromPointBuildCache<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pt: POINT,
            cacherequest: *mut core::ffi::c_void,
            element: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::ElementFromPointBuildCache(
                    this,
                    core::mem::transmute(&pt),
                    core::mem::transmute_copy(&cacherequest),
                ) {
                    Ok(ok__) => {
                        element.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetFocusedElementBuildCache<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            cacherequest: *mut core::ffi::c_void,
            element: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::GetFocusedElementBuildCache(
                    this,
                    core::mem::transmute_copy(&cacherequest),
                ) {
                    Ok(ok__) => {
                        element.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateTreeWalker<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pcondition: *mut core::ffi::c_void,
            walker: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CreateTreeWalker(
                    this,
                    core::mem::transmute_copy(&pcondition),
                ) {
                    Ok(ok__) => {
                        walker.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn ControlViewWalker<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            walker: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::ControlViewWalker(this) {
                    Ok(ok__) => {
                        walker.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn ContentViewWalker<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            walker: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::ContentViewWalker(this) {
                    Ok(ok__) => {
                        walker.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn RawViewWalker<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            walker: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::RawViewWalker(this) {
                    Ok(ok__) => {
                        walker.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn RawViewCondition<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            condition: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::RawViewCondition(this) {
                    Ok(ok__) => {
                        condition.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn ControlViewCondition<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            condition: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::ControlViewCondition(this) {
                    Ok(ok__) => {
                        condition.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn ContentViewCondition<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            condition: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::ContentViewCondition(this) {
                    Ok(ok__) => {
                        condition.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateCacheRequest<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            cacherequest: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CreateCacheRequest(this) {
                    Ok(ok__) => {
                        cacherequest.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateTrueCondition<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            newcondition: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CreateTrueCondition(this) {
                    Ok(ok__) => {
                        newcondition.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateFalseCondition<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            newcondition: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CreateFalseCondition(this) {
                    Ok(ok__) => {
                        newcondition.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreatePropertyCondition<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            propertyid: PROPERTYID,
            value: VARIANT,
            newcondition: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CreatePropertyCondition(
                    this,
                    core::mem::transmute_copy(&propertyid),
                    core::mem::transmute(&value),
                ) {
                    Ok(ok__) => {
                        newcondition.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreatePropertyConditionEx<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            propertyid: PROPERTYID,
            value: VARIANT,
            flags: PropertyConditionFlags,
            newcondition: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CreatePropertyConditionEx(
                    this,
                    core::mem::transmute_copy(&propertyid),
                    core::mem::transmute(&value),
                    core::mem::transmute_copy(&flags),
                ) {
                    Ok(ok__) => {
                        newcondition.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateAndCondition<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            condition1: *mut core::ffi::c_void,
            condition2: *mut core::ffi::c_void,
            newcondition: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CreateAndCondition(
                    this,
                    core::mem::transmute_copy(&condition1),
                    core::mem::transmute_copy(&condition2),
                ) {
                    Ok(ok__) => {
                        newcondition.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateAndConditionFromArray<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            conditions: *const SAFEARRAY,
            newcondition: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CreateAndConditionFromArray(
                    this,
                    core::mem::transmute_copy(&conditions),
                ) {
                    Ok(ok__) => {
                        newcondition.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateAndConditionFromNativeArray<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            conditions: *const *mut core::ffi::c_void,
            conditioncount: i32,
            newcondition: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CreateAndConditionFromNativeArray(
                    this,
                    core::mem::transmute_copy(&conditions),
                    core::mem::transmute_copy(&conditioncount),
                ) {
                    Ok(ok__) => {
                        newcondition.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateOrCondition<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            condition1: *mut core::ffi::c_void,
            condition2: *mut core::ffi::c_void,
            newcondition: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CreateOrCondition(
                    this,
                    core::mem::transmute_copy(&condition1),
                    core::mem::transmute_copy(&condition2),
                ) {
                    Ok(ok__) => {
                        newcondition.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateOrConditionFromArray<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            conditions: *const SAFEARRAY,
            newcondition: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CreateOrConditionFromArray(
                    this,
                    core::mem::transmute_copy(&conditions),
                ) {
                    Ok(ok__) => {
                        newcondition.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateOrConditionFromNativeArray<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            conditions: *const *mut core::ffi::c_void,
            conditioncount: i32,
            newcondition: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CreateOrConditionFromNativeArray(
                    this,
                    core::mem::transmute_copy(&conditions),
                    core::mem::transmute_copy(&conditioncount),
                ) {
                    Ok(ok__) => {
                        newcondition.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateNotCondition<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            condition: *mut core::ffi::c_void,
            newcondition: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CreateNotCondition(
                    this,
                    core::mem::transmute_copy(&condition),
                ) {
                    Ok(ok__) => {
                        newcondition.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn AddAutomationEventHandler<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            eventid: EVENTID,
            element: *mut core::ffi::c_void,
            scope: TreeScope,
            cacherequest: *mut core::ffi::c_void,
            handler: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomation_Impl::AddAutomationEventHandler(
                    this,
                    core::mem::transmute_copy(&eventid),
                    core::mem::transmute_copy(&element),
                    core::mem::transmute_copy(&scope),
                    core::mem::transmute_copy(&cacherequest),
                    core::mem::transmute_copy(&handler),
                )
                .into()
            }
        }
        unsafe extern "system" fn RemoveAutomationEventHandler<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            eventid: EVENTID,
            element: *mut core::ffi::c_void,
            handler: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomation_Impl::RemoveAutomationEventHandler(
                    this,
                    core::mem::transmute_copy(&eventid),
                    core::mem::transmute_copy(&element),
                    core::mem::transmute_copy(&handler),
                )
                .into()
            }
        }
        unsafe extern "system" fn AddPropertyChangedEventHandlerNativeArray<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            element: *mut core::ffi::c_void,
            scope: TreeScope,
            cacherequest: *mut core::ffi::c_void,
            handler: *mut core::ffi::c_void,
            propertyarray: *const PROPERTYID,
            propertycount: i32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomation_Impl::AddPropertyChangedEventHandlerNativeArray(
                    this,
                    core::mem::transmute_copy(&element),
                    core::mem::transmute_copy(&scope),
                    core::mem::transmute_copy(&cacherequest),
                    core::mem::transmute_copy(&handler),
                    core::mem::transmute_copy(&propertyarray),
                    core::mem::transmute_copy(&propertycount),
                )
                .into()
            }
        }
        unsafe extern "system" fn AddPropertyChangedEventHandler<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            element: *mut core::ffi::c_void,
            scope: TreeScope,
            cacherequest: *mut core::ffi::c_void,
            handler: *mut core::ffi::c_void,
            propertyarray: *const SAFEARRAY,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomation_Impl::AddPropertyChangedEventHandler(
                    this,
                    core::mem::transmute_copy(&element),
                    core::mem::transmute_copy(&scope),
                    core::mem::transmute_copy(&cacherequest),
                    core::mem::transmute_copy(&handler),
                    core::mem::transmute_copy(&propertyarray),
                )
                .into()
            }
        }
        unsafe extern "system" fn RemovePropertyChangedEventHandler<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            element: *mut core::ffi::c_void,
            handler: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomation_Impl::RemovePropertyChangedEventHandler(
                    this,
                    core::mem::transmute_copy(&element),
                    core::mem::transmute_copy(&handler),
                )
                .into()
            }
        }
        unsafe extern "system" fn AddStructureChangedEventHandler<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            element: *mut core::ffi::c_void,
            scope: TreeScope,
            cacherequest: *mut core::ffi::c_void,
            handler: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomation_Impl::AddStructureChangedEventHandler(
                    this,
                    core::mem::transmute_copy(&element),
                    core::mem::transmute_copy(&scope),
                    core::mem::transmute_copy(&cacherequest),
                    core::mem::transmute_copy(&handler),
                )
                .into()
            }
        }
        unsafe extern "system" fn RemoveStructureChangedEventHandler<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            element: *mut core::ffi::c_void,
            handler: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomation_Impl::RemoveStructureChangedEventHandler(
                    this,
                    core::mem::transmute_copy(&element),
                    core::mem::transmute_copy(&handler),
                )
                .into()
            }
        }
        unsafe extern "system" fn AddFocusChangedEventHandler<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            cacherequest: *mut core::ffi::c_void,
            handler: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomation_Impl::AddFocusChangedEventHandler(
                    this,
                    core::mem::transmute_copy(&cacherequest),
                    core::mem::transmute_copy(&handler),
                )
                .into()
            }
        }
        unsafe extern "system" fn RemoveFocusChangedEventHandler<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            handler: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomation_Impl::RemoveFocusChangedEventHandler(
                    this,
                    core::mem::transmute_copy(&handler),
                )
                .into()
            }
        }
        unsafe extern "system" fn RemoveAllEventHandlers<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomation_Impl::RemoveAllEventHandlers(this).into()
            }
        }
        unsafe extern "system" fn IntNativeArrayToSafeArray<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            array: *const i32,
            arraycount: i32,
            safearray: *mut *mut SAFEARRAY,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::IntNativeArrayToSafeArray(
                    this,
                    core::mem::transmute_copy(&array),
                    core::mem::transmute_copy(&arraycount),
                ) {
                    Ok(ok__) => {
                        safearray.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn IntSafeArrayToNativeArray<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            intarray: *const SAFEARRAY,
            array: *mut *mut i32,
            arraycount: *mut i32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::IntSafeArrayToNativeArray(
                    this,
                    core::mem::transmute_copy(&intarray),
                    core::mem::transmute_copy(&array),
                ) {
                    Ok(ok__) => {
                        arraycount.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn RectToVariant<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            rc: RECT,
            var: *mut VARIANT,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::RectToVariant(this, core::mem::transmute(&rc)) {
                    Ok(ok__) => {
                        var.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn VariantToRect<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            var: VARIANT,
            rc: *mut RECT,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::VariantToRect(this, core::mem::transmute(&var)) {
                    Ok(ok__) => {
                        rc.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn SafeArrayToRectNativeArray<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            rects: *const SAFEARRAY,
            rectarray: *mut *mut RECT,
            rectarraycount: *mut i32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::SafeArrayToRectNativeArray(
                    this,
                    core::mem::transmute_copy(&rects),
                    core::mem::transmute_copy(&rectarray),
                ) {
                    Ok(ok__) => {
                        rectarraycount.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateProxyFactoryEntry<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            factory: *mut core::ffi::c_void,
            factoryentry: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CreateProxyFactoryEntry(
                    this,
                    core::mem::transmute_copy(&factory),
                ) {
                    Ok(ok__) => {
                        factoryentry.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn ProxyFactoryMapping<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            factorymapping: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::ProxyFactoryMapping(this) {
                    Ok(ok__) => {
                        factorymapping.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetPropertyProgrammaticName<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            property: PROPERTYID,
            name: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::GetPropertyProgrammaticName(
                    this,
                    core::mem::transmute_copy(&property),
                ) {
                    Ok(ok__) => {
                        name.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetPatternProgrammaticName<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pattern: PATTERNID,
            name: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::GetPatternProgrammaticName(
                    this,
                    core::mem::transmute_copy(&pattern),
                ) {
                    Ok(ok__) => {
                        name.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn PollForPotentialSupportedPatterns<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pelement: *mut core::ffi::c_void,
            patternids: *mut *mut SAFEARRAY,
            patternnames: *mut *mut SAFEARRAY,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomation_Impl::PollForPotentialSupportedPatterns(
                    this,
                    core::mem::transmute_copy(&pelement),
                    core::mem::transmute_copy(&patternids),
                    core::mem::transmute_copy(&patternnames),
                )
                .into()
            }
        }
        unsafe extern "system" fn PollForPotentialSupportedProperties<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pelement: *mut core::ffi::c_void,
            propertyids: *mut *mut SAFEARRAY,
            propertynames: *mut *mut SAFEARRAY,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomation_Impl::PollForPotentialSupportedProperties(
                    this,
                    core::mem::transmute_copy(&pelement),
                    core::mem::transmute_copy(&propertyids),
                    core::mem::transmute_copy(&propertynames),
                )
                .into()
            }
        }
        unsafe extern "system" fn CheckNotSupported<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            value: VARIANT,
            isnotsupported: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CheckNotSupported(this, core::mem::transmute(&value)) {
                    Ok(ok__) => {
                        isnotsupported.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn ReservedNotSupportedValue<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            notsupportedvalue: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::ReservedNotSupportedValue(this) {
                    Ok(ok__) => {
                        notsupportedvalue.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn ReservedMixedAttributeValue<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            mixedattributevalue: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::ReservedMixedAttributeValue(this) {
                    Ok(ok__) => {
                        mixedattributevalue.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn ElementFromIAccessible<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            accessible: *mut core::ffi::c_void,
            childid: i32,
            element: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::ElementFromIAccessible(
                    this,
                    core::mem::transmute_copy(&accessible),
                    core::mem::transmute_copy(&childid),
                ) {
                    Ok(ok__) => {
                        element.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn ElementFromIAccessibleBuildCache<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            accessible: *mut core::ffi::c_void,
            childid: i32,
            cacherequest: *mut core::ffi::c_void,
            element: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::ElementFromIAccessibleBuildCache(
                    this,
                    core::mem::transmute_copy(&accessible),
                    core::mem::transmute_copy(&childid),
                    core::mem::transmute_copy(&cacherequest),
                ) {
                    Ok(ok__) => {
                        element.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            CompareElements: CompareElements::<Identity, OFFSET>,
            CompareRuntimeIds: CompareRuntimeIds::<Identity, OFFSET>,
            GetRootElement: GetRootElement::<Identity, OFFSET>,
            ElementFromHandle: ElementFromHandle::<Identity, OFFSET>,
            ElementFromPoint: ElementFromPoint::<Identity, OFFSET>,
            GetFocusedElement: GetFocusedElement::<Identity, OFFSET>,
            GetRootElementBuildCache: GetRootElementBuildCache::<Identity, OFFSET>,
            ElementFromHandleBuildCache: ElementFromHandleBuildCache::<Identity, OFFSET>,
            ElementFromPointBuildCache: ElementFromPointBuildCache::<Identity, OFFSET>,
            GetFocusedElementBuildCache: GetFocusedElementBuildCache::<Identity, OFFSET>,
            CreateTreeWalker: CreateTreeWalker::<Identity, OFFSET>,
            ControlViewWalker: ControlViewWalker::<Identity, OFFSET>,
            ContentViewWalker: ContentViewWalker::<Identity, OFFSET>,
            RawViewWalker: RawViewWalker::<Identity, OFFSET>,
            RawViewCondition: RawViewCondition::<Identity, OFFSET>,
            ControlViewCondition: ControlViewCondition::<Identity, OFFSET>,
            ContentViewCondition: ContentViewCondition::<Identity, OFFSET>,
            CreateCacheRequest: CreateCacheRequest::<Identity, OFFSET>,
            CreateTrueCondition: CreateTrueCondition::<Identity, OFFSET>,
            CreateFalseCondition: CreateFalseCondition::<Identity, OFFSET>,
            CreatePropertyCondition: CreatePropertyCondition::<Identity, OFFSET>,
            CreatePropertyConditionEx: CreatePropertyConditionEx::<Identity, OFFSET>,
            CreateAndCondition: CreateAndCondition::<Identity, OFFSET>,
            CreateAndConditionFromArray: CreateAndConditionFromArray::<Identity, OFFSET>,
            CreateAndConditionFromNativeArray: CreateAndConditionFromNativeArray::<Identity, OFFSET>,
            CreateOrCondition: CreateOrCondition::<Identity, OFFSET>,
            CreateOrConditionFromArray: CreateOrConditionFromArray::<Identity, OFFSET>,
            CreateOrConditionFromNativeArray: CreateOrConditionFromNativeArray::<Identity, OFFSET>,
            CreateNotCondition: CreateNotCondition::<Identity, OFFSET>,
            AddAutomationEventHandler: AddAutomationEventHandler::<Identity, OFFSET>,
            RemoveAutomationEventHandler: RemoveAutomationEventHandler::<Identity, OFFSET>,
            AddPropertyChangedEventHandlerNativeArray: AddPropertyChangedEventHandlerNativeArray::<
                Identity,
                OFFSET,
            >,
            AddPropertyChangedEventHandler: AddPropertyChangedEventHandler::<Identity, OFFSET>,
            RemovePropertyChangedEventHandler: RemovePropertyChangedEventHandler::<Identity, OFFSET>,
            AddStructureChangedEventHandler: AddStructureChangedEventHandler::<Identity, OFFSET>,
            RemoveStructureChangedEventHandler: RemoveStructureChangedEventHandler::<
                Identity,
                OFFSET,
            >,
            AddFocusChangedEventHandler: AddFocusChangedEventHandler::<Identity, OFFSET>,
            RemoveFocusChangedEventHandler: RemoveFocusChangedEventHandler::<Identity, OFFSET>,
            RemoveAllEventHandlers: RemoveAllEventHandlers::<Identity, OFFSET>,
            IntNativeArrayToSafeArray: IntNativeArrayToSafeArray::<Identity, OFFSET>,
            IntSafeArrayToNativeArray: IntSafeArrayToNativeArray::<Identity, OFFSET>,
            RectToVariant: RectToVariant::<Identity, OFFSET>,
            VariantToRect: VariantToRect::<Identity, OFFSET>,
            SafeArrayToRectNativeArray: SafeArrayToRectNativeArray::<Identity, OFFSET>,
            CreateProxyFactoryEntry: CreateProxyFactoryEntry::<Identity, OFFSET>,
            ProxyFactoryMapping: ProxyFactoryMapping::<Identity, OFFSET>,
            GetPropertyProgrammaticName: GetPropertyProgrammaticName::<Identity, OFFSET>,
            GetPatternProgrammaticName: GetPatternProgrammaticName::<Identity, OFFSET>,
            PollForPotentialSupportedPatterns: PollForPotentialSupportedPatterns::<Identity, OFFSET>,
            PollForPotentialSupportedProperties: PollForPotentialSupportedProperties::<
                Identity,
                OFFSET,
            >,
            CheckNotSupported: CheckNotSupported::<Identity, OFFSET>,
            ReservedNotSupportedValue: ReservedNotSupportedValue::<Identity, OFFSET>,
            ReservedMixedAttributeValue: ReservedMixedAttributeValue::<Identity, OFFSET>,
            ElementFromIAccessible: ElementFromIAccessible::<Identity, OFFSET>,
            ElementFromIAccessibleBuildCache: ElementFromIAccessibleBuildCache::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IUIAutomation as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IUIAutomation {}
windows_core::imp::define_interface!(
    IUIAutomationCacheRequest,
    IUIAutomationCacheRequest_Vtbl,
    0xb32a92b5_bc25_4078_9c08_d7ee95c48e03
);
windows_core::imp::interface_hierarchy!(IUIAutomationCacheRequest, windows_core::IUnknown);
#[repr(C)]
pub struct IUIAutomationCacheRequest_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    AddProperty: usize,
    AddPattern: usize,
    Clone: usize,
    TreeScope: usize,
    SetTreeScope: usize,
    TreeFilter: usize,
    SetTreeFilter: usize,
    AutomationElementMode: usize,
    SetAutomationElementMode: usize,
}
impl windows_core::RuntimeName for IUIAutomationCacheRequest {}
windows_core::imp::define_interface!(
    IUIAutomationCondition,
    IUIAutomationCondition_Vtbl,
    0x352ffba8_0973_437c_a61f_f64cafd81df9
);
windows_core::imp::interface_hierarchy!(IUIAutomationCondition, windows_core::IUnknown);
#[repr(C)]
pub struct IUIAutomationCondition_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
}
pub trait IUIAutomationCondition_Impl: windows_core::IUnknownImpl {}
impl IUIAutomationCondition_Vtbl {
    pub const fn new<Identity: IUIAutomationCondition_Impl, const OFFSET: isize>() -> Self {
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IUIAutomationCondition as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IUIAutomationCondition {}
windows_core::imp::define_interface!(
    IUIAutomationElement,
    IUIAutomationElement_Vtbl,
    0xd22108aa_8ac5_49a5_837b_37bbb3d7591e
);
windows_core::imp::interface_hierarchy!(IUIAutomationElement, windows_core::IUnknown);
impl IUIAutomationElement {
    pub(crate) unsafe fn SetFocus(&self) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).SetFocus)(windows_core::Interface::as_raw(self))
        }
    }
    pub(crate) unsafe fn GetRuntimeId(&self) -> windows_core::Result<*mut SAFEARRAY> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetRuntimeId)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn FindFirst<P1>(
        &self,
        scope: TreeScope,
        condition: P1,
    ) -> windows_core::Result<Self>
    where
        P1: windows_core::Param<IUIAutomationCondition>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FindFirst)(
                windows_core::Interface::as_raw(self),
                scope,
                condition.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn FindAll<P1>(
        &self,
        scope: TreeScope,
        condition: P1,
    ) -> windows_core::Result<IUIAutomationElementArray>
    where
        P1: windows_core::Param<IUIAutomationCondition>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FindAll)(
                windows_core::Interface::as_raw(self),
                scope,
                condition.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn FindFirstBuildCache<P1, P2>(
        &self,
        scope: TreeScope,
        condition: P1,
        cacherequest: P2,
    ) -> windows_core::Result<Self>
    where
        P1: windows_core::Param<IUIAutomationCondition>,
        P2: windows_core::Param<IUIAutomationCacheRequest>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FindFirstBuildCache)(
                windows_core::Interface::as_raw(self),
                scope,
                condition.param().abi(),
                cacherequest.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn FindAllBuildCache<P1, P2>(
        &self,
        scope: TreeScope,
        condition: P1,
        cacherequest: P2,
    ) -> windows_core::Result<IUIAutomationElementArray>
    where
        P1: windows_core::Param<IUIAutomationCondition>,
        P2: windows_core::Param<IUIAutomationCacheRequest>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FindAllBuildCache)(
                windows_core::Interface::as_raw(self),
                scope,
                condition.param().abi(),
                cacherequest.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn BuildUpdatedCache<P0>(
        &self,
        cacherequest: P0,
    ) -> windows_core::Result<Self>
    where
        P0: windows_core::Param<IUIAutomationCacheRequest>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).BuildUpdatedCache)(
                windows_core::Interface::as_raw(self),
                cacherequest.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetCurrentPropertyValue(
        &self,
        propertyid: PROPERTYID,
    ) -> windows_core::Result<VARIANT> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetCurrentPropertyValue)(
                windows_core::Interface::as_raw(self),
                propertyid,
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn GetCurrentPropertyValueEx(
        &self,
        propertyid: PROPERTYID,
        ignoredefaultvalue: bool,
    ) -> windows_core::Result<VARIANT> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetCurrentPropertyValueEx)(
                windows_core::Interface::as_raw(self),
                propertyid,
                ignoredefaultvalue.into(),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn GetCachedPropertyValue(
        &self,
        propertyid: PROPERTYID,
    ) -> windows_core::Result<VARIANT> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetCachedPropertyValue)(
                windows_core::Interface::as_raw(self),
                propertyid,
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn GetCachedPropertyValueEx(
        &self,
        propertyid: PROPERTYID,
        ignoredefaultvalue: bool,
    ) -> windows_core::Result<VARIANT> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetCachedPropertyValueEx)(
                windows_core::Interface::as_raw(self),
                propertyid,
                ignoredefaultvalue.into(),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn GetCurrentPatternAs<T>(
        &self,
        patternid: PATTERNID,
    ) -> windows_core::Result<T>
    where
        T: windows_core::Interface,
    {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (windows_core::Interface::vtable(self).GetCurrentPatternAs)(
                windows_core::Interface::as_raw(self),
                patternid,
                &T::IID,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetCachedPatternAs<T>(
        &self,
        patternid: PATTERNID,
    ) -> windows_core::Result<T>
    where
        T: windows_core::Interface,
    {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (windows_core::Interface::vtable(self).GetCachedPatternAs)(
                windows_core::Interface::as_raw(self),
                patternid,
                &T::IID,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetCurrentPattern(
        &self,
        patternid: PATTERNID,
    ) -> windows_core::Result<windows_core::IUnknown> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetCurrentPattern)(
                windows_core::Interface::as_raw(self),
                patternid,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetCachedPattern(
        &self,
        patternid: PATTERNID,
    ) -> windows_core::Result<windows_core::IUnknown> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetCachedPattern)(
                windows_core::Interface::as_raw(self),
                patternid,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetCachedParent(&self) -> windows_core::Result<Self> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetCachedParent)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetCachedChildren(
        &self,
    ) -> windows_core::Result<IUIAutomationElementArray> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetCachedChildren)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CurrentProcessId(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentProcessId)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentControlType(&self) -> windows_core::Result<CONTROLTYPEID> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentControlType)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentLocalizedControlType(
        &self,
    ) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentLocalizedControlType)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CurrentName(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentName)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CurrentAcceleratorKey(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentAcceleratorKey)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CurrentAccessKey(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentAccessKey)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CurrentHasKeyboardFocus(
        &self,
    ) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentHasKeyboardFocus)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentIsKeyboardFocusable(
        &self,
    ) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentIsKeyboardFocusable)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentIsEnabled(&self) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentIsEnabled)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentAutomationId(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentAutomationId)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CurrentClassName(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentClassName)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CurrentHelpText(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentHelpText)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CurrentCulture(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentCulture)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentIsControlElement(
        &self,
    ) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentIsControlElement)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentIsContentElement(
        &self,
    ) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentIsContentElement)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentIsPassword(&self) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentIsPassword)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentNativeWindowHandle(&self) -> windows_core::Result<UIA_HWND> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentNativeWindowHandle)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentItemType(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentItemType)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CurrentIsOffscreen(&self) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentIsOffscreen)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentOrientation(&self) -> windows_core::Result<OrientationType> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentOrientation)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentFrameworkId(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentFrameworkId)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CurrentIsRequiredForForm(
        &self,
    ) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentIsRequiredForForm)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentItemStatus(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentItemStatus)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CurrentBoundingRectangle(&self) -> windows_core::Result<RECT> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentBoundingRectangle)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentLabeledBy(&self) -> windows_core::Result<Self> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentLabeledBy)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CurrentAriaRole(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentAriaRole)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CurrentAriaProperties(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentAriaProperties)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CurrentIsDataValidForForm(
        &self,
    ) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentIsDataValidForForm)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentControllerFor(
        &self,
    ) -> windows_core::Result<IUIAutomationElementArray> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentControllerFor)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CurrentDescribedBy(
        &self,
    ) -> windows_core::Result<IUIAutomationElementArray> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentDescribedBy)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CurrentFlowsTo(&self) -> windows_core::Result<IUIAutomationElementArray> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentFlowsTo)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CurrentProviderDescription(
        &self,
    ) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentProviderDescription)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CachedProcessId(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedProcessId)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedControlType(&self) -> windows_core::Result<CONTROLTYPEID> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedControlType)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedLocalizedControlType(
        &self,
    ) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedLocalizedControlType)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CachedName(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedName)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CachedAcceleratorKey(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedAcceleratorKey)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CachedAccessKey(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedAccessKey)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CachedHasKeyboardFocus(&self) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedHasKeyboardFocus)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedIsKeyboardFocusable(
        &self,
    ) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedIsKeyboardFocusable)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedIsEnabled(&self) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedIsEnabled)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedAutomationId(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedAutomationId)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CachedClassName(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedClassName)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CachedHelpText(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedHelpText)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CachedCulture(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedCulture)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedIsControlElement(&self) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedIsControlElement)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedIsContentElement(&self) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedIsContentElement)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedIsPassword(&self) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedIsPassword)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedNativeWindowHandle(&self) -> windows_core::Result<UIA_HWND> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedNativeWindowHandle)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedItemType(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedItemType)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CachedIsOffscreen(&self) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedIsOffscreen)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedOrientation(&self) -> windows_core::Result<OrientationType> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedOrientation)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedFrameworkId(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedFrameworkId)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CachedIsRequiredForForm(
        &self,
    ) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedIsRequiredForForm)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedItemStatus(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedItemStatus)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CachedBoundingRectangle(&self) -> windows_core::Result<RECT> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedBoundingRectangle)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedLabeledBy(&self) -> windows_core::Result<Self> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedLabeledBy)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CachedAriaRole(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedAriaRole)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CachedAriaProperties(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedAriaProperties)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CachedIsDataValidForForm(
        &self,
    ) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedIsDataValidForForm)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedControllerFor(
        &self,
    ) -> windows_core::Result<IUIAutomationElementArray> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedControllerFor)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CachedDescribedBy(
        &self,
    ) -> windows_core::Result<IUIAutomationElementArray> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedDescribedBy)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CachedFlowsTo(&self) -> windows_core::Result<IUIAutomationElementArray> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedFlowsTo)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CachedProviderDescription(
        &self,
    ) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedProviderDescription)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn GetClickablePoint(
        &self,
        clickable: *mut POINT,
    ) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetClickablePoint)(
                windows_core::Interface::as_raw(self),
                clickable as _,
                &mut result__,
            )
            .map(|| result__)
        }
    }
}
#[repr(C)]
pub struct IUIAutomationElement_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub SetFocus: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub GetRuntimeId: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut SAFEARRAY,
    ) -> windows_core::HRESULT,
    pub FindFirst: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        TreeScope,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub FindAll: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        TreeScope,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub FindFirstBuildCache: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        TreeScope,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub FindAllBuildCache: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        TreeScope,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub BuildUpdatedCache: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetCurrentPropertyValue: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        PROPERTYID,
        *mut VARIANT,
    ) -> windows_core::HRESULT,
    pub GetCurrentPropertyValueEx: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        PROPERTYID,
        windows_core::BOOL,
        *mut VARIANT,
    ) -> windows_core::HRESULT,
    pub GetCachedPropertyValue: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        PROPERTYID,
        *mut VARIANT,
    ) -> windows_core::HRESULT,
    pub GetCachedPropertyValueEx: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        PROPERTYID,
        windows_core::BOOL,
        *mut VARIANT,
    ) -> windows_core::HRESULT,
    pub GetCurrentPatternAs: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        PATTERNID,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetCachedPatternAs: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        PATTERNID,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetCurrentPattern: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        PATTERNID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetCachedPattern: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        PATTERNID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetCachedParent: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetCachedChildren: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentProcessId:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub CurrentControlType: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut CONTROLTYPEID,
    ) -> windows_core::HRESULT,
    pub CurrentLocalizedControlType: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentName: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentAcceleratorKey: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentAccessKey: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentHasKeyboardFocus: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CurrentIsKeyboardFocusable: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CurrentIsEnabled: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CurrentAutomationId: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentClassName: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentHelpText: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentCulture:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub CurrentIsControlElement: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CurrentIsContentElement: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CurrentIsPassword: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CurrentNativeWindowHandle:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut UIA_HWND) -> windows_core::HRESULT,
    pub CurrentItemType: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentIsOffscreen: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CurrentOrientation: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut OrientationType,
    ) -> windows_core::HRESULT,
    pub CurrentFrameworkId: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentIsRequiredForForm: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CurrentItemStatus: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentBoundingRectangle:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut RECT) -> windows_core::HRESULT,
    pub CurrentLabeledBy: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentAriaRole: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentAriaProperties: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentIsDataValidForForm: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CurrentControllerFor: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentDescribedBy: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentFlowsTo: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentProviderDescription: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedProcessId:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub CachedControlType: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut CONTROLTYPEID,
    ) -> windows_core::HRESULT,
    pub CachedLocalizedControlType: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedName: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedAcceleratorKey: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedAccessKey: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedHasKeyboardFocus: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CachedIsKeyboardFocusable: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CachedIsEnabled: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CachedAutomationId: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedClassName: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedHelpText: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedCulture:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub CachedIsControlElement: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CachedIsContentElement: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CachedIsPassword: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CachedNativeWindowHandle:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut UIA_HWND) -> windows_core::HRESULT,
    pub CachedItemType: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedIsOffscreen: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CachedOrientation: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut OrientationType,
    ) -> windows_core::HRESULT,
    pub CachedFrameworkId: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedIsRequiredForForm: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CachedItemStatus: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedBoundingRectangle:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut RECT) -> windows_core::HRESULT,
    pub CachedLabeledBy: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedAriaRole: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedAriaProperties: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedIsDataValidForForm: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CachedControllerFor: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedDescribedBy: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedFlowsTo: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedProviderDescription: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetClickablePoint: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut POINT,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
}
pub trait IUIAutomationElement_Impl: windows_core::IUnknownImpl {
    fn SetFocus(&self) -> windows_core::Result<()>;
    fn GetRuntimeId(&self) -> windows_core::Result<*mut SAFEARRAY>;
    fn FindFirst(
        &self,
        scope: TreeScope,
        condition: windows_core::Ref<IUIAutomationCondition>,
    ) -> windows_core::Result<IUIAutomationElement>;
    fn FindAll(
        &self,
        scope: TreeScope,
        condition: windows_core::Ref<IUIAutomationCondition>,
    ) -> windows_core::Result<IUIAutomationElementArray>;
    fn FindFirstBuildCache(
        &self,
        scope: TreeScope,
        condition: windows_core::Ref<IUIAutomationCondition>,
        cacherequest: windows_core::Ref<IUIAutomationCacheRequest>,
    ) -> windows_core::Result<IUIAutomationElement>;
    fn FindAllBuildCache(
        &self,
        scope: TreeScope,
        condition: windows_core::Ref<IUIAutomationCondition>,
        cacherequest: windows_core::Ref<IUIAutomationCacheRequest>,
    ) -> windows_core::Result<IUIAutomationElementArray>;
    fn BuildUpdatedCache(
        &self,
        cacherequest: windows_core::Ref<IUIAutomationCacheRequest>,
    ) -> windows_core::Result<IUIAutomationElement>;
    fn GetCurrentPropertyValue(&self, propertyid: PROPERTYID) -> windows_core::Result<VARIANT>;
    fn GetCurrentPropertyValueEx(
        &self,
        propertyid: PROPERTYID,
        ignoredefaultvalue: windows_core::BOOL,
    ) -> windows_core::Result<VARIANT>;
    fn GetCachedPropertyValue(&self, propertyid: PROPERTYID) -> windows_core::Result<VARIANT>;
    fn GetCachedPropertyValueEx(
        &self,
        propertyid: PROPERTYID,
        ignoredefaultvalue: windows_core::BOOL,
    ) -> windows_core::Result<VARIANT>;
    fn GetCurrentPatternAs(
        &self,
        patternid: PATTERNID,
        riid: *const windows_core::GUID,
        patternobject: *mut *mut core::ffi::c_void,
    ) -> windows_core::Result<()>;
    fn GetCachedPatternAs(
        &self,
        patternid: PATTERNID,
        riid: *const windows_core::GUID,
        patternobject: *mut *mut core::ffi::c_void,
    ) -> windows_core::Result<()>;
    fn GetCurrentPattern(
        &self,
        patternid: PATTERNID,
    ) -> windows_core::Result<windows_core::IUnknown>;
    fn GetCachedPattern(
        &self,
        patternid: PATTERNID,
    ) -> windows_core::Result<windows_core::IUnknown>;
    fn GetCachedParent(&self) -> windows_core::Result<IUIAutomationElement>;
    fn GetCachedChildren(&self) -> windows_core::Result<IUIAutomationElementArray>;
    fn CurrentProcessId(&self) -> windows_core::Result<i32>;
    fn CurrentControlType(&self) -> windows_core::Result<CONTROLTYPEID>;
    fn CurrentLocalizedControlType(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CurrentName(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CurrentAcceleratorKey(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CurrentAccessKey(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CurrentHasKeyboardFocus(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CurrentIsKeyboardFocusable(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CurrentIsEnabled(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CurrentAutomationId(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CurrentClassName(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CurrentHelpText(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CurrentCulture(&self) -> windows_core::Result<i32>;
    fn CurrentIsControlElement(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CurrentIsContentElement(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CurrentIsPassword(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CurrentNativeWindowHandle(&self) -> windows_core::Result<UIA_HWND>;
    fn CurrentItemType(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CurrentIsOffscreen(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CurrentOrientation(&self) -> windows_core::Result<OrientationType>;
    fn CurrentFrameworkId(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CurrentIsRequiredForForm(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CurrentItemStatus(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CurrentBoundingRectangle(&self) -> windows_core::Result<RECT>;
    fn CurrentLabeledBy(&self) -> windows_core::Result<IUIAutomationElement>;
    fn CurrentAriaRole(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CurrentAriaProperties(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CurrentIsDataValidForForm(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CurrentControllerFor(&self) -> windows_core::Result<IUIAutomationElementArray>;
    fn CurrentDescribedBy(&self) -> windows_core::Result<IUIAutomationElementArray>;
    fn CurrentFlowsTo(&self) -> windows_core::Result<IUIAutomationElementArray>;
    fn CurrentProviderDescription(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CachedProcessId(&self) -> windows_core::Result<i32>;
    fn CachedControlType(&self) -> windows_core::Result<CONTROLTYPEID>;
    fn CachedLocalizedControlType(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CachedName(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CachedAcceleratorKey(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CachedAccessKey(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CachedHasKeyboardFocus(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CachedIsKeyboardFocusable(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CachedIsEnabled(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CachedAutomationId(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CachedClassName(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CachedHelpText(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CachedCulture(&self) -> windows_core::Result<i32>;
    fn CachedIsControlElement(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CachedIsContentElement(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CachedIsPassword(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CachedNativeWindowHandle(&self) -> windows_core::Result<UIA_HWND>;
    fn CachedItemType(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CachedIsOffscreen(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CachedOrientation(&self) -> windows_core::Result<OrientationType>;
    fn CachedFrameworkId(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CachedIsRequiredForForm(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CachedItemStatus(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CachedBoundingRectangle(&self) -> windows_core::Result<RECT>;
    fn CachedLabeledBy(&self) -> windows_core::Result<IUIAutomationElement>;
    fn CachedAriaRole(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CachedAriaProperties(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CachedIsDataValidForForm(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CachedControllerFor(&self) -> windows_core::Result<IUIAutomationElementArray>;
    fn CachedDescribedBy(&self) -> windows_core::Result<IUIAutomationElementArray>;
    fn CachedFlowsTo(&self) -> windows_core::Result<IUIAutomationElementArray>;
    fn CachedProviderDescription(&self) -> windows_core::Result<windows_core::BSTR>;
    fn GetClickablePoint(&self, clickable: *mut POINT) -> windows_core::Result<windows_core::BOOL>;
}
impl IUIAutomationElement_Vtbl {
    pub const fn new<Identity: IUIAutomationElement_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn SetFocus<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomationElement_Impl::SetFocus(this).into()
            }
        }
        unsafe extern "system" fn GetRuntimeId<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            runtimeid: *mut *mut SAFEARRAY,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::GetRuntimeId(this) {
                    Ok(ok__) => {
                        runtimeid.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn FindFirst<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            scope: TreeScope,
            condition: *mut core::ffi::c_void,
            found: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::FindFirst(
                    this,
                    core::mem::transmute_copy(&scope),
                    core::mem::transmute_copy(&condition),
                ) {
                    Ok(ok__) => {
                        found.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn FindAll<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            scope: TreeScope,
            condition: *mut core::ffi::c_void,
            found: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::FindAll(
                    this,
                    core::mem::transmute_copy(&scope),
                    core::mem::transmute_copy(&condition),
                ) {
                    Ok(ok__) => {
                        found.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn FindFirstBuildCache<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            scope: TreeScope,
            condition: *mut core::ffi::c_void,
            cacherequest: *mut core::ffi::c_void,
            found: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::FindFirstBuildCache(
                    this,
                    core::mem::transmute_copy(&scope),
                    core::mem::transmute_copy(&condition),
                    core::mem::transmute_copy(&cacherequest),
                ) {
                    Ok(ok__) => {
                        found.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn FindAllBuildCache<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            scope: TreeScope,
            condition: *mut core::ffi::c_void,
            cacherequest: *mut core::ffi::c_void,
            found: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::FindAllBuildCache(
                    this,
                    core::mem::transmute_copy(&scope),
                    core::mem::transmute_copy(&condition),
                    core::mem::transmute_copy(&cacherequest),
                ) {
                    Ok(ok__) => {
                        found.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn BuildUpdatedCache<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            cacherequest: *mut core::ffi::c_void,
            updatedelement: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::BuildUpdatedCache(
                    this,
                    core::mem::transmute_copy(&cacherequest),
                ) {
                    Ok(ok__) => {
                        updatedelement.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetCurrentPropertyValue<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            propertyid: PROPERTYID,
            retval: *mut VARIANT,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::GetCurrentPropertyValue(
                    this,
                    core::mem::transmute_copy(&propertyid),
                ) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetCurrentPropertyValueEx<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            propertyid: PROPERTYID,
            ignoredefaultvalue: windows_core::BOOL,
            retval: *mut VARIANT,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::GetCurrentPropertyValueEx(
                    this,
                    core::mem::transmute_copy(&propertyid),
                    core::mem::transmute_copy(&ignoredefaultvalue),
                ) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetCachedPropertyValue<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            propertyid: PROPERTYID,
            retval: *mut VARIANT,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::GetCachedPropertyValue(
                    this,
                    core::mem::transmute_copy(&propertyid),
                ) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetCachedPropertyValueEx<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            propertyid: PROPERTYID,
            ignoredefaultvalue: windows_core::BOOL,
            retval: *mut VARIANT,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::GetCachedPropertyValueEx(
                    this,
                    core::mem::transmute_copy(&propertyid),
                    core::mem::transmute_copy(&ignoredefaultvalue),
                ) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetCurrentPatternAs<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            patternid: PATTERNID,
            riid: *const windows_core::GUID,
            patternobject: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomationElement_Impl::GetCurrentPatternAs(
                    this,
                    core::mem::transmute_copy(&patternid),
                    core::mem::transmute_copy(&riid),
                    core::mem::transmute_copy(&patternobject),
                )
                .into()
            }
        }
        unsafe extern "system" fn GetCachedPatternAs<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            patternid: PATTERNID,
            riid: *const windows_core::GUID,
            patternobject: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomationElement_Impl::GetCachedPatternAs(
                    this,
                    core::mem::transmute_copy(&patternid),
                    core::mem::transmute_copy(&riid),
                    core::mem::transmute_copy(&patternobject),
                )
                .into()
            }
        }
        unsafe extern "system" fn GetCurrentPattern<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            patternid: PATTERNID,
            patternobject: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::GetCurrentPattern(
                    this,
                    core::mem::transmute_copy(&patternid),
                ) {
                    Ok(ok__) => {
                        patternobject.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetCachedPattern<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            patternid: PATTERNID,
            patternobject: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::GetCachedPattern(
                    this,
                    core::mem::transmute_copy(&patternid),
                ) {
                    Ok(ok__) => {
                        patternobject.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetCachedParent<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            parent: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::GetCachedParent(this) {
                    Ok(ok__) => {
                        parent.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetCachedChildren<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            children: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::GetCachedChildren(this) {
                    Ok(ok__) => {
                        children.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentProcessId<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut i32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentProcessId(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentControlType<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut CONTROLTYPEID,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentControlType(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentLocalizedControlType<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentLocalizedControlType(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentName<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentName(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentAcceleratorKey<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentAcceleratorKey(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentAccessKey<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentAccessKey(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentHasKeyboardFocus<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentHasKeyboardFocus(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentIsKeyboardFocusable<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentIsKeyboardFocusable(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentIsEnabled<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentIsEnabled(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentAutomationId<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentAutomationId(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentClassName<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentClassName(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentHelpText<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentHelpText(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentCulture<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut i32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentCulture(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentIsControlElement<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentIsControlElement(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentIsContentElement<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentIsContentElement(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentIsPassword<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentIsPassword(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentNativeWindowHandle<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut UIA_HWND,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentNativeWindowHandle(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentItemType<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentItemType(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentIsOffscreen<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentIsOffscreen(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentOrientation<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut OrientationType,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentOrientation(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentFrameworkId<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentFrameworkId(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentIsRequiredForForm<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentIsRequiredForForm(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentItemStatus<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentItemStatus(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentBoundingRectangle<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut RECT,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentBoundingRectangle(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentLabeledBy<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentLabeledBy(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentAriaRole<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentAriaRole(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentAriaProperties<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentAriaProperties(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentIsDataValidForForm<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentIsDataValidForForm(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentControllerFor<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentControllerFor(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentDescribedBy<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentDescribedBy(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentFlowsTo<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentFlowsTo(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentProviderDescription<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentProviderDescription(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedProcessId<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut i32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedProcessId(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedControlType<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut CONTROLTYPEID,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedControlType(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedLocalizedControlType<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedLocalizedControlType(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedName<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedName(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedAcceleratorKey<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedAcceleratorKey(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedAccessKey<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedAccessKey(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedHasKeyboardFocus<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedHasKeyboardFocus(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedIsKeyboardFocusable<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedIsKeyboardFocusable(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedIsEnabled<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedIsEnabled(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedAutomationId<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedAutomationId(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedClassName<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedClassName(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedHelpText<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedHelpText(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedCulture<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut i32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedCulture(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedIsControlElement<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedIsControlElement(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedIsContentElement<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedIsContentElement(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedIsPassword<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedIsPassword(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedNativeWindowHandle<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut UIA_HWND,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedNativeWindowHandle(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedItemType<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedItemType(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedIsOffscreen<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedIsOffscreen(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedOrientation<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut OrientationType,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedOrientation(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedFrameworkId<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedFrameworkId(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedIsRequiredForForm<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedIsRequiredForForm(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedItemStatus<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedItemStatus(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedBoundingRectangle<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut RECT,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedBoundingRectangle(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedLabeledBy<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedLabeledBy(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedAriaRole<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedAriaRole(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedAriaProperties<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedAriaProperties(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedIsDataValidForForm<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedIsDataValidForForm(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedControllerFor<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedControllerFor(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedDescribedBy<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedDescribedBy(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedFlowsTo<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedFlowsTo(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedProviderDescription<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedProviderDescription(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetClickablePoint<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            clickable: *mut POINT,
            gotclickable: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::GetClickablePoint(
                    this,
                    core::mem::transmute_copy(&clickable),
                ) {
                    Ok(ok__) => {
                        gotclickable.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            SetFocus: SetFocus::<Identity, OFFSET>,
            GetRuntimeId: GetRuntimeId::<Identity, OFFSET>,
            FindFirst: FindFirst::<Identity, OFFSET>,
            FindAll: FindAll::<Identity, OFFSET>,
            FindFirstBuildCache: FindFirstBuildCache::<Identity, OFFSET>,
            FindAllBuildCache: FindAllBuildCache::<Identity, OFFSET>,
            BuildUpdatedCache: BuildUpdatedCache::<Identity, OFFSET>,
            GetCurrentPropertyValue: GetCurrentPropertyValue::<Identity, OFFSET>,
            GetCurrentPropertyValueEx: GetCurrentPropertyValueEx::<Identity, OFFSET>,
            GetCachedPropertyValue: GetCachedPropertyValue::<Identity, OFFSET>,
            GetCachedPropertyValueEx: GetCachedPropertyValueEx::<Identity, OFFSET>,
            GetCurrentPatternAs: GetCurrentPatternAs::<Identity, OFFSET>,
            GetCachedPatternAs: GetCachedPatternAs::<Identity, OFFSET>,
            GetCurrentPattern: GetCurrentPattern::<Identity, OFFSET>,
            GetCachedPattern: GetCachedPattern::<Identity, OFFSET>,
            GetCachedParent: GetCachedParent::<Identity, OFFSET>,
            GetCachedChildren: GetCachedChildren::<Identity, OFFSET>,
            CurrentProcessId: CurrentProcessId::<Identity, OFFSET>,
            CurrentControlType: CurrentControlType::<Identity, OFFSET>,
            CurrentLocalizedControlType: CurrentLocalizedControlType::<Identity, OFFSET>,
            CurrentName: CurrentName::<Identity, OFFSET>,
            CurrentAcceleratorKey: CurrentAcceleratorKey::<Identity, OFFSET>,
            CurrentAccessKey: CurrentAccessKey::<Identity, OFFSET>,
            CurrentHasKeyboardFocus: CurrentHasKeyboardFocus::<Identity, OFFSET>,
            CurrentIsKeyboardFocusable: CurrentIsKeyboardFocusable::<Identity, OFFSET>,
            CurrentIsEnabled: CurrentIsEnabled::<Identity, OFFSET>,
            CurrentAutomationId: CurrentAutomationId::<Identity, OFFSET>,
            CurrentClassName: CurrentClassName::<Identity, OFFSET>,
            CurrentHelpText: CurrentHelpText::<Identity, OFFSET>,
            CurrentCulture: CurrentCulture::<Identity, OFFSET>,
            CurrentIsControlElement: CurrentIsControlElement::<Identity, OFFSET>,
            CurrentIsContentElement: CurrentIsContentElement::<Identity, OFFSET>,
            CurrentIsPassword: CurrentIsPassword::<Identity, OFFSET>,
            CurrentNativeWindowHandle: CurrentNativeWindowHandle::<Identity, OFFSET>,
            CurrentItemType: CurrentItemType::<Identity, OFFSET>,
            CurrentIsOffscreen: CurrentIsOffscreen::<Identity, OFFSET>,
            CurrentOrientation: CurrentOrientation::<Identity, OFFSET>,
            CurrentFrameworkId: CurrentFrameworkId::<Identity, OFFSET>,
            CurrentIsRequiredForForm: CurrentIsRequiredForForm::<Identity, OFFSET>,
            CurrentItemStatus: CurrentItemStatus::<Identity, OFFSET>,
            CurrentBoundingRectangle: CurrentBoundingRectangle::<Identity, OFFSET>,
            CurrentLabeledBy: CurrentLabeledBy::<Identity, OFFSET>,
            CurrentAriaRole: CurrentAriaRole::<Identity, OFFSET>,
            CurrentAriaProperties: CurrentAriaProperties::<Identity, OFFSET>,
            CurrentIsDataValidForForm: CurrentIsDataValidForForm::<Identity, OFFSET>,
            CurrentControllerFor: CurrentControllerFor::<Identity, OFFSET>,
            CurrentDescribedBy: CurrentDescribedBy::<Identity, OFFSET>,
            CurrentFlowsTo: CurrentFlowsTo::<Identity, OFFSET>,
            CurrentProviderDescription: CurrentProviderDescription::<Identity, OFFSET>,
            CachedProcessId: CachedProcessId::<Identity, OFFSET>,
            CachedControlType: CachedControlType::<Identity, OFFSET>,
            CachedLocalizedControlType: CachedLocalizedControlType::<Identity, OFFSET>,
            CachedName: CachedName::<Identity, OFFSET>,
            CachedAcceleratorKey: CachedAcceleratorKey::<Identity, OFFSET>,
            CachedAccessKey: CachedAccessKey::<Identity, OFFSET>,
            CachedHasKeyboardFocus: CachedHasKeyboardFocus::<Identity, OFFSET>,
            CachedIsKeyboardFocusable: CachedIsKeyboardFocusable::<Identity, OFFSET>,
            CachedIsEnabled: CachedIsEnabled::<Identity, OFFSET>,
            CachedAutomationId: CachedAutomationId::<Identity, OFFSET>,
            CachedClassName: CachedClassName::<Identity, OFFSET>,
            CachedHelpText: CachedHelpText::<Identity, OFFSET>,
            CachedCulture: CachedCulture::<Identity, OFFSET>,
            CachedIsControlElement: CachedIsControlElement::<Identity, OFFSET>,
            CachedIsContentElement: CachedIsContentElement::<Identity, OFFSET>,
            CachedIsPassword: CachedIsPassword::<Identity, OFFSET>,
            CachedNativeWindowHandle: CachedNativeWindowHandle::<Identity, OFFSET>,
            CachedItemType: CachedItemType::<Identity, OFFSET>,
            CachedIsOffscreen: CachedIsOffscreen::<Identity, OFFSET>,
            CachedOrientation: CachedOrientation::<Identity, OFFSET>,
            CachedFrameworkId: CachedFrameworkId::<Identity, OFFSET>,
            CachedIsRequiredForForm: CachedIsRequiredForForm::<Identity, OFFSET>,
            CachedItemStatus: CachedItemStatus::<Identity, OFFSET>,
            CachedBoundingRectangle: CachedBoundingRectangle::<Identity, OFFSET>,
            CachedLabeledBy: CachedLabeledBy::<Identity, OFFSET>,
            CachedAriaRole: CachedAriaRole::<Identity, OFFSET>,
            CachedAriaProperties: CachedAriaProperties::<Identity, OFFSET>,
            CachedIsDataValidForForm: CachedIsDataValidForForm::<Identity, OFFSET>,
            CachedControllerFor: CachedControllerFor::<Identity, OFFSET>,
            CachedDescribedBy: CachedDescribedBy::<Identity, OFFSET>,
            CachedFlowsTo: CachedFlowsTo::<Identity, OFFSET>,
            CachedProviderDescription: CachedProviderDescription::<Identity, OFFSET>,
            GetClickablePoint: GetClickablePoint::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IUIAutomationElement as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IUIAutomationElement {}
windows_core::imp::define_interface!(
    IUIAutomationElementArray,
    IUIAutomationElementArray_Vtbl,
    0x14314595_b4bc_4055_95f2_58f2e42c9855
);
windows_core::imp::interface_hierarchy!(IUIAutomationElementArray, windows_core::IUnknown);
impl IUIAutomationElementArray {
    pub(crate) unsafe fn Length(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Length)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn GetElement(
        &self,
        index: i32,
    ) -> windows_core::Result<IUIAutomationElement> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetElement)(
                windows_core::Interface::as_raw(self),
                index,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IUIAutomationElementArray_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub Length:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub GetElement: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        i32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
pub trait IUIAutomationElementArray_Impl: windows_core::IUnknownImpl {
    fn Length(&self) -> windows_core::Result<i32>;
    fn GetElement(&self, index: i32) -> windows_core::Result<IUIAutomationElement>;
}
impl IUIAutomationElementArray_Vtbl {
    pub const fn new<Identity: IUIAutomationElementArray_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn Length<
            Identity: IUIAutomationElementArray_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            length: *mut i32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElementArray_Impl::Length(this) {
                    Ok(ok__) => {
                        length.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetElement<
            Identity: IUIAutomationElementArray_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            index: i32,
            element: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElementArray_Impl::GetElement(
                    this,
                    core::mem::transmute_copy(&index),
                ) {
                    Ok(ok__) => {
                        element.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            Length: Length::<Identity, OFFSET>,
            GetElement: GetElement::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IUIAutomationElementArray as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IUIAutomationElementArray {}
windows_core::imp::define_interface!(
    IUIAutomationEventHandler,
    IUIAutomationEventHandler_Vtbl,
    0x146c3c17_f12e_4e22_8c27_f894b9b79c69
);
windows_core::imp::interface_hierarchy!(IUIAutomationEventHandler, windows_core::IUnknown);
#[repr(C)]
pub struct IUIAutomationEventHandler_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    HandleAutomationEvent: usize,
}
impl windows_core::RuntimeName for IUIAutomationEventHandler {}
windows_core::imp::define_interface!(
    IUIAutomationFocusChangedEventHandler,
    IUIAutomationFocusChangedEventHandler_Vtbl,
    0xc270f6b5_5c69_4290_9745_7a7f97169468
);
windows_core::imp::interface_hierarchy!(
    IUIAutomationFocusChangedEventHandler,
    windows_core::IUnknown
);
#[repr(C)]
pub struct IUIAutomationFocusChangedEventHandler_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    HandleFocusChangedEvent: usize,
}
impl windows_core::RuntimeName for IUIAutomationFocusChangedEventHandler {}
windows_core::imp::define_interface!(
    IUIAutomationInvokePattern,
    IUIAutomationInvokePattern_Vtbl,
    0xfb377fbe_8ea6_46d5_9c73_6499642d3059
);
windows_core::imp::interface_hierarchy!(IUIAutomationInvokePattern, windows_core::IUnknown);
impl IUIAutomationInvokePattern {
    pub(crate) unsafe fn Invoke(&self) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).Invoke)(windows_core::Interface::as_raw(self))
        }
    }
}
#[repr(C)]
pub struct IUIAutomationInvokePattern_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub Invoke: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
}
pub trait IUIAutomationInvokePattern_Impl: windows_core::IUnknownImpl {
    fn Invoke(&self) -> windows_core::Result<()>;
}
impl IUIAutomationInvokePattern_Vtbl {
    pub const fn new<Identity: IUIAutomationInvokePattern_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn Invoke<
            Identity: IUIAutomationInvokePattern_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomationInvokePattern_Impl::Invoke(this).into()
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            Invoke: Invoke::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IUIAutomationInvokePattern as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IUIAutomationInvokePattern {}
windows_core::imp::define_interface!(
    IUIAutomationPropertyChangedEventHandler,
    IUIAutomationPropertyChangedEventHandler_Vtbl,
    0x40cd37d4_c756_4b0c_8c6f_bddfeeb13b50
);
windows_core::imp::interface_hierarchy!(
    IUIAutomationPropertyChangedEventHandler,
    windows_core::IUnknown
);
#[repr(C)]
pub struct IUIAutomationPropertyChangedEventHandler_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    HandlePropertyChangedEvent: usize,
}
impl windows_core::RuntimeName for IUIAutomationPropertyChangedEventHandler {}
windows_core::imp::define_interface!(
    IUIAutomationProxyFactory,
    IUIAutomationProxyFactory_Vtbl,
    0x85b94ecd_849d_42b6_b94d_d6db23fdf5a4
);
windows_core::imp::interface_hierarchy!(IUIAutomationProxyFactory, windows_core::IUnknown);
#[repr(C)]
pub struct IUIAutomationProxyFactory_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    CreateProvider: usize,
    ProxyFactoryId: usize,
}
impl windows_core::RuntimeName for IUIAutomationProxyFactory {}
windows_core::imp::define_interface!(
    IUIAutomationProxyFactoryEntry,
    IUIAutomationProxyFactoryEntry_Vtbl,
    0xd50e472e_b64b_490c_bca1_d30696f9f289
);
windows_core::imp::interface_hierarchy!(IUIAutomationProxyFactoryEntry, windows_core::IUnknown);
#[repr(C)]
pub struct IUIAutomationProxyFactoryEntry_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    ProxyFactory: usize,
    ClassName: usize,
    ImageName: usize,
    AllowSubstringMatch: usize,
    CanCheckBaseClass: usize,
    NeedsAdviseEvents: usize,
    SetClassName: usize,
    SetImageName: usize,
    SetAllowSubstringMatch: usize,
    SetCanCheckBaseClass: usize,
    SetNeedsAdviseEvents: usize,
    SetWinEventsForAutomationEvent: usize,
    GetWinEventsForAutomationEvent: usize,
}
impl windows_core::RuntimeName for IUIAutomationProxyFactoryEntry {}
windows_core::imp::define_interface!(
    IUIAutomationProxyFactoryMapping,
    IUIAutomationProxyFactoryMapping_Vtbl,
    0x09e31e18_872d_4873_93d1_1e541ec133fd
);
windows_core::imp::interface_hierarchy!(IUIAutomationProxyFactoryMapping, windows_core::IUnknown);
#[repr(C)]
pub struct IUIAutomationProxyFactoryMapping_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    Count: usize,
    GetTable: usize,
    GetEntry: usize,
    SetTable: usize,
    InsertEntries: usize,
    InsertEntry: usize,
    RemoveEntry: usize,
    ClearTable: usize,
    RestoreDefaultTable: usize,
}
impl windows_core::RuntimeName for IUIAutomationProxyFactoryMapping {}
windows_core::imp::define_interface!(
    IUIAutomationStructureChangedEventHandler,
    IUIAutomationStructureChangedEventHandler_Vtbl,
    0xe81d1b4e_11c5_42f8_9754_e7036c79f054
);
windows_core::imp::interface_hierarchy!(
    IUIAutomationStructureChangedEventHandler,
    windows_core::IUnknown
);
#[repr(C)]
pub struct IUIAutomationStructureChangedEventHandler_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    HandleStructureChangedEvent: usize,
}
impl windows_core::RuntimeName for IUIAutomationStructureChangedEventHandler {}
windows_core::imp::define_interface!(
    IUIAutomationTogglePattern,
    IUIAutomationTogglePattern_Vtbl,
    0x94cf8058_9b8d_4ab9_8bfd_4cd0a33c8c70
);
windows_core::imp::interface_hierarchy!(IUIAutomationTogglePattern, windows_core::IUnknown);
impl IUIAutomationTogglePattern {
    pub(crate) unsafe fn Toggle(&self) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).Toggle)(windows_core::Interface::as_raw(self))
        }
    }
    pub(crate) unsafe fn CurrentToggleState(&self) -> windows_core::Result<ToggleState> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentToggleState)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedToggleState(&self) -> windows_core::Result<ToggleState> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedToggleState)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
}
#[repr(C)]
pub struct IUIAutomationTogglePattern_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub Toggle: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub CurrentToggleState: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut ToggleState,
    ) -> windows_core::HRESULT,
    pub CachedToggleState: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut ToggleState,
    ) -> windows_core::HRESULT,
}
pub trait IUIAutomationTogglePattern_Impl: windows_core::IUnknownImpl {
    fn Toggle(&self) -> windows_core::Result<()>;
    fn CurrentToggleState(&self) -> windows_core::Result<ToggleState>;
    fn CachedToggleState(&self) -> windows_core::Result<ToggleState>;
}
impl IUIAutomationTogglePattern_Vtbl {
    pub const fn new<Identity: IUIAutomationTogglePattern_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn Toggle<
            Identity: IUIAutomationTogglePattern_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomationTogglePattern_Impl::Toggle(this).into()
            }
        }
        unsafe extern "system" fn CurrentToggleState<
            Identity: IUIAutomationTogglePattern_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut ToggleState,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationTogglePattern_Impl::CurrentToggleState(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedToggleState<
            Identity: IUIAutomationTogglePattern_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut ToggleState,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationTogglePattern_Impl::CachedToggleState(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            Toggle: Toggle::<Identity, OFFSET>,
            CurrentToggleState: CurrentToggleState::<Identity, OFFSET>,
            CachedToggleState: CachedToggleState::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IUIAutomationTogglePattern as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IUIAutomationTogglePattern {}
windows_core::imp::define_interface!(
    IUIAutomationTreeWalker,
    IUIAutomationTreeWalker_Vtbl,
    0x4042c624_389c_4afc_a630_9df854a541fc
);
windows_core::imp::interface_hierarchy!(IUIAutomationTreeWalker, windows_core::IUnknown);
#[repr(C)]
pub struct IUIAutomationTreeWalker_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetParentElement: usize,
    GetFirstChildElement: usize,
    GetLastChildElement: usize,
    GetNextSiblingElement: usize,
    GetPreviousSiblingElement: usize,
    NormalizeElement: usize,
    GetParentElementBuildCache: usize,
    GetFirstChildElementBuildCache: usize,
    GetLastChildElementBuildCache: usize,
    GetNextSiblingElementBuildCache: usize,
    GetPreviousSiblingElementBuildCache: usize,
    NormalizeElementBuildCache: usize,
    Condition: usize,
}
impl windows_core::RuntimeName for IUIAutomationTreeWalker {}
windows_core::imp::define_interface!(
    IUIAutomationValuePattern,
    IUIAutomationValuePattern_Vtbl,
    0xa94cd8b1_0844_4cd6_9d2d_640537ab39e9
);
windows_core::imp::interface_hierarchy!(IUIAutomationValuePattern, windows_core::IUnknown);
impl IUIAutomationValuePattern {
    pub(crate) unsafe fn SetValue(&self, val: &windows_core::BSTR) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).SetValue)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(val),
            )
        }
    }
    pub(crate) unsafe fn CurrentValue(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentValue)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CurrentIsReadOnly(&self) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentIsReadOnly)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedValue(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedValue)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CachedIsReadOnly(&self) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedIsReadOnly)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
}
#[repr(C)]
pub struct IUIAutomationValuePattern_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub SetValue: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentValue: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentIsReadOnly: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CachedValue: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedIsReadOnly: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
}
pub trait IUIAutomationValuePattern_Impl: windows_core::IUnknownImpl {
    fn SetValue(&self, val: &windows_core::BSTR) -> windows_core::Result<()>;
    fn CurrentValue(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CurrentIsReadOnly(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CachedValue(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CachedIsReadOnly(&self) -> windows_core::Result<windows_core::BOOL>;
}
impl IUIAutomationValuePattern_Vtbl {
    pub const fn new<Identity: IUIAutomationValuePattern_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn SetValue<
            Identity: IUIAutomationValuePattern_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            val: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomationValuePattern_Impl::SetValue(this, core::mem::transmute(&val)).into()
            }
        }
        unsafe extern "system" fn CurrentValue<
            Identity: IUIAutomationValuePattern_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationValuePattern_Impl::CurrentValue(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentIsReadOnly<
            Identity: IUIAutomationValuePattern_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationValuePattern_Impl::CurrentIsReadOnly(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedValue<
            Identity: IUIAutomationValuePattern_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationValuePattern_Impl::CachedValue(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedIsReadOnly<
            Identity: IUIAutomationValuePattern_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationValuePattern_Impl::CachedIsReadOnly(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            SetValue: SetValue::<Identity, OFFSET>,
            CurrentValue: CurrentValue::<Identity, OFFSET>,
            CurrentIsReadOnly: CurrentIsReadOnly::<Identity, OFFSET>,
            CachedValue: CachedValue::<Identity, OFFSET>,
            CachedIsReadOnly: CachedIsReadOnly::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IUIAutomationValuePattern as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IUIAutomationValuePattern {}
windows_core::imp::define_interface!(
    IUIElement,
    IUIElement_Vtbl,
    0xc3c01020_320c_5cf6_9d24_d396bbfa4d8b
);
impl windows_core::RuntimeType for IUIElement {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IUIElement {
    pub(crate) fn ActualSize(&self) -> windows_core::Result<windows_numerics::Vector2> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ActualSize)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn TransformToVisual<P0>(&self, visual: P0) -> windows_core::Result<GeneralTransform>
    where
        P0: windows_core::Param<UIElement>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TransformToVisual)(
                windows_core::Interface::as_raw(self),
                visual.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IUIElement_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    DesiredSize: usize,
    AllowDrop: usize,
    SetAllowDrop: usize,
    Opacity: usize,
    SetOpacity: usize,
    Clip: usize,
    SetClip: usize,
    RenderTransform: usize,
    SetRenderTransform: usize,
    Projection: usize,
    SetProjection: usize,
    Transform3D: usize,
    SetTransform3D: usize,
    RenderTransformOrigin: usize,
    SetRenderTransformOrigin: usize,
    IsHitTestVisible: usize,
    SetIsHitTestVisible: usize,
    Visibility: usize,
    SetVisibility: usize,
    RenderSize: usize,
    UseLayoutRounding: usize,
    SetUseLayoutRounding: usize,
    Transitions: usize,
    SetTransitions: usize,
    CacheMode: usize,
    SetCacheMode: usize,
    IsTapEnabled: usize,
    SetIsTapEnabled: usize,
    IsDoubleTapEnabled: usize,
    SetIsDoubleTapEnabled: usize,
    CanDrag: usize,
    SetCanDrag: usize,
    IsRightTapEnabled: usize,
    SetIsRightTapEnabled: usize,
    IsHoldingEnabled: usize,
    SetIsHoldingEnabled: usize,
    ManipulationMode: usize,
    SetManipulationMode: usize,
    PointerCaptures: usize,
    ContextFlyout: usize,
    SetContextFlyout: usize,
    CompositeMode: usize,
    SetCompositeMode: usize,
    Lights: usize,
    CanBeScrollAnchor: usize,
    SetCanBeScrollAnchor: usize,
    ExitDisplayModeOnAccessKeyInvoked: usize,
    SetExitDisplayModeOnAccessKeyInvoked: usize,
    IsAccessKeyScope: usize,
    SetIsAccessKeyScope: usize,
    AccessKeyScopeOwner: usize,
    SetAccessKeyScopeOwner: usize,
    AccessKey: usize,
    SetAccessKey: usize,
    KeyTipPlacementMode: usize,
    SetKeyTipPlacementMode: usize,
    KeyTipHorizontalOffset: usize,
    SetKeyTipHorizontalOffset: usize,
    KeyTipVerticalOffset: usize,
    SetKeyTipVerticalOffset: usize,
    KeyTipTarget: usize,
    SetKeyTipTarget: usize,
    XYFocusKeyboardNavigation: usize,
    SetXYFocusKeyboardNavigation: usize,
    XYFocusUpNavigationStrategy: usize,
    SetXYFocusUpNavigationStrategy: usize,
    XYFocusDownNavigationStrategy: usize,
    SetXYFocusDownNavigationStrategy: usize,
    XYFocusLeftNavigationStrategy: usize,
    SetXYFocusLeftNavigationStrategy: usize,
    XYFocusRightNavigationStrategy: usize,
    SetXYFocusRightNavigationStrategy: usize,
    KeyboardAccelerators: usize,
    KeyboardAcceleratorPlacementTarget: usize,
    SetKeyboardAcceleratorPlacementTarget: usize,
    KeyboardAcceleratorPlacementMode: usize,
    SetKeyboardAcceleratorPlacementMode: usize,
    HighContrastAdjustment: usize,
    SetHighContrastAdjustment: usize,
    TabFocusNavigation: usize,
    SetTabFocusNavigation: usize,
    OpacityTransition: usize,
    SetOpacityTransition: usize,
    Translation: usize,
    SetTranslation: usize,
    TranslationTransition: usize,
    SetTranslationTransition: usize,
    Rotation: usize,
    SetRotation: usize,
    RotationTransition: usize,
    SetRotationTransition: usize,
    Scale: usize,
    SetScale: usize,
    ScaleTransition: usize,
    SetScaleTransition: usize,
    TransformMatrix: usize,
    SetTransformMatrix: usize,
    CenterPoint: usize,
    SetCenterPoint: usize,
    RotationAxis: usize,
    SetRotationAxis: usize,
    ActualOffset: usize,
    pub ActualSize: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_numerics::Vector2,
    ) -> windows_core::HRESULT,
    XamlRoot: usize,
    SetXamlRoot: usize,
    Shadow: usize,
    SetShadow: usize,
    RasterizationScale: usize,
    SetRasterizationScale: usize,
    FocusState: usize,
    UseSystemFocusVisuals: usize,
    SetUseSystemFocusVisuals: usize,
    XYFocusLeft: usize,
    SetXYFocusLeft: usize,
    XYFocusRight: usize,
    SetXYFocusRight: usize,
    XYFocusUp: usize,
    SetXYFocusUp: usize,
    XYFocusDown: usize,
    SetXYFocusDown: usize,
    IsTabStop: usize,
    SetIsTabStop: usize,
    TabIndex: usize,
    SetTabIndex: usize,
    KeyUp: usize,
    RemoveKeyUp: usize,
    KeyDown: usize,
    RemoveKeyDown: usize,
    GotFocus: usize,
    RemoveGotFocus: usize,
    LostFocus: usize,
    RemoveLostFocus: usize,
    DragStarting: usize,
    RemoveDragStarting: usize,
    DropCompleted: usize,
    RemoveDropCompleted: usize,
    CharacterReceived: usize,
    RemoveCharacterReceived: usize,
    DragEnter: usize,
    RemoveDragEnter: usize,
    DragLeave: usize,
    RemoveDragLeave: usize,
    DragOver: usize,
    RemoveDragOver: usize,
    Drop: usize,
    RemoveDrop: usize,
    PointerPressed: usize,
    RemovePointerPressed: usize,
    PointerMoved: usize,
    RemovePointerMoved: usize,
    PointerReleased: usize,
    RemovePointerReleased: usize,
    PointerEntered: usize,
    RemovePointerEntered: usize,
    PointerExited: usize,
    RemovePointerExited: usize,
    PointerCaptureLost: usize,
    RemovePointerCaptureLost: usize,
    PointerCanceled: usize,
    RemovePointerCanceled: usize,
    PointerWheelChanged: usize,
    RemovePointerWheelChanged: usize,
    Tapped: usize,
    RemoveTapped: usize,
    DoubleTapped: usize,
    RemoveDoubleTapped: usize,
    Holding: usize,
    RemoveHolding: usize,
    ContextRequested: usize,
    RemoveContextRequested: usize,
    ContextCanceled: usize,
    RemoveContextCanceled: usize,
    RightTapped: usize,
    RemoveRightTapped: usize,
    ManipulationStarting: usize,
    RemoveManipulationStarting: usize,
    ManipulationInertiaStarting: usize,
    RemoveManipulationInertiaStarting: usize,
    ManipulationStarted: usize,
    RemoveManipulationStarted: usize,
    ManipulationDelta: usize,
    RemoveManipulationDelta: usize,
    ManipulationCompleted: usize,
    RemoveManipulationCompleted: usize,
    AccessKeyDisplayRequested: usize,
    RemoveAccessKeyDisplayRequested: usize,
    AccessKeyDisplayDismissed: usize,
    RemoveAccessKeyDisplayDismissed: usize,
    AccessKeyInvoked: usize,
    RemoveAccessKeyInvoked: usize,
    ProcessKeyboardAccelerators: usize,
    RemoveProcessKeyboardAccelerators: usize,
    GettingFocus: usize,
    RemoveGettingFocus: usize,
    LosingFocus: usize,
    RemoveLosingFocus: usize,
    NoFocusCandidateFound: usize,
    RemoveNoFocusCandidateFound: usize,
    PreviewKeyDown: usize,
    RemovePreviewKeyDown: usize,
    PreviewKeyUp: usize,
    RemovePreviewKeyUp: usize,
    BringIntoViewRequested: usize,
    RemoveBringIntoViewRequested: usize,
    Measure: usize,
    Arrange: usize,
    CapturePointer: usize,
    ReleasePointerCapture: usize,
    ReleasePointerCaptures: usize,
    AddHandler: usize,
    RemoveHandler: usize,
    pub TransformToVisual: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct KEYBDINPUT {
    pub wVk: u16,
    pub wScan: u16,
    pub dwFlags: u32,
    pub time: u32,
    pub dwExtraInfo: usize,
}
pub const KEYEVENTF_KEYUP: i32 = 2;
pub const KEYEVENTF_UNICODE: i32 = 4;
#[repr(C)]
#[cfg(target_arch = "x86")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct KNONVOLATILE_CONTEXT_POINTERS {
    pub Dummy: u32,
}
#[repr(C)]
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
#[derive(Clone, Copy)]
pub struct KNONVOLATILE_CONTEXT_POINTERS {
    pub Anonymous: KNONVOLATILE_CONTEXT_POINTERS_0,
    pub Anonymous2: KNONVOLATILE_CONTEXT_POINTERS_1,
}
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
impl Default for KNONVOLATILE_CONTEXT_POINTERS {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
#[derive(Clone, Copy)]
pub union KNONVOLATILE_CONTEXT_POINTERS_0 {
    pub FloatingContext: [PM128A; 16],
    pub Anonymous: KNONVOLATILE_CONTEXT_POINTERS_0_0,
}
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
impl Default for KNONVOLATILE_CONTEXT_POINTERS_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct KNONVOLATILE_CONTEXT_POINTERS_0_0 {
    pub Xmm0: PM128A,
    pub Xmm1: PM128A,
    pub Xmm2: PM128A,
    pub Xmm3: PM128A,
    pub Xmm4: PM128A,
    pub Xmm5: PM128A,
    pub Xmm6: PM128A,
    pub Xmm7: PM128A,
    pub Xmm8: PM128A,
    pub Xmm9: PM128A,
    pub Xmm10: PM128A,
    pub Xmm11: PM128A,
    pub Xmm12: PM128A,
    pub Xmm13: PM128A,
    pub Xmm14: PM128A,
    pub Xmm15: PM128A,
}
#[repr(C)]
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
#[derive(Clone, Copy)]
pub union KNONVOLATILE_CONTEXT_POINTERS_1 {
    pub IntegerContext: [PDWORD64; 16],
    pub Anonymous: KNONVOLATILE_CONTEXT_POINTERS_1_0,
}
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
impl Default for KNONVOLATILE_CONTEXT_POINTERS_1 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct KNONVOLATILE_CONTEXT_POINTERS_1_0 {
    pub Rax: PDWORD64,
    pub Rcx: PDWORD64,
    pub Rdx: PDWORD64,
    pub Rbx: PDWORD64,
    pub Rsp: PDWORD64,
    pub Rbp: PDWORD64,
    pub Rsi: PDWORD64,
    pub Rdi: PDWORD64,
    pub R8: PDWORD64,
    pub R9: PDWORD64,
    pub R10: PDWORD64,
    pub R11: PDWORD64,
    pub R12: PDWORD64,
    pub R13: PDWORD64,
    pub R14: PDWORD64,
    pub R15: PDWORD64,
}
#[cfg(target_arch = "aarch64")]
pub type KNONVOLATILE_CONTEXT_POINTERS = KNONVOLATILE_CONTEXT_POINTERS_ARM64;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct KNONVOLATILE_CONTEXT_POINTERS_ARM64 {
    pub X19: PDWORD64,
    pub X20: PDWORD64,
    pub X21: PDWORD64,
    pub X22: PDWORD64,
    pub X23: PDWORD64,
    pub X24: PDWORD64,
    pub X25: PDWORD64,
    pub X26: PDWORD64,
    pub X27: PDWORD64,
    pub X28: PDWORD64,
    pub Fp: PDWORD64,
    pub Lr: PDWORD64,
    pub D8: PDWORD64,
    pub D9: PDWORD64,
    pub D10: PDWORD64,
    pub D11: PDWORD64,
    pub D12: PDWORD64,
    pub D13: PDWORD64,
    pub D14: PDWORD64,
    pub D15: PDWORD64,
}
pub type LPARAM = isize;
pub type LPCONTEXT = PCONTEXT;
pub type LRESULT = isize;
pub const LWA_ALPHA: i32 = 2;
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct M128A {
    pub Low: u64,
    pub High: i64,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MODULEENTRY32W {
    pub dwSize: u32,
    pub th32ModuleID: u32,
    pub th32ProcessID: u32,
    pub GlblcntUsage: u32,
    pub ProccntUsage: u32,
    pub modBaseAddr: *mut u8,
    pub modBaseSize: u32,
    pub hModule: HMODULE,
    pub szModule: [u16; 256],
    pub szExePath: [u16; 260],
}
impl Default for MODULEENTRY32W {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub const MOUSEEVENTF_ABSOLUTE: i32 = 32768;
pub const MOUSEEVENTF_LEFTDOWN: i32 = 2;
pub const MOUSEEVENTF_LEFTUP: i32 = 4;
pub const MOUSEEVENTF_MOVE: i32 = 1;
pub const MOUSEEVENTF_VIRTUALDESK: i32 = 16384;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MOUSEINPUT {
    pub dx: i32,
    pub dy: i32,
    pub mouseData: u32,
    pub dwFlags: u32,
    pub time: u32,
    pub dwExtraInfo: usize,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: u32,
    pub wParam: WPARAM,
    pub lParam: LPARAM,
    pub time: u32,
    pub pt: POINT,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MSLLHOOKSTRUCT {
    pub pt: POINT,
    pub mouseData: u32,
    pub flags: u32,
    pub time: u32,
    pub dwExtraInfo: usize,
}
#[cfg(target_arch = "aarch64")]
pub type NEON128 = ARM64_NT_NEON128;
pub type OrientationType = i32;
pub type PATTERNID = i32;
#[cfg(any(target_arch = "arm64ec", target_arch = "x86", target_arch = "x86_64"))]
pub type PCONTEXT = *mut CONTEXT;
#[cfg(target_arch = "aarch64")]
pub type PCONTEXT = *mut ARM64_NT_CONTEXT;
pub type PDWORD64 = *mut u64;
#[cfg(any(target_arch = "arm64ec", target_arch = "x86", target_arch = "x86_64"))]
pub type PEXCEPTION_ROUTINE = Option<
    unsafe extern "system" fn(
        exceptionrecord: *mut EXCEPTION_RECORD,
        establisherframe: *const core::ffi::c_void,
        contextrecord: *mut CONTEXT,
        dispatchercontext: *const core::ffi::c_void,
    ) -> EXCEPTION_DISPOSITION,
>;
#[cfg(target_arch = "aarch64")]
pub type PEXCEPTION_ROUTINE = Option<
    unsafe extern "system" fn(
        exceptionrecord: *mut EXCEPTION_RECORD,
        establisherframe: *const core::ffi::c_void,
        contextrecord: *mut ARM64_NT_CONTEXT,
        dispatchercontext: *const core::ffi::c_void,
    ) -> EXCEPTION_DISPOSITION,
>;
pub type PM128A = *mut M128A;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}
pub type PROPERTYID = i32;
pub type PTIMERAPCROUTINE = Option<
    unsafe extern "system" fn(
        lpargtocompletionroutine: *const core::ffi::c_void,
        dwtimerlowvalue: u32,
        dwtimerhighvalue: u32,
    ),
>;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
impl windows_core::imp::TypeKind for Point {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for Point {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"struct(Windows.Foundation.Point;f4;f4)");
}
pub type PropertyConditionFlags = i32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}
#[repr(C)]
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
#[derive(Clone, Copy)]
pub struct RUNTIME_FUNCTION {
    pub BeginAddress: u32,
    pub EndAddress: u32,
    pub Anonymous: RUNTIME_FUNCTION_0,
}
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
impl Default for RUNTIME_FUNCTION {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
#[derive(Clone, Copy)]
pub union RUNTIME_FUNCTION_0 {
    pub UnwindInfoAddress: u32,
    pub UnwindData: u32,
}
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
impl Default for RUNTIME_FUNCTION_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[cfg(target_arch = "aarch64")]
pub type RUNTIME_FUNCTION = ARM64_RUNTIME_FUNCTION;
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SAFEARRAY {
    pub cDims: u16,
    pub fFeatures: u16,
    pub cbElements: u32,
    pub cLocks: u32,
    pub pvData: *mut core::ffi::c_void,
    pub rgsabound: [SAFEARRAYBOUND; 1],
}
impl Default for SAFEARRAY {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SAFEARRAYBOUND {
    pub cElements: u32,
    pub lLbound: i32,
}
pub type SCODE = i32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SECURITY_ATTRIBUTES {
    pub nLength: u32,
    pub lpSecurityDescriptor: *mut core::ffi::c_void,
    pub bInheritHandle: windows_core::BOOL,
}
pub const SM_CXVIRTUALSCREEN: i32 = 78;
pub const SM_CYVIRTUALSCREEN: i32 = 79;
pub const SM_XVIRTUALSCREEN: i32 = 76;
pub const SM_YVIRTUALSCREEN: i32 = 77;
pub const SWP_NOACTIVATE: i32 = 16;
pub const SWP_SHOWWINDOW: i32 = 64;
pub const SW_HIDE: i32 = 0;
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SYMBOL_INFOW {
    pub SizeOfStruct: u32,
    pub TypeIndex: u32,
    pub Reserved: [u64; 2],
    pub Index: u32,
    pub Size: u32,
    pub ModBase: u64,
    pub Flags: u32,
    pub Value: u64,
    pub Address: u64,
    pub Register: u32,
    pub Scope: u32,
    pub Tag: u32,
    pub NameLen: u32,
    pub MaxNameLen: u32,
    pub Name: [u16; 1],
}
impl Default for SYMBOL_INFOW {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub const SYMFLAG_EXPORT: i32 = 512;
pub const SYMOPT_DEFERRED_LOADS: i32 = 4;
pub const SYMOPT_FAIL_CRITICAL_ERRORS: i32 = 512;
pub const SYMOPT_UNDNAME: i32 = 2;
pub const TH32CS_SNAPMODULE: i32 = 8;
pub const THREAD_GET_CONTEXT: i32 = 8;
pub const THREAD_PRIORITY_TIME_CRITICAL: i32 = 15;
pub const THREAD_QUERY_INFORMATION: i32 = 64;
pub const THREAD_SUSPEND_RESUME: i32 = 2;
pub const TIMER_ALL_ACCESS: i32 = 2031619;
pub type ToggleState = i32;
pub type TreeScope = i32;
pub type UIA_HWND = *mut core::ffi::c_void;
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UIElement(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    UIElement,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(UIElement, DependencyObject);
impl windows_core::RuntimeType for UIElement {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IUIElement>();
}
unsafe impl windows_core::Interface for UIElement {
    type Vtable = <IUIElement as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IUIElement as windows_core::Interface>::IID;
}
impl core::ops::Deref for UIElement {
    type Target = IUIElement;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for UIElement {
    const NAME: &'static str = "Microsoft.UI.Xaml.UIElement";
}
unsafe impl Send for UIElement {}
unsafe impl Sync for UIElement {}
pub const UNW_FLAG_NHANDLER: i32 = 0;
#[repr(C)]
pub struct VARIANT {
    pub Anonymous: VARIANT_0,
}
impl Clone for VARIANT {
    fn clone(&self) -> Self {
        unsafe { core::mem::transmute_copy(self) }
    }
}
impl Default for VARIANT {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
pub union VARIANT_0 {
    pub Anonymous: core::mem::ManuallyDrop<VARIANT_0_0>,
    pub decVal: DECIMAL,
}
impl Clone for VARIANT_0 {
    fn clone(&self) -> Self {
        unsafe { core::mem::transmute_copy(self) }
    }
}
impl Default for VARIANT_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
pub struct VARIANT_0_0 {
    pub vt: VARTYPE,
    pub wReserved1: u16,
    pub wReserved2: u16,
    pub wReserved3: u16,
    pub Anonymous: VARIANT_0_0_0,
}
impl Clone for VARIANT_0_0 {
    fn clone(&self) -> Self {
        unsafe { core::mem::transmute_copy(self) }
    }
}
impl Default for VARIANT_0_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
pub union VARIANT_0_0_0 {
    pub llVal: i64,
    pub lVal: i32,
    pub bVal: u8,
    pub iVal: i16,
    pub fltVal: f32,
    pub dblVal: f64,
    pub boolVal: VARIANT_BOOL,
    pub __OBSOLETE__VARIANT_BOOL: VARIANT_BOOL,
    pub scode: SCODE,
    pub cyVal: CY,
    pub date: f64,
    pub bstrVal: core::mem::ManuallyDrop<windows_core::BSTR>,
    pub punkVal: core::mem::ManuallyDrop<Option<windows_core::IUnknown>>,
    pub pdispVal: core::mem::ManuallyDrop<Option<IDispatch>>,
    pub parray: *mut SAFEARRAY,
    pub pbVal: *mut u8,
    pub piVal: *mut i16,
    pub plVal: *mut i32,
    pub pllVal: *mut i64,
    pub pfltVal: *mut f32,
    pub pdblVal: *mut f64,
    pub pboolVal: *mut VARIANT_BOOL,
    pub __OBSOLETE__VARIANT_PBOOL: *mut VARIANT_BOOL,
    pub pscode: *mut SCODE,
    pub pcyVal: *mut CY,
    pub pdate: *mut f64,
    pub pbstrVal: *mut windows_core::BSTR,
    pub ppunkVal: *mut Option<windows_core::IUnknown>,
    pub ppdispVal: *mut Option<IDispatch>,
    pub pparray: *mut *mut SAFEARRAY,
    pub pvarVal: *mut VARIANT,
    pub byref: *mut core::ffi::c_void,
    pub cVal: i8,
    pub uiVal: u16,
    pub ulVal: u32,
    pub ullVal: u64,
    pub intVal: i32,
    pub uintVal: u32,
    pub pdecVal: *mut DECIMAL,
    pub pcVal: *mut i8,
    pub puiVal: *mut u16,
    pub pulVal: *mut u32,
    pub pullVal: *mut u64,
    pub pintVal: *mut i32,
    pub puintVal: *mut u32,
    pub Anonymous: core::mem::ManuallyDrop<VARIANT_0_0_0_0>,
}
impl Clone for VARIANT_0_0_0 {
    fn clone(&self) -> Self {
        unsafe { core::mem::transmute_copy(self) }
    }
}
impl Default for VARIANT_0_0_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VARIANT_0_0_0_0 {
    pub pvRecord: *mut core::ffi::c_void,
    pub pRecInfo: core::mem::ManuallyDrop<Option<IRecordInfo>>,
}
pub type VARIANT_BOOL = i16;
pub type VARTYPE = u16;
pub const WH_MOUSE_LL: i32 = 14;
pub const WM_APP: i32 = 32768;
pub const WM_LBUTTONDOWN: i32 = 513;
pub const WM_LBUTTONUP: i32 = 514;
pub const WM_MOUSEMOVE: i32 = 512;
pub const WM_QUIT: i32 = 18;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct WNDCLASSW {
    pub style: u32,
    pub lpfnWndProc: WNDPROC,
    pub cbClsExtra: i32,
    pub cbWndExtra: i32,
    pub hInstance: HINSTANCE,
    pub hIcon: HICON,
    pub hCursor: HCURSOR,
    pub hbrBackground: HBRUSH,
    pub lpszMenuName: windows_core::PCWSTR,
    pub lpszClassName: windows_core::PCWSTR,
}
pub type WNDENUMPROC =
    Option<unsafe extern "system" fn(param0: HWND, param1: LPARAM) -> windows_core::BOOL>;
pub type WNDPROC = Option<
    unsafe extern "system" fn(param0: HWND, param1: u32, param2: WPARAM, param3: LPARAM) -> LRESULT,
>;
pub type WPARAM = usize;
pub const WS_EX_LAYERED: i32 = 524288;
pub const WS_EX_NOACTIVATE: i32 = 134217728;
pub const WS_EX_TOOLWINDOW: i32 = 128;
pub const WS_EX_TOPMOST: i32 = 8;
pub const WS_EX_TRANSPARENT: i32 = 32;
pub const WS_POPUP: u32 = 2147483648;
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
pub type XMM_SAVE_AREA32 = XSAVE_FORMAT;
#[repr(C)]
#[cfg(target_arch = "x86")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct XSAVE_FORMAT {
    pub ControlWord: u16,
    pub StatusWord: u16,
    pub TagWord: u8,
    pub Reserved1: u8,
    pub ErrorOpcode: u16,
    pub ErrorOffset: u32,
    pub ErrorSelector: u16,
    pub Reserved2: u16,
    pub DataOffset: u32,
    pub DataSelector: u16,
    pub Reserved3: u16,
    pub MxCsr: u32,
    pub MxCsr_Mask: u32,
    pub FloatRegisters: [M128A; 8],
    pub XmmRegisters: [M128A; 8],
    pub Reserved4: [u8; 224],
}
#[cfg(target_arch = "x86")]
impl Default for XSAVE_FORMAT {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct XSAVE_FORMAT {
    pub ControlWord: u16,
    pub StatusWord: u16,
    pub TagWord: u8,
    pub Reserved1: u8,
    pub ErrorOpcode: u16,
    pub ErrorOffset: u32,
    pub ErrorSelector: u16,
    pub Reserved2: u16,
    pub DataOffset: u32,
    pub DataSelector: u16,
    pub Reserved3: u16,
    pub MxCsr: u32,
    pub MxCsr_Mask: u32,
    pub FloatRegisters: [M128A; 8],
    pub XmmRegisters: [M128A; 16],
    pub Reserved4: [u8; 96],
}
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
impl Default for XSAVE_FORMAT {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
pub type _IMAGE_RUNTIME_FUNCTION_ENTRY = RUNTIME_FUNCTION;
#[repr(C)]
#[cfg(any(target_arch = "aarch64", target_arch = "x86"))]
#[derive(Clone, Copy)]
pub struct _IMAGE_RUNTIME_FUNCTION_ENTRY {
    pub BeginAddress: u32,
    pub EndAddress: u32,
    pub Anonymous: _IMAGE_RUNTIME_FUNCTION_ENTRY_0,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86"))]
impl Default for _IMAGE_RUNTIME_FUNCTION_ENTRY {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(any(target_arch = "aarch64", target_arch = "x86"))]
#[derive(Clone, Copy)]
pub union _IMAGE_RUNTIME_FUNCTION_ENTRY_0 {
    pub UnwindInfoAddress: u32,
    pub UnwindData: u32,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86"))]
impl Default for _IMAGE_RUNTIME_FUNCTION_ENTRY_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
