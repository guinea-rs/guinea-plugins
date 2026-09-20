windows_core::link!("user32.dll" "system" fn CallNextHookEx(hhk : HHOOK, ncode : i32, wparam : WPARAM, lparam : LPARAM) -> LRESULT);
windows_core::link!("user32.dll" "system" fn ClientToScreen(hwnd : HWND, lppoint : *mut POINT) -> windows_core::BOOL);
windows_core::link!("kernel32.dll" "system" fn CloseHandle(hobject : HANDLE) -> windows_core::BOOL);
windows_core::link!("advapi32.dll" "system" fn CloseTrace(tracehandle : PROCESSTRACE_HANDLE) -> u32);
windows_core::link!("ole32.dll" "system" fn CoTaskMemFree(pv : *mut core::ffi::c_void));
windows_core::link!("advapi32.dll" "system" fn ControlTraceW(traceid : CONTROLTRACE_ID, instancename : windows_core::PCWSTR, properties : *mut EVENT_TRACE_PROPERTIES, controlcode : u32) -> u32);
windows_core::link!("gdi32.dll" "system" fn CreateSolidBrush(color : COLORREF) -> HBRUSH);
windows_core::link!("kernel32.dll" "system" fn CreateToolhelp32Snapshot(dwflags : u32, th32processid : u32) -> HANDLE);
windows_core::link!("user32.dll" "system" fn CreateWindowExW(dwexstyle : u32, lpclassname : windows_core::PCWSTR, lpwindowname : windows_core::PCWSTR, dwstyle : u32, x : i32, y : i32, nwidth : i32, nheight : i32, hwndparent : HWND, hmenu : HMENU, hinstance : HINSTANCE, lpparam : *const core::ffi::c_void) -> HWND);
windows_core::link!("user32.dll" "system" fn DefWindowProcW(hwnd : HWND, msg : u32, wparam : WPARAM, lparam : LPARAM) -> LRESULT);
windows_core::link!("user32.dll" "system" fn DestroyWindow(hwnd : HWND) -> windows_core::BOOL);
windows_core::link!("advapi32.dll" "system" fn EnableTraceEx2(traceid : CONTROLTRACE_ID, providerid : *const windows_core::GUID, controlcode : u32, level : u8, matchanykeyword : u64, matchallkeyword : u64, timeout : u32, enableparameters : *const ENABLE_TRACE_PARAMETERS) -> u32);
windows_core::link!("user32.dll" "system" fn EnumThreadWindows(dwthreadid : u32, lpfn : WNDENUMPROC, lparam : LPARAM) -> windows_core::BOOL);
windows_core::link!("user32.dll" "system" fn GetAncestor(hwnd : HWND, gaflags : u32) -> HWND);
windows_core::link!("kernel32.dll" "system" fn GetCurrentThreadId() -> u32);
windows_core::link!("user32.dll" "system" fn GetCursorPos(lppoint : *mut POINT) -> windows_core::BOOL);
windows_core::link!("user32.dll" "system" fn GetDpiForWindow(hwnd : HWND) -> u32);
windows_core::link!("user32.dll" "system" fn GetMessageW(lpmsg : *mut MSG, hwnd : HWND, wmsgfiltermin : u32, wmsgfiltermax : u32) -> windows_core::BOOL);
windows_core::link!("kernel32.dll" "system" fn GetModuleHandleW(lpmodulename : windows_core::PCWSTR) -> HMODULE);
windows_core::link!("kernel32.dll" "system" fn GetProcAddress(hmodule : HMODULE, lpprocname : windows_core::PCSTR) -> FARPROC);
windows_core::link!("user32.dll" "system" fn GetWindowLongW(hwnd : HWND, nindex : i32) -> i32);
windows_core::link!("user32.dll" "system" fn GetWindowThreadProcessId(hwnd : HWND, lpdwprocessid : *mut u32) -> u32);
windows_core::link!("user32.dll" "system" fn IsWindowVisible(hwnd : HWND) -> windows_core::BOOL);
windows_core::link!("kernel32.dll" "system" fn LoadLibraryW(lplibfilename : windows_core::PCWSTR) -> HMODULE);
windows_core::link!("kernel32.dll" "system" fn Module32FirstW(hsnapshot : HANDLE, lpme : *mut MODULEENTRY32W) -> windows_core::BOOL);
windows_core::link!("kernel32.dll" "system" fn Module32NextW(hsnapshot : HANDLE, lpme : *mut MODULEENTRY32W) -> windows_core::BOOL);
windows_core::link!("advapi32.dll" "system" fn OpenTraceW(logfile : *mut EVENT_TRACE_LOGFILEW) -> PROCESSTRACE_HANDLE);
windows_core::link!("user32.dll" "system" fn PostThreadMessageW(idthread : u32, msg : u32, wparam : WPARAM, lparam : LPARAM) -> windows_core::BOOL);
windows_core::link!("advapi32.dll" "system" fn ProcessTrace(handlearray : *const PROCESSTRACE_HANDLE, handlecount : u32, starttime : *const FILETIME, endtime : *const FILETIME) -> u32);
windows_core::link!("user32.dll" "system" fn RegisterClassW(lpwndclass : *const WNDCLASSW) -> ATOM);
windows_core::link!("user32.dll" "system" fn ScreenToClient(hwnd : HWND, lppoint : *mut POINT) -> windows_core::BOOL);
windows_core::link!("user32.dll" "system" fn SendMessageW(hwnd : HWND, msg : u32, wparam : WPARAM, lparam : LPARAM) -> LRESULT);
windows_core::link!("user32.dll" "system" fn SetLayeredWindowAttributes(hwnd : HWND, crkey : COLORREF, balpha : u8, dwflags : u32) -> windows_core::BOOL);
windows_core::link!("user32.dll" "system" fn SetWindowPos(hwnd : HWND, hwndinsertafter : HWND, x : i32, y : i32, cx : i32, cy : i32, uflags : u32) -> windows_core::BOOL);
windows_core::link!("user32.dll" "system" fn SetWindowsHookExW(idhook : i32, lpfn : HOOKPROC, hmod : HINSTANCE, dwthreadid : u32) -> HHOOK);
windows_core::link!("user32.dll" "system" fn ShowWindow(hwnd : HWND, ncmdshow : i32) -> windows_core::BOOL);
windows_core::link!("advapi32.dll" "system" fn StartTraceW(traceid : *mut CONTROLTRACE_ID, instancename : windows_core::PCWSTR, properties : *mut EVENT_TRACE_PROPERTIES) -> u32);
windows_core::link!("tdh.dll" "system" fn TdhGetEventInformation(event : *const EVENT_RECORD, tdhcontextcount : u32, tdhcontext : *const TDH_CONTEXT, buffer : *mut TRACE_EVENT_INFO, buffersize : *mut u32) -> TDHSTATUS);
windows_core::link!("tdh.dll" "system" fn TdhGetProperty(pevent : *const EVENT_RECORD, tdhcontextcount : u32, ptdhcontext : *const TDH_CONTEXT, propertydatacount : u32, ppropertydata : *const PROPERTY_DATA_DESCRIPTOR, buffersize : u32, pbuffer : *mut u8) -> TDHSTATUS);
windows_core::link!("tdh.dll" "system" fn TdhGetPropertySize(pevent : *const EVENT_RECORD, tdhcontextcount : u32, ptdhcontext : *const TDH_CONTEXT, propertydatacount : u32, ppropertydata : *const PROPERTY_DATA_DESCRIPTOR, ppropertysize : *mut u32) -> TDHSTATUS);
windows_core::link!("user32.dll" "system" fn UnhookWindowsHookEx(hhk : HHOOK) -> windows_core::BOOL);
windows_core::link!("user32.dll" "system" fn WindowFromPoint(point : POINT) -> HWND);
pub type ATOM = u16;
pub const CLASS_E_CLASSNOTAVAILABLE: windows_core::HRESULT =
    windows_core::HRESULT(0x80040111_u32 as _);
