//! What devtools ask of the tree: properties, edits, what lies under a point,
//! and where an element is. Everything here runs on the UI thread.

use std::cell::RefCell;

use guinea_devtools_protocol::native::{Bounds, Enumeration, Property};
use windows_core::{BSTR, IInspectable, Interface};

use crate::bindings::{
    AutomationProperties, ClientToScreen, CoTaskMemFree, DependencyObject, GetDpiForWindow, HWND, POINT, Point,
    ScreenToClient, UIElement,
};
use crate::diag::{
    EnumType, IS_PROPERTY_READ_ONLY, IS_VALUE_BINDING_EXPRESSION, IS_VALUE_HANDLE, IVisualTreeService, IXamlDiagnostics,
    IXamlDiagnostics2, InstanceHandle, PropertyChainSource, PropertyChainValue, RECT, SafeArray, free_safe_array, text,
};
use crate::{tree, ui};

pub struct Inspector {
    diagnostics: IXamlDiagnostics,
    service: IVisualTreeService,
    roots: Option<IXamlDiagnostics2>,
}

thread_local! {
    static INSPECTOR: RefCell<Option<Inspector>> = const { RefCell::new(None) };
}

/// Keeps what `SetSite` handed over, for the UI thread's later use.
pub fn install(diagnostics: IXamlDiagnostics) -> windows_core::Result<()> {
    let service = diagnostics.cast()?;
    let roots = diagnostics.cast().ok();

    INSPECTOR.with(|slot| {
        *slot.borrow_mut() = Some(Inspector {
            diagnostics,
            service,
            roots,
        })
    });
    Ok(())
}

/// Runs `job` against the inspector, on the UI thread.
pub fn with<R>(job: impl FnOnce(&Inspector) -> Result<R, String>) -> Result<R, String> {
    ui::on_ui(|| {
        INSPECTOR.with(|slot| match slot.borrow().as_ref() {
            Some(inspector) => job(inspector),
            None => Err("the inspector is not installed".to_string()),
        })
    })
    .unwrap_or_else(|| Err("no way onto the UI thread".to_string()))
}

fn source_name(source: i32) -> &'static str {
    match source {
        1 => "default",
        2 => "built_in_style",
        3 => "style",
        4 => "local",
        5 => "inherited",
        6 => "default_style_trigger",
        7 => "template_trigger",
        8 => "style_trigger",
        9 => "implicit_style_reference",
        10 => "parent_template",
        11 => "parent_template_trigger",
        12 => "animation",
        13 => "coercion",
        14 => "visual_state",
        _ => "unknown",
    }
}

/// The elements of a one-dimensional `SAFEARRAY`, borrowed.
///
/// # Safety
/// `array` is null or a live one-dimensional array of `T`.
unsafe fn safe_array<'a, T>(array: *const SafeArray) -> &'a [T] {
    if array.is_null() {
        return &[];
    }

    let array = unsafe { &*array };
    if array.dimensions != 1 || array.data.is_null() {
        return &[];
    }
    unsafe { std::slice::from_raw_parts(array.data as *const T, array.bounds[0].elements as usize) }
}

fn free(bstr: *const u16) {
    if !bstr.is_null() {
        drop(unsafe { BSTR::from_raw(bstr) });
    }
}

impl Inspector {
    /// Every property of `element`, with the colour each brush paints.
    pub fn properties(&self, element: InstanceHandle) -> Result<Vec<Property>, String> {
        let mut properties = self.chain(element)?;

        for property in &mut properties {
            let brush = property.object && property.value_type.ends_with("Brush") && property.value != "0";
            if let Some(handle) = brush.then(|| property.value.parse().ok()).flatten() {
                property.color = self.winning(handle, "Color");
            }
        }
        Ok(properties)
    }

    /// The value that wins for `name` on `element`.
    fn winning(&self, element: InstanceHandle, name: &str) -> Option<String> {
        self.chain(element)
            .ok()?
            .into_iter()
            .find(|property| property.name == name && !property.overridden)
            .map(|property| property.value)
    }

