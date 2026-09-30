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
    /// Each element's children, so that removing one takes its subtree
    /// without walking the whole tree.
    children: HashMap<u64, Vec<u64>>,
    pending: Vec<Change>,
    roots: Vec<InstanceHandle>,
}

impl State {
    fn add(&mut self, element: Element) {
        let (handle, parent) = (element.handle, element.parent);
        if let Some(before) = self.elements.insert(handle, element)
            && before.parent != parent
        {
            self.detach(before.parent, handle);
        }

        let siblings = self.children.entry(parent).or_default();
        if !siblings.contains(&handle) {
            siblings.push(handle);
        }
    }

    /// Takes `handle` and everything below it out of the tree.
    fn remove(&mut self, handle: u64) {
        if let Some(element) = self.elements.get(&handle) {
            let parent = element.parent;
            self.detach(parent, handle);
        }

        let mut gone = vec![handle];
        while let Some(next) = gone.pop() {
            self.elements.remove(&next);
            if let Some(below) = self.children.remove(&next) {
                gone.extend(below);
            }
        }
    }

    fn detach(&mut self, parent: u64, handle: u64) {
        if let Some(siblings) = self.children.get_mut(&parent) {
            siblings.retain(|&sibling| sibling != handle);
            if siblings.is_empty() {
                self.children.remove(&parent);
            }
        }
    }
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
                    mark: String::new(),
                };
                state.add(added.clone());
                state.pending.push(Change::Added(added));
            } else {
                state.remove(element.handle);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn element(handle: u64, parent: u64) -> Element {
        Element {
            handle,
            parent,
            index: 0,
            kind: String::new(),
            name: String::new(),
            mark: String::new(),
        }
    }

    #[test]
    fn removing_an_element_takes_its_subtree_and_nothing_else() {
        let mut state = State::default();
        for (handle, parent) in [(1, 0), (2, 1), (3, 2), (4, 1), (5, 0)] {
            state.add(element(handle, parent));
        }

        state.remove(2);
        let mut left: Vec<u64> = state.elements.keys().copied().collect();
        left.sort_unstable();
        assert_eq!(left, [1, 4, 5]);
        assert_eq!(state.children.get(&1), Some(&vec![4]));
        assert!(!state.children.contains_key(&2));

        state.add(element(4, 5));
        assert!(!state.children.contains_key(&1), "a moved element leaves its old parent");
        state.remove(5);
        let mut left: Vec<u64> = state.elements.keys().copied().collect();
        left.sort_unstable();
        assert_eq!(left, [1]);
    }
}
