//! The live tree, as XAML diagnostics tell it.
//!
//! Changes arrive on the UI thread, one element at a time; they are only
//! recorded here, and the link takes them in batches. The whole tree is kept
//! as well, so a devtools that connects later gets all of it at once.

use std::collections::HashMap;
use std::sync::Mutex;

use guinea_devtools_protocol::native::{Change, Element};
use windows_core::{HRESULT, implement};

use crate::diag::{
    IVisualTreeServiceCallback, IVisualTreeServiceCallback3, IVisualTreeServiceCallback3_Impl,
    IVisualTreeServiceCallback_Impl, InstanceHandle, MUTATION_ADD, ParentChildRelation,
    VisualElement, text,
};

#[derive(Default)]
struct State {
    elements: HashMap<u64, Element>,
    pending: Vec<Change>,
    roots: Vec<InstanceHandle>,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

fn with<R>(job: impl FnOnce(&mut State) -> R) -> R {
    let mut state = STATE.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    job(state.get_or_insert_with(State::default))
}

/// What changed since the last call.
pub fn take_changes() -> Vec<Change> {
    with(|state| std::mem::take(&mut state.pending))
}

/// The whole tree, siblings in order, and nothing pending any more.
pub fn whole() -> Vec<Change> {
    with(|state| {
        state.pending.clear();

        let mut elements: Vec<Element> = state.elements.values().cloned().collect();
        elements.sort_by_key(|element| (element.parent, element.index));
        elements.into_iter().map(Change::Added).collect()
    })
}

/// The elements below `under` - or the whole tree - depth first, siblings in
/// order: the order a test counts the elements that carry one mark in.
pub fn in_order(under: Option<u64>) -> Vec<u64> {
    with(|state| {
        let mut children: HashMap<u64, Vec<&Element>> = HashMap::new();
        for element in state.elements.values() {
            children.entry(element.parent).or_default().push(element);
        }
        for siblings in children.values_mut() {
            siblings.sort_by_key(|element| element.index);
        }

        let mut unseen: Vec<u64> = match under {
            Some(parent) => children.get(&parent).map_or_else(Vec::new, |below| {
                below.iter().rev().map(|element| element.handle).collect()
            }),
            None => {
                let mut tops: Vec<&Element> = state
                    .elements
                    .values()
                    .filter(|element| !state.elements.contains_key(&element.parent))
                    .collect();
                tops.sort_by_key(|element| (element.parent, element.index));
                tops.iter().rev().map(|element| element.handle).collect()
            }
        };

        let mut ordered = Vec::new();
        while let Some(handle) = unseen.pop() {
            ordered.push(handle);
            if let Some(below) = children.get(&handle) {
                unseen.extend(below.iter().rev().map(|element| element.handle));
            }
        }
        ordered
    })
}

/// The XAML roots - one per window or island - that hit tests start from.
pub fn roots() -> Vec<InstanceHandle> {
    with(|state| state.roots.clone())
}

#[implement(IVisualTreeServiceCallback, IVisualTreeServiceCallback3)]
pub struct Watcher;

impl IVisualTreeServiceCallback_Impl for Watcher_Impl {
    unsafe fn OnVisualTreeChange(
        &self,
        relation: ParentChildRelation,
        element: VisualElement,
        mutation: i32,
    ) -> HRESULT {
        with(|state| {
            if mutation == MUTATION_ADD {
                let added = Element {
                    handle: element.handle,
                    parent: relation.parent,
                    index: relation.child_index,
                    kind: text(element.kind),
                    name: text(element.name),
                };
                state.elements.insert(added.handle, added.clone());
                state.pending.push(Change::Added(added));
            } else {
                let mut gone = vec![element.handle];
                while let Some(next) = gone.pop() {
                    state.elements.remove(&next);
                    gone.extend(
                        state
                            .elements
                            .values()
                            .filter(|child| child.parent == next)
                            .map(|child| child.handle),
                    );
                }

                state.pending.push(Change::Removed {
                    handle: element.handle,
                    parent: relation.parent,
                });
            }
        });
        HRESULT(0)
    }
}

impl IVisualTreeServiceCallback3_Impl for Watcher_Impl {
    unsafe fn OnXamlRootChange(&self, root: InstanceHandle, mutation: i32) -> HRESULT {
        with(|state| {
            if mutation == MUTATION_ADD {
                state.roots.push(root);
            } else {
                state.roots.retain(|&known| known != root);
            }
        });
        HRESULT(0)
    }
}
