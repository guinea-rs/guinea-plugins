//! A XAML diagnostics tap: the DLL devtools load into a WinUI 3 process to see
//! its live tree.
//!
//! Devtools call `InitializeXamlDiagnosticsEx` with this DLL's path and
//! [`CLSID`]; the process's XAML runtime loads it, creates the object below
//! and hands it an `IXamlDiagnostics` through `SetSite`, on the UI thread.
//! From there the tap subscribes to the tree and connects to devtools like
//! any application would. Nothing in it knows about guinea: any WinUI 3
//! process can be looked into.

#![cfg(windows)]

mod generated;
mod highlight;
pub mod inject;
mod input;
mod inspect;
mod link;
mod perf;
pub mod pick;
mod sample;
#[allow(non_snake_case)]
mod tree;
mod ui;

use std::ffi::c_void;

use windows_core::{GUID, HRESULT, IUnknown, Interface, Ref, implement};

/// What the rest of the tap calls the two generated modules, so a call site
/// says `crate::bindings::…` and never minds where they sit.
use generated::{bindings, diag};

use bindings::{
    CLASS_E_CLASSNOTAVAILABLE, E_NOINTERFACE, IClassFactory, IClassFactory_Impl, IObjectWithSite,
    IObjectWithSite_Impl,
};
use diag::{IVisualTreeService, IVisualTreeServiceCallback, IXamlDiagnostics};

/// The class XAML is asked to create.
pub const CLSID: GUID = GUID::from_u128(0x6f0b1c52_8d7e_4a39_9b0e_3c1d2e4f5a61);

/// The file this crate builds, next to the devtools executable.
pub const DLL: &str = "guinea_xaml_tap.dll";

#[implement(IObjectWithSite)]
struct Tap;

impl IObjectWithSite_Impl for Tap_Impl {
    fn SetSite(&self, site: Ref<IUnknown>) -> windows_core::Result<()> {
        let Some(site) = site.as_ref() else {
            return Ok(());
        };
        let diagnostics: IXamlDiagnostics = site.cast()?;

        ui::install();
        inspect::install(diagnostics.clone())?;

        let service: IVisualTreeService = diagnostics.cast()?;
        let raw = service.into_raw() as usize;

        std::thread::Builder::new()
            .name("guinea-xaml-tap-advise".into())
            .spawn(move || {
                let service = unsafe { IVisualTreeService::from_raw(raw as *mut c_void) };
                let watcher: IVisualTreeServiceCallback = tree::Watcher.into();

                let advised = unsafe { service.AdviseVisualTreeChange(watcher.as_raw()) };
                if let Err(error) = advised.ok() {
                    tracing::warn!(%error, "the xaml tree cannot be watched");
                    return;
                }

                std::mem::forget(watcher);
                link::spawn();
            })
            .map_err(|_| windows_core::Error::from_hresult(HRESULT(0x8000_4005_u32 as i32)))?;

        Ok(())
    }

    fn GetSite(&self, _riid: *const GUID, _site: *mut *mut c_void) -> windows_core::Result<()> {
        Err(E_NOINTERFACE.into())
    }
}

#[implement(IClassFactory)]
struct Factory;

impl IClassFactory_Impl for Factory_Impl {
    fn CreateInstance(
        &self,
        _outer: Ref<IUnknown>,
        riid: *const GUID,
        object: *mut *mut c_void,
    ) -> windows_core::Result<()> {
        let tap: IUnknown = Tap.into();
        unsafe { tap.query(riid, object).ok() }
    }

    fn LockServer(&self, _lock: windows_core::BOOL) -> windows_core::Result<()> {
        Ok(())
    }
}

/// # Safety
/// Called by COM with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllGetClassObject(
    clsid: *const GUID,
    riid: *const GUID,
    object: *mut *mut c_void,
) -> HRESULT {
    if unsafe { *clsid } != CLSID {
        return CLASS_E_CLASSNOTAVAILABLE;
    }

    let factory: IClassFactory = Factory.into();
    unsafe { factory.query(riid, object) }
}

/// The tap stays for the life of the process: XAML keeps its callbacks.
#[unsafe(no_mangle)]
pub extern "system" fn DllCanUnloadNow() -> HRESULT {
    HRESULT(1)
}