    /// Every enumeration the tree's properties use.
    pub fn enums(&self) -> Result<Vec<Enumeration>, String> {
        let mut count = 0u32;
        let mut types: *mut EnumType = std::ptr::null_mut();
        unsafe {
            self.service
                .GetEnums(&mut count, &mut types)
                .ok()
                .map_err(|error| format!("reading the enumerations: {error}"))?;
        }

        let read = unsafe { std::slice::from_raw_parts(types, count as usize) };
        let enums = read
            .iter()
            .map(|kind| {
                let ints = unsafe { safe_array::<i32>(kind.value_ints) };
                let names = unsafe { safe_array::<*const u16>(kind.value_strings) };
                Enumeration {
                    name: text(kind.name),
                    values: ints.iter().zip(names).map(|(&value, &name)| (value, text(name))).collect(),
                }
            })
            .collect();

        // Ours to free now that they are read: the name of each enumeration,
        // and both arrays - destroying the array of names frees the names.
        for kind in read {
            free(kind.name);
            unsafe {
                free_safe_array(kind.value_ints);
                free_safe_array(kind.value_strings);
            }
        }
        unsafe { CoTaskMemFree(types.cast()) };

        Ok(enums)
    }

    fn chain(&self, element: InstanceHandle) -> Result<Vec<Property>, String> {
        let mut source_count = 0;
        let mut sources: *mut PropertyChainSource = std::ptr::null_mut();
        let mut value_count = 0;
        let mut values: *mut PropertyChainValue = std::ptr::null_mut();

        unsafe {
            self.service
                .GetPropertyValuesChain(element, &mut source_count, &mut sources, &mut value_count, &mut values)
                .ok()
                .map_err(|error| format!("reading the properties: {error}"))?;
        }

        let chain = unsafe { std::slice::from_raw_parts(sources, source_count as usize) };
        let read = unsafe { std::slice::from_raw_parts(values, value_count as usize) };

        let properties = read
            .iter()
            .map(|value| Property {
                name: text(value.property_name),
                value: text(value.value),
                value_type: text(value.value_type),
                declaring_type: text(value.declaring_type),
                source: chain
                    .get(value.property_chain_index as usize)
                    .map_or("unknown", |source| source_name(source.source))
                    .to_string(),
                overridden: value.overridden.as_bool(),
                index: value.index,
                read_only: value.metadata_bits & IS_PROPERTY_READ_ONLY != 0,
                object: value.metadata_bits & IS_VALUE_HANDLE != 0,
                binding: value.metadata_bits & IS_VALUE_BINDING_EXPRESSION != 0,
                color: None,
            })
            .collect();

        for source in chain {
            for bstr in [source.target_type, source.name, source.source_info.file_name, source.source_info.hash] {
                free(bstr);
            }
        }
        for value in read {
            for bstr in [value.kind, value.declaring_type, value.value_type, value.item_type, value.value, value.property_name] {
                free(bstr);
            }
        }
        unsafe {
            CoTaskMemFree(sources.cast());
            CoTaskMemFree(values.cast());
        }

        Ok(properties)
    }

    pub fn set_property(
        &self,
        element: InstanceHandle,
        property: u32,
        type_name: &str,
        value: &str,
    ) -> Result<(), String> {
        let kind = BSTR::from(type_name);
        let text = BSTR::from(value);
        let mut created = 0;

        unsafe {
            self.service
                .CreateInstance(kind.as_ptr(), text.as_ptr(), &mut created)
                .ok()
                .map_err(|error| format!("making a {type_name} of {value:?}: {error}"))?;
            self.service
                .SetProperty(element, created, property)
                .ok()
                .map_err(|error| format!("setting it: {error}"))
        }
    }