pub type COLORREF = u32;
pub type CONTROLTRACE_ID = u64;
pub type DECODING_SOURCE = i32;
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
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ENABLE_TRACE_PARAMETERS {
    pub Version: u32,
    pub EnableProperty: u32,
    pub ControlFlags: u32,
    pub SourceId: windows_core::GUID,
    pub EnableFilterDesc: PEVENT_FILTER_DESCRIPTOR,
    pub FilterDescCount: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ETW_BUFFER_CONTEXT {
    pub Anonymous: ETW_BUFFER_CONTEXT_0,
    pub LoggerId: u16,
}
impl Default for ETW_BUFFER_CONTEXT {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union ETW_BUFFER_CONTEXT_0 {
    pub Anonymous: ETW_BUFFER_CONTEXT_0_0,
    pub ProcessorIndex: u16,
}
impl Default for ETW_BUFFER_CONTEXT_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ETW_BUFFER_CONTEXT_0_0 {
    pub ProcessorNumber: u8,
    pub Alignment: u8,
}
pub const EVENT_CONTROL_CODE_ENABLE_PROVIDER: i32 = 1;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EVENT_DESCRIPTOR {
    pub Id: u16,
    pub Version: u8,
    pub Channel: u8,
    pub Level: u8,
    pub Opcode: u8,
    pub Task: u16,
    pub Keyword: u64,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EVENT_FILTER_DESCRIPTOR {
    pub Ptr: u64,
    pub Size: u32,
    pub Type: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EVENT_HEADER {
    pub Size: u16,
    pub HeaderType: u16,
    pub Flags: u16,
    pub EventProperty: u16,
    pub ThreadId: u32,
    pub ProcessId: u32,
    pub TimeStamp: i64,
    pub ProviderId: windows_core::GUID,
    pub EventDescriptor: EVENT_DESCRIPTOR,
    pub Anonymous: EVENT_HEADER_0,
    pub ActivityId: windows_core::GUID,
}
impl Default for EVENT_HEADER {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union EVENT_HEADER_0 {
    pub Anonymous: EVENT_HEADER_0_0,
    pub ProcessorTime: u64,
}
impl Default for EVENT_HEADER_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EVENT_HEADER_0_0 {
    pub KernelTime: u32,
    pub UserTime: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EVENT_HEADER_EXTENDED_DATA_ITEM {
    pub Reserved1: u16,
    pub ExtType: u16,
    pub Anonymous: EVENT_HEADER_EXTENDED_DATA_ITEM_0,
    pub DataSize: u16,
    pub DataPtr: u64,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EVENT_HEADER_EXTENDED_DATA_ITEM_0 {
    pub _bitfield: u16,
}
impl EVENT_HEADER_EXTENDED_DATA_ITEM_0 {
    pub fn Linkage(&self) -> bool {
        self._bitfield & 1 != 0
    }
    pub fn set_Linkage(&mut self, value: bool) {
        self._bitfield = (self._bitfield & !1) | (value as u16);
    }
    pub fn Reserved2(&self) -> u16 {
        self._bitfield >> 1
    }
    pub fn set_Reserved2(&mut self, value: u16) {
        self._bitfield = (self._bitfield & !(32767 << 1)) | ((value & 32767) << 1);
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EVENT_PROPERTY_INFO {
    pub Flags: PROPERTY_FLAGS,
    pub NameOffset: u32,
    pub Anonymous: EVENT_PROPERTY_INFO_0,
    pub Anonymous2: EVENT_PROPERTY_INFO_1,
    pub Anonymous3: EVENT_PROPERTY_INFO_2,
    pub Anonymous4: EVENT_PROPERTY_INFO_3,
}
impl Default for EVENT_PROPERTY_INFO {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union EVENT_PROPERTY_INFO_0 {
    pub nonStructType: EVENT_PROPERTY_INFO_0_0,
    pub structType: EVENT_PROPERTY_INFO_0_1,
    pub customSchemaType: EVENT_PROPERTY_INFO_0_2,
}
impl Default for EVENT_PROPERTY_INFO_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union EVENT_PROPERTY_INFO_1 {
    pub count: u16,
    pub countPropertyIndex: u16,
}
impl Default for EVENT_PROPERTY_INFO_1 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union EVENT_PROPERTY_INFO_2 {
    pub length: u16,
    pub lengthPropertyIndex: u16,
}
impl Default for EVENT_PROPERTY_INFO_2 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union EVENT_PROPERTY_INFO_3 {
    pub Reserved: u32,
    pub Anonymous: EVENT_PROPERTY_INFO_3_0,
}
impl Default for EVENT_PROPERTY_INFO_3 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EVENT_PROPERTY_INFO_3_0 {
    pub _bitfield: u32,
}
impl EVENT_PROPERTY_INFO_3_0 {
    pub fn Tags(&self) -> u32 {
        (self._bitfield << 4) >> 4
    }
    pub fn set_Tags(&mut self, value: u32) {
        self._bitfield = (self._bitfield & !268435455) | (value & 268435455);
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EVENT_PROPERTY_INFO_0_0 {
    pub InType: u16,
    pub OutType: u16,
    pub MapNameOffset: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EVENT_PROPERTY_INFO_0_1 {
    pub StructStartIndex: u16,
    pub NumOfStructMembers: u16,
    pub padding: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EVENT_PROPERTY_INFO_0_2 {
    pub InType: u16,
    pub OutType: u16,
    pub CustomSchemaOffset: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EVENT_RECORD {
    pub EventHeader: EVENT_HEADER,
    pub BufferContext: ETW_BUFFER_CONTEXT,
    pub ExtendedDataCount: u16,
    pub UserDataLength: u16,
    pub ExtendedData: PEVENT_HEADER_EXTENDED_DATA_ITEM,
    pub UserData: *mut core::ffi::c_void,
    pub UserContext: *mut core::ffi::c_void,
}
impl Default for EVENT_RECORD {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EVENT_TRACE {
    pub Header: EVENT_TRACE_HEADER,
    pub InstanceId: u32,
    pub ParentInstanceId: u32,
    pub ParentGuid: windows_core::GUID,
    pub MofData: *mut core::ffi::c_void,
    pub MofLength: u32,
    pub Anonymous: EVENT_TRACE_0,
}
impl Default for EVENT_TRACE {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union EVENT_TRACE_0 {
    pub ClientContext: u32,
    pub BufferContext: ETW_BUFFER_CONTEXT,
}
impl Default for EVENT_TRACE_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub const EVENT_TRACE_CONTROL_STOP: i32 = 1;
pub const EVENT_TRACE_FILE_MODE_CIRCULAR: i32 = 2;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EVENT_TRACE_HEADER {
    pub Size: u16,
    pub Anonymous: EVENT_TRACE_HEADER_0,
    pub Anonymous2: EVENT_TRACE_HEADER_1,
    pub ThreadId: u32,
    pub ProcessId: u32,
    pub TimeStamp: i64,
    pub Anonymous3: EVENT_TRACE_HEADER_2,
    pub Anonymous4: EVENT_TRACE_HEADER_3,
}
impl Default for EVENT_TRACE_HEADER {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union EVENT_TRACE_HEADER_0 {
    pub FieldTypeFlags: u16,
    pub Anonymous: EVENT_TRACE_HEADER_0_0,
}
impl Default for EVENT_TRACE_HEADER_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EVENT_TRACE_HEADER_0_0 {
    pub HeaderType: u8,
    pub MarkerFlags: u8,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union EVENT_TRACE_HEADER_1 {
    pub Version: u32,
    pub Class: EVENT_TRACE_HEADER_1_0,
}
impl Default for EVENT_TRACE_HEADER_1 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EVENT_TRACE_HEADER_1_0 {
    pub Type: u8,
    pub Level: u8,
    pub Version: u16,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union EVENT_TRACE_HEADER_2 {
    pub Guid: windows_core::GUID,
    pub GuidPtr: u64,
}
impl Default for EVENT_TRACE_HEADER_2 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union EVENT_TRACE_HEADER_3 {
    pub Anonymous: EVENT_TRACE_HEADER_3_0,
    pub ProcessorTime: u64,
    pub Anonymous2: EVENT_TRACE_HEADER_3_1,
}
impl Default for EVENT_TRACE_HEADER_3 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EVENT_TRACE_HEADER_3_0 {
    pub KernelTime: u32,
    pub UserTime: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EVENT_TRACE_HEADER_3_1 {
    pub ClientContext: u32,
    pub Flags: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EVENT_TRACE_LOGFILEW {
    pub LogFileName: windows_core::PWSTR,
    pub LoggerName: windows_core::PWSTR,
    pub CurrentTime: i64,
    pub BuffersRead: u32,
    pub Anonymous: EVENT_TRACE_LOGFILEW_0,
    pub CurrentEvent: EVENT_TRACE,
    pub LogfileHeader: TRACE_LOGFILE_HEADER,
    pub BufferCallback: PEVENT_TRACE_BUFFER_CALLBACKW,
    pub BufferSize: u32,
    pub Filled: u32,
    pub EventsLost: u32,
    pub Anonymous2: EVENT_TRACE_LOGFILEW_1,
    pub IsKernelTrace: u32,
    pub Context: *mut core::ffi::c_void,
}
impl Default for EVENT_TRACE_LOGFILEW {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union EVENT_TRACE_LOGFILEW_0 {
    pub LogFileMode: u32,
    pub ProcessTraceMode: u32,
}
impl Default for EVENT_TRACE_LOGFILEW_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union EVENT_TRACE_LOGFILEW_1 {
    pub EventCallback: PEVENT_CALLBACK,
    pub EventRecordCallback: PEVENT_RECORD_CALLBACK,
}
impl Default for EVENT_TRACE_LOGFILEW_1 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub const EVENT_TRACE_PRIVATE_IN_PROC: i32 = 131072;
pub const EVENT_TRACE_PRIVATE_LOGGER_MODE: i32 = 2048;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EVENT_TRACE_PROPERTIES {
    pub Wnode: WNODE_HEADER,
    pub BufferSize: u32,
    pub MinimumBuffers: u32,
    pub MaximumBuffers: u32,
    pub MaximumFileSize: u32,
    pub LogFileMode: u32,
    pub FlushTimer: u32,
    pub EnableFlags: u32,
    pub Anonymous: EVENT_TRACE_PROPERTIES_0,
    pub NumberOfBuffers: u32,
    pub FreeBuffers: u32,
    pub EventsLost: u32,
    pub BuffersWritten: u32,
    pub LogBuffersLost: u32,
    pub RealTimeBuffersLost: u32,
    pub LoggerThreadId: HANDLE,
    pub LogFileNameOffset: u32,
    pub LoggerNameOffset: u32,
}
impl Default for EVENT_TRACE_PROPERTIES {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union EVENT_TRACE_PROPERTIES_0 {
    pub AgeLimit: i32,
    pub FlushThreshold: i32,
}
impl Default for EVENT_TRACE_PROPERTIES_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub const E_NOINTERFACE: windows_core::HRESULT = windows_core::HRESULT(0x80004002_u32 as _);
pub type FARPROC = Option<unsafe extern "system" fn() -> isize>;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FILETIME {
    pub dwLowDateTime: u32,
    pub dwHighDateTime: u32,
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
    IClassFactory,
    IClassFactory_Vtbl,
    0x00000001_0000_0000_c000_000000000046
);
windows_core::imp::interface_hierarchy!(IClassFactory, windows_core::IUnknown);
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
windows_core::imp::define_interface!(
    IObjectWithSite,
    IObjectWithSite_Vtbl,
    0xfc4801a3_2ba9_11cf_a229_00aa003d7352
);
windows_core::imp::interface_hierarchy!(IObjectWithSite, windows_core::IUnknown);
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
pub type LPARAM = isize;
pub type LRESULT = isize;
pub const LWA_ALPHA: i32 = 2;
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
pub type PEVENT_CALLBACK = Option<unsafe extern "system" fn(pevent: *mut EVENT_TRACE)>;
pub type PEVENT_FILTER_DESCRIPTOR = *mut EVENT_FILTER_DESCRIPTOR;
pub type PEVENT_HEADER_EXTENDED_DATA_ITEM = *mut EVENT_HEADER_EXTENDED_DATA_ITEM;
pub type PEVENT_RECORD_CALLBACK = Option<unsafe extern "system" fn(eventrecord: *mut EVENT_RECORD)>;
pub type PEVENT_TRACE_BUFFER_CALLBACKW =
    Option<unsafe extern "system" fn(logfile: *mut EVENT_TRACE_LOGFILEW) -> u32>;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}
pub type PROCESSTRACE_HANDLE = u64;
pub const PROCESS_TRACE_MODE_EVENT_RECORD: i32 = 268435456;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PROPERTY_DATA_DESCRIPTOR {
    pub PropertyName: u64,
    pub ArrayIndex: u32,
    pub Reserved: u32,
}
pub type PROPERTY_FLAGS = i32;
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
pub const SWP_NOACTIVATE: i32 = 16;
pub const SWP_SHOWWINDOW: i32 = 64;
pub const SW_HIDE: i32 = 0;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SYSTEMTIME {
    pub wYear: u16,
    pub wMonth: u16,
    pub wDayOfWeek: u16,
    pub wDay: u16,
    pub wHour: u16,
    pub wMinute: u16,
    pub wSecond: u16,
    pub wMilliseconds: u16,
}
pub type TDHSTATUS = u32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TDH_CONTEXT {
    pub ParameterValue: u64,
    pub ParameterType: TDH_CONTEXT_TYPE,
    pub ParameterSize: u32,
}
pub type TDH_CONTEXT_TYPE = i32;
pub type TEMPLATE_FLAGS = i32;
pub const TH32CS_SNAPMODULE: i32 = 8;
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TIME_ZONE_INFORMATION {
    pub Bias: i32,
    pub StandardName: [u16; 32],
    pub StandardDate: SYSTEMTIME,
    pub StandardBias: i32,
    pub DaylightName: [u16; 32],
    pub DaylightDate: SYSTEMTIME,
    pub DaylightBias: i32,
}
impl Default for TIME_ZONE_INFORMATION {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TRACE_EVENT_INFO {
    pub ProviderGuid: windows_core::GUID,
    pub EventGuid: windows_core::GUID,
    pub EventDescriptor: EVENT_DESCRIPTOR,
    pub DecodingSource: DECODING_SOURCE,
    pub ProviderNameOffset: u32,
    pub LevelNameOffset: u32,
    pub ChannelNameOffset: u32,
    pub KeywordsNameOffset: u32,
    pub TaskNameOffset: u32,
    pub OpcodeNameOffset: u32,
    pub EventMessageOffset: u32,
    pub ProviderMessageOffset: u32,
    pub BinaryXMLOffset: u32,
    pub BinaryXMLSize: u32,
    pub Anonymous: TRACE_EVENT_INFO_0,
    pub Anonymous2: TRACE_EVENT_INFO_1,
    pub PropertyCount: u32,
    pub TopLevelPropertyCount: u32,
    pub Anonymous3: TRACE_EVENT_INFO_2,
    pub EventPropertyInfoArray: [EVENT_PROPERTY_INFO; 1],
}
impl Default for TRACE_EVENT_INFO {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union TRACE_EVENT_INFO_0 {
    pub EventNameOffset: u32,
    pub ActivityIDNameOffset: u32,
}
impl Default for TRACE_EVENT_INFO_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union TRACE_EVENT_INFO_1 {
    pub EventAttributesOffset: u32,
    pub RelatedActivityIDNameOffset: u32,
}
impl Default for TRACE_EVENT_INFO_1 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union TRACE_EVENT_INFO_2 {
    pub Flags: TEMPLATE_FLAGS,
    pub Anonymous: TRACE_EVENT_INFO_2_0,
}
impl Default for TRACE_EVENT_INFO_2 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TRACE_EVENT_INFO_2_0 {
    pub _bitfield: u32,
}
impl TRACE_EVENT_INFO_2_0 {
    pub fn Reserved(&self) -> u32 {
        (self._bitfield << 28) >> 28
    }
    pub fn set_Reserved(&mut self, value: u32) {
        self._bitfield = (self._bitfield & !15) | (value & 15);
    }
    pub fn Tags(&self) -> u32 {
        self._bitfield >> 4
    }
    pub fn set_Tags(&mut self, value: u32) {
        self._bitfield = (self._bitfield & !(268435455 << 4)) | ((value & 268435455) << 4);
    }
}
pub const TRACE_LEVEL_VERBOSE: i32 = 5;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TRACE_LOGFILE_HEADER {
    pub BufferSize: u32,
    pub Anonymous: TRACE_LOGFILE_HEADER_0,
    pub ProviderVersion: u32,
    pub NumberOfProcessors: u32,
    pub EndTime: i64,
    pub TimerResolution: u32,
    pub MaximumFileSize: u32,
    pub LogFileMode: u32,
    pub BuffersWritten: u32,
    pub Anonymous2: TRACE_LOGFILE_HEADER_1,
    pub LoggerName: windows_core::PWSTR,
    pub LogFileName: windows_core::PWSTR,
    pub TimeZone: TIME_ZONE_INFORMATION,
    pub BootTime: i64,
    pub PerfFreq: i64,
    pub StartTime: i64,
    pub ReservedFlags: u32,
    pub BuffersLost: u32,
}
impl Default for TRACE_LOGFILE_HEADER {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union TRACE_LOGFILE_HEADER_0 {
    pub Version: u32,
    pub VersionDetail: TRACE_LOGFILE_HEADER_0_0,
}
impl Default for TRACE_LOGFILE_HEADER_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TRACE_LOGFILE_HEADER_0_0 {
    pub MajorVersion: u8,
    pub MinorVersion: u8,
    pub SubVersion: u8,
    pub SubMinorVersion: u8,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union TRACE_LOGFILE_HEADER_1 {
    pub LogInstanceGuid: windows_core::GUID,
    pub Anonymous: TRACE_LOGFILE_HEADER_1_0,
}
impl Default for TRACE_LOGFILE_HEADER_1 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TRACE_LOGFILE_HEADER_1_0 {
    pub StartBuffers: u32,
    pub PointerSize: u32,
    pub EventsLost: u32,
    pub CpuSpeedInMHz: u32,
}
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
pub const WNODE_FLAG_TRACED_GUID: i32 = 131072;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct WNODE_HEADER {
    pub BufferSize: u32,
    pub ProviderId: u32,
    pub Anonymous: WNODE_HEADER_0,
    pub Anonymous2: WNODE_HEADER_1,
    pub Guid: windows_core::GUID,
    pub ClientContext: u32,
    pub Flags: u32,
}
impl Default for WNODE_HEADER {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union WNODE_HEADER_0 {
    pub HistoricalContext: u64,
    pub Anonymous: WNODE_HEADER_0_0,
}
impl Default for WNODE_HEADER_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WNODE_HEADER_0_0 {
    pub Version: u32,
    pub Linkage: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union WNODE_HEADER_1 {
    pub CountLost: u32,
    pub KernelHandle: HANDLE,
    pub TimeStamp: i64,
}
impl Default for WNODE_HEADER_1 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub type WPARAM = usize;
pub const WS_EX_LAYERED: i32 = 524288;
pub const WS_EX_NOACTIVATE: i32 = 134217728;
pub const WS_EX_TOOLWINDOW: i32 = 128;
pub const WS_EX_TOPMOST: i32 = 8;
pub const WS_EX_TRANSPARENT: i32 = 32;
pub const WS_POPUP: u32 = 2147483648;
