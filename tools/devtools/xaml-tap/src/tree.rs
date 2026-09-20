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