    /// What lies under `(x, y)` on the screen, innermost first, and where the
    /// innermost is.
    pub fn hit_test(&self, x: i32, y: i32) -> Result<(Vec<u64>, Option<Bounds>), String> {
        let Some(roots) = &self.roots else {
            return Err("this WinUI has no hit test by root".to_string());
        };

        let screen = POINT { x, y };
        let Some(window) = ui::window_at(screen) else {
            return Ok((Vec::new(), None));
        };

        let mut client = screen;
        let _ = unsafe { ScreenToClient(window, &mut client) };

        let scale = scale_of(window);
        let left = (client.x as f32 / scale) as i32;
        let top = (client.y as f32 / scale) as i32;
        let at = RECT {
            left,
            top,
            right: left + 1,
            bottom: top + 1,
        };

        for root in tree::roots() {
            let mut count = 0u32;
            let mut handles: *mut InstanceHandle = std::ptr::null_mut();
            let hit = unsafe { roots.HitTestForXamlRoot(root, at, &mut count, &mut handles) };
            if hit.is_err() || count == 0 {
                continue;
            }

            let chain = unsafe { std::slice::from_raw_parts(handles, count as usize) }.to_vec();
            unsafe { CoTaskMemFree(handles.cast()) };

            let bounds = self.bounds_in(chain[0], window);
            return Ok((chain, bounds));
        }

        Ok((Vec::new(), None))
    }

    /// The mark `element` carries - its `AutomationId` - when it has one.
    pub fn mark(&self, element: InstanceHandle) -> Option<String> {
        let mut raw = std::ptr::null_mut();
        unsafe { self.diagnostics.GetIInspectableFromHandle(element, &mut raw).ok().ok()? };
        let inspectable = unsafe { IInspectable::from_raw(raw) };
        let object: DependencyObject = inspectable.cast().ok()?;

        let mark = AutomationProperties::GetAutomationId(&object).ok()?;
        (!mark.is_empty()).then_some(mark)
    }

    /// Where `element` is on the screen, and the window it sits in.
    ///
    /// The transform an element gives is relative to its own root, so the
    /// same numbers read differently in every window. The window this
    /// element belongs to is the one whose client area the rectangle lands
    /// inside; with one window that is the only candidate anyway.
    pub fn bounds(&self, element: InstanceHandle) -> Option<(Bounds, HWND)> {
        let windows = ui::windows();
        let placed: Vec<(Bounds, HWND)> = windows
            .iter()
            .filter_map(|&window| Some((self.bounds_in(element, window)?, window)))
            .collect();

        placed
            .iter()
            .find(|(bounds, window)| within(*bounds, *window))
            .copied()
            .or_else(|| placed.first().copied())
    }

    fn bounds_in(&self, element: InstanceHandle, window: HWND) -> Option<Bounds> {
        let mut raw = std::ptr::null_mut();
        unsafe { self.diagnostics.GetIInspectableFromHandle(element, &mut raw).ok().ok()? };
        let inspectable = unsafe { IInspectable::from_raw(raw) };
        let element: UIElement = inspectable.cast().ok()?;

        let origin = element
            .TransformToVisual(None::<&UIElement>)
            .ok()?
            .TransformPoint(Point { x: 0.0, y: 0.0 })
            .ok()?;
        let size = element.ActualSize().ok()?;

        let scale = scale_of(window);
        let mut corner = POINT {
            x: (origin.x * scale).round() as i32,
            y: (origin.y * scale).round() as i32,
        };
        let _ = unsafe { ClientToScreen(window, &mut corner) };

        Some(Bounds {
            x: corner.x,
            y: corner.y,
            width: (size.x * scale).round() as i32,
            height: (size.y * scale).round() as i32,
        })
    }
}

/// Whether `bounds` overlaps `window`'s client area at all.
fn within(bounds: Bounds, window: HWND) -> bool {
    let Some(client) = ui::client_area(window) else {
        return false;
    };

    bounds.x + bounds.width >= client.left
        && bounds.y + bounds.height >= client.top
        && bounds.x <= client.right
        && bounds.y <= client.bottom
}

fn scale_of(window: HWND) -> f32 {
    match unsafe { GetDpiForWindow(window) } {
        0 => 1.0,
        dpi => dpi as f32 / 96.0,
    }
}
