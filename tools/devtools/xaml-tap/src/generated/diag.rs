//! The XAML diagnostics interfaces, from `xamlom.h` and WinUI's
//! `XamlOM.WinUI.idl`. No metadata carries them.

use std::ffi::c_void;

use windows_core::{BOOL, HRESULT, IUnknown, interface};

pub type InstanceHandle = u64;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SourceInfo {
    pub file_name: *const u16,
    pub line: u32,
    pub column: u32,
    pub char_position: u32,
    pub hash: *const u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ParentChildRelation {
    pub parent: InstanceHandle,
    pub child: InstanceHandle,
    pub child_index: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VisualElement {
    pub handle: InstanceHandle,
    pub source: SourceInfo,
    pub kind: *const u16,
    pub name: *const u16,
    pub children: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PropertyChainSource {
    pub handle: InstanceHandle,
    pub target_type: *const u16,
    pub name: *const u16,
    pub source: i32,
    pub source_info: SourceInfo,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PropertyChainValue {
    pub index: u32,
    pub kind: *const u16,
    pub declaring_type: *const u16,
    pub value_type: *const u16,
    pub item_type: *const u16,
    pub value: *const u16,
    pub overridden: BOOL,
    pub metadata_bits: i64,
    pub property_name: *const u16,
    pub property_chain_index: u32,
}

pub const MUTATION_ADD: i32 = 0;

pub const IS_VALUE_HANDLE: i64 = 0x1;
pub const IS_PROPERTY_READ_ONLY: i64 = 0x2;
pub const IS_VALUE_BINDING_EXPRESSION: i64 = 0x10;

#[repr(C)]
pub struct SafeArrayBound {
    pub elements: u32,
    pub lower_bound: i32,
}

#[repr(C)]
pub struct SafeArray {
    pub dimensions: u16,
    pub features: u16,
    pub element_size: u32,
    pub locks: u32,
    pub data: *mut c_void,
    pub bounds: [SafeArrayBound; 1],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EnumType {
    pub name: *const u16,
    pub value_ints: *const SafeArray,
    pub value_strings: *const SafeArray,
}

windows_core::link!("oleaut32.dll" "system" fn SafeArrayDestroy(array : *const SafeArray) -> HRESULT);

/// Frees a `SAFEARRAY` the runtime handed over, and the `BSTR`s in it when it
/// holds strings.
///
/// # Safety
/// `array` is null or an array nothing else will read again.
pub unsafe fn free_safe_array(array: *const SafeArray) {
    if !array.is_null() {
        let _ = unsafe { SafeArrayDestroy(array) };
    }
}

#[interface("AA7A8931-80E4-4FEC-8F3B-553F87B4966E")]
pub unsafe trait IVisualTreeServiceCallback: IUnknown {
    fn OnVisualTreeChange(
        &self,
        relation: ParentChildRelation,
        element: VisualElement,
        mutation: i32,
    ) -> HRESULT;
}

#[interface("765DF10A-08C8-46B0-82C9-297CFC4CEAD7")]
pub unsafe trait IVisualTreeServiceCallback3: IUnknown {
    fn OnXamlRootChange(&self, root: InstanceHandle, mutation: i32) -> HRESULT;
}

#[interface("A593B11A-D17F-48BB-8F66-83910731C8A5")]
pub unsafe trait IVisualTreeService: IUnknown {
    pub fn AdviseVisualTreeChange(&self, callback: *mut c_void) -> HRESULT;
    fn UnadviseVisualTreeChange(&self, callback: *mut c_void) -> HRESULT;
    pub fn GetEnums(&self, count: *mut u32, enums: *mut *mut EnumType) -> HRESULT;
    pub fn CreateInstance(
        &self,
        type_name: *const u16,
        value: *const u16,
        handle: *mut InstanceHandle,
    ) -> HRESULT;
    pub fn GetPropertyValuesChain(
        &self,
        handle: InstanceHandle,
        source_count: *mut u32,
        sources: *mut *mut PropertyChainSource,
        property_count: *mut u32,
        values: *mut *mut PropertyChainValue,
    ) -> HRESULT;
    pub fn SetProperty(
        &self,
        handle: InstanceHandle,
        value: InstanceHandle,
        property: u32,
    ) -> HRESULT;
}

#[interface("18C9E2B6-3F43-4116-9F2B-FF935D7770D2")]
pub unsafe trait IXamlDiagnostics: IUnknown {
    fn GetDispatcher(&self, dispatcher: *mut *mut c_void) -> HRESULT;
    fn GetUiLayer(&self, layer: *mut *mut c_void) -> HRESULT;
    fn GetApplication(&self, application: *mut *mut c_void) -> HRESULT;
    pub fn GetIInspectableFromHandle(
        &self,
        handle: InstanceHandle,
        instance: *mut *mut c_void,
    ) -> HRESULT;
    fn GetHandleFromIInspectable(
        &self,
        instance: *mut c_void,
        handle: *mut InstanceHandle,
    ) -> HRESULT;
}

#[interface("523A35EE-EB38-4AE6-A3E1-5B7D0D547BD0")]
pub unsafe trait IXamlDiagnostics2: IUnknown {
    fn GetUiLayerForXamlRoot(&self, root: InstanceHandle, layer: *mut *mut c_void) -> HRESULT;
    pub fn HitTestForXamlRoot(
        &self,
        root: InstanceHandle,
        rect: RECT,
        count: *mut u32,
        handles: *mut *mut InstanceHandle,
    ) -> HRESULT;
}

/// A `BSTR` the runtime owns, read.
pub fn text(bstr: *const u16) -> String {
    if bstr.is_null() {
        return String::new();
    }

    let bytes = unsafe { *(bstr as *const u32).sub(1) } as usize;
    String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(bstr, bytes / 2) })
}
