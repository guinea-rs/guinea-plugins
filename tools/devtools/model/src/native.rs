//! The native tree a backend's inspector reported, kept as it changes.

use std::collections::HashMap;

use guinea_devtools_protocol::native::{Bounds, Change, Element, Property};

/// Every element, with its children in order.
#[derive(Clone, Debug, Default)]
pub struct NativeTree {
    elements: HashMap<u64, Element>,
    children: HashMap<u64, Vec<u64>>,
    roots: Vec<u64>,
}

impl NativeTree {
    pub fn apply(&mut self, changes: Vec<Change>) {
        for change in changes {
            match change {
                Change::Added(element) => self.add(element),
                Change::Removed { handle, .. } => self.remove(handle),
            }
        }
    }

    fn add(&mut self, element: Element) {
        let handle = element.handle;
        if self.elements.contains_key(&handle) {
            self.detach(handle);
        }

        let siblings = if element.parent == 0 {
            &mut self.roots
        } else {
            self.children.entry(element.parent).or_default()
        };
        let at = (element.index as usize).min(siblings.len());
        siblings.insert(at, handle);

        self.elements.insert(handle, element);
    }

    fn remove(&mut self, handle: u64) {
        self.detach(handle);

        let mut gone = vec![handle];
        while let Some(next) = gone.pop() {
            self.elements.remove(&next);
            gone.extend(self.children.remove(&next).unwrap_or_default());
        }
    }

    fn detach(&mut self, handle: u64) {
        let Some(parent) = self.elements.get(&handle).map(|element| element.parent) else {
            return;
        };

        let siblings = if parent == 0 {
            &mut self.roots
        } else {
            match self.children.get_mut(&parent) {
                Some(siblings) => siblings,
                None => return,
            }
        };
        siblings.retain(|&sibling| sibling != handle);
    }

    pub fn get(&self, handle: u64) -> Option<&Element> {
        self.elements.get(&handle)
    }

    pub fn roots(&self) -> &[u64] {
        &self.roots
    }

    pub fn children(&self, handle: u64) -> &[u64] {
        self.children.get(&handle).map_or(&[], Vec::as_slice)
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Every element carrying `mark`, depth first, siblings in order.
    pub fn marked(&self, mark: &str) -> Vec<u64> {
        let mut found = Vec::new();
        let mut unseen: Vec<u64> = self.roots.iter().rev().copied().collect();

        while let Some(handle) = unseen.pop() {
            if self
                .elements
                .get(&handle)
                .is_some_and(|element| element.mark == mark)
            {
                found.push(handle);
            }
            unseen.extend(self.children(handle).iter().rev());
        }

        found
    }

    /// From the root down to `handle`, both included; empty when it is gone.
    pub fn path(&self, handle: u64) -> Vec<u64> {
        let mut path = Vec::new();
        let mut at = handle;

        while let Some(element) = self.elements.get(&at) {
            path.push(at);
            if element.parent == 0 || path.len() > 4096 {
                break;
            }
            at = element.parent;
        }

        path.reverse();
        path
    }
}

/// What the inspector answered last.
#[derive(Clone, Debug, Default)]
pub struct Inspection {
    pub tree: NativeTree,
    /// The element whose properties arrived last, and them.
    pub properties: Option<(u64, Vec<Property>)>,
    /// What lay under the pointer, innermost first.
    pub picked: Option<Picked>,
    /// The last command that was turned down, and why.
    pub refused: Option<(String, String)>,
    /// The frames of the last capture, oldest first.
    pub frames: Vec<guinea_devtools_protocol::native::Frame>,
    /// Every frame captured since the inspector came, oldest first: the
    /// newest [`KEPT_FRAMES`] of them. A capture restarts the inspector's
    /// ring, so the ring alone only ever holds what came since the last one.
    pub kept: Vec<guinea_devtools_protocol::native::Frame>,
    /// The UI thread's stacks, sampled while sampling was on.
    pub sampled: crate::samples::Sampled,
    /// Each enumeration's value names, by the type's full name.
    pub enums: HashMap<String, Vec<(i32, String)>>,
}

/// How many captured frames an inspection keeps: at 60 frames a second,
/// about ten minutes of steady drawing.
pub const KEPT_FRAMES: usize = 36_000;

impl Inspection {
    /// Takes in a capture: it becomes the last one, and its frames join the
    /// kept ones - once each, however many captures a frame was in.
    pub fn captured(&mut self, frames: Vec<guinea_devtools_protocol::native::Frame>) {
        for frame in frames.iter().filter(|frame| frame.qpc != 0) {
            if let Err(at) = self.kept.binary_search_by_key(&frame.qpc, |kept| kept.qpc) {
                self.kept.insert(at, frame.clone());
            }
        }
        let over = self.kept.len().saturating_sub(KEPT_FRAMES);
        self.kept.drain(..over);
        self.frames = frames;
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Picked {
    pub chain: Vec<u64>,
    pub bounds: Option<Bounds>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn element(handle: u64, parent: u64, index: u32) -> Change {
        Change::Added(Element {
            handle,
            parent,
            index,
            kind: format!("Kind{handle}"),
            ..Element::default()
        })
    }

    #[test]
    fn children_keep_the_order_they_were_given() {
        let mut tree = NativeTree::default();
        tree.apply(vec![element(1, 0, 0), element(3, 1, 0), element(2, 1, 0)]);

        assert_eq!(tree.roots(), [1]);
        assert_eq!(tree.children(1), [2, 3]);
        assert_eq!(tree.path(3), [1, 3]);
    }

    #[test]
    fn marked_elements_come_in_tree_order() {
        let marked = |handle: u64, parent: u64, index: u32, mark: &str| {
            Change::Added(Element {
                handle,
                parent,
                index,
                mark: mark.into(),
                ..Element::default()
            })
        };
        let mut tree = NativeTree::default();
        tree.apply(vec![
            marked(1, 0, 0, "Shell"),
            marked(2, 1, 0, ""),
            marked(3, 2, 0, "Page"),
            marked(4, 1, 1, "Page"),
        ]);

        assert_eq!(tree.marked("Page"), [3, 4]);
        assert_eq!(tree.marked("Shell"), [1]);
        assert!(tree.marked("Other").is_empty());
    }

    #[test]
    fn removing_an_element_takes_its_subtree() {
        let mut tree = NativeTree::default();
        tree.apply(vec![element(1, 0, 0), element(2, 1, 0), element(3, 2, 0)]);

        tree.apply(vec![Change::Removed {
            handle: 2,
            parent: 1,
        }]);

        assert_eq!(tree.len(), 1);
        assert!(tree.children(1).is_empty());
        assert!(tree.get(3).is_none());
    }

    #[test]
    fn an_element_added_again_moves() {
        let mut tree = NativeTree::default();
        tree.apply(vec![element(1, 0, 0), element(2, 0, 1), element(3, 1, 0)]);

        tree.apply(vec![element(3, 2, 0)]);

        assert!(tree.children(1).is_empty());
        assert_eq!(tree.children(2), [3]);
    }
}
