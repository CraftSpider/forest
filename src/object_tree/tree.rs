
use super::error::{Error, Result};
use super::{NodeRef, NodeRefMut};

use core::fmt;
use core::ptr::NonNull;
use core::cell::RefCell;
#[cfg(feature = "unstable")]
use core::marker::Unsize;
use alloc::vec::Vec;
use alloc::boxed::Box;
use slotmap::{new_key_type, SlotMap, SecondaryMap};
use typed_arena::Arena;
use crate::stable_map::StableMap;

new_key_type! {
    /// Key for a node in a tree. Altering the tree will not invalidate the key, as long
    /// as the node it references isn't removed
    pub struct TreeKey;
}

/// An implementation of a tree data structure, with the ability to get mutable references to
/// multiple nodes at once. Supports access via slot keys, or by traversing immutable or mutable
/// node references.
#[derive(Clone)]
pub struct Tree<T: /* ?Sized */> {
    inner: RefCell<InnerTree<T>>,
}

impl<T: /* ?Sized */> Tree<T> {
    /// Create a new tree
    #[must_use]
    pub fn new() -> Tree<T> {
        Tree::default()
    }

    /// Get the length of this tree, the total number of nodes
    pub fn len(&self) -> usize {
        self.inner.borrow().nodes.len()
    }

    /// Check whether this tree is empty (contains no nodes)
    pub fn is_empty(&self) -> bool {
        self.inner.borrow().nodes.is_empty()
    }

    /// Add a new root from a type that unsizes into the type of the tree
    #[cfg(feature = "unstable")]
    pub fn add_root_from<U: Unsize<T>>(&self, item: U) -> TreeKey {
        let mut rc = self.inner.borrow_mut();

        let new_node = RefCell::new(item);

        let new_node =
            unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(new_node) as Box<RefCell<T>>)) };

        let new_key = rc.nodes.insert(new_node);
        rc.roots.push(new_key);
        new_key
    }

    /// Create a new child of a node from a type that unsizes into the type of the tree
    #[cfg(feature = "unstable")]
    pub fn new_child_from<U: Unsize<T>>(&self, item: U, parent: TreeKey) {
        self.inner.borrow_mut().new_child_from(item, parent);
    }

    /// Set the first node as the parent of the second node,
    /// unsetting the current parent if there is one
    pub fn set_child(&self, parent: TreeKey, child: TreeKey) {
        self.inner.borrow_mut().set_child(parent, child);
    }

    /// Remove the second node as a child of the first node
    pub fn remove_child(&self, parent: TreeKey, child: TreeKey) {
        self.inner.borrow_mut().remove_child(parent, child);
    }

    /// Remove a node from the tree, removing all children as well. Fails if the node or any
    /// of its children are currently borrowed.
    pub fn remove_node_recursive(&self, node: TreeKey) -> Result<()> {
        self.inner.borrow_mut()
            .remove_node_recursive(node)
    }

    /// Try to get an immutable reference to a node identified by the provided key
    pub fn try_get<'a, 'b>(&'a self, key: TreeKey) -> Result<NodeRef<'a, 'b, T>> {
        let inner = self.inner.borrow();
        let rc = inner.nodes.get(key).ok_or(Error::Missing)?;
        NodeRef::try_borrow(self, key, rc)
    }

    /// Try to get a mutable reference to a node identified by the provided key
    pub fn try_get_mut<'a, 'b>(&'a self, key: TreeKey) -> Result<NodeRefMut<'a, 'b, T>> {
        let inner = self.inner.borrow();
        let rc = inner.nodes.get(key).ok_or(Error::Missing)?;
        NodeRefMut::try_borrow(self, key, rc)
    }

    /// Iterate over all nodes in this tree, in no particular order
    pub fn unordered_iter(&self) -> impl Iterator<Item = Result<NodeRef<'_, '_, T>>> {
        self.inner
            .borrow()
            .nodes
            .iter()
            .map(|(key, item)| NodeRef::try_borrow(self, key, item))
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// Iterate over all nodes in this tree mutably, in no particular order
    pub fn unordered_iter_mut(&self) -> impl Iterator<Item = Result<NodeRefMut<'_, '_, T>>> {
        self.inner
            .borrow()
            .nodes
            .iter()
            .map(|(key, item)| NodeRefMut::try_borrow(self, key, item))
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// Iterator over the keys of all nodes in this tree, in no particular order
    pub fn unordered_keys(&self) -> impl Iterator<Item = TreeKey> {
        self.inner
            .borrow()
            .nodes
            .keys()
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// Iterate over the roots of this tree.
    ///
    /// A root is any node that has no parent
    pub fn roots<'a>(&'a self) -> impl Iterator<Item = Result<NodeRef<'a, '_, T>>> + 'a {
        let inner = self.inner.borrow();

        inner
            .roots
            .iter()
            .map(|key| {
                let node = inner.nodes.get(*key).ok_or(Error::Missing)?;
                NodeRef::try_borrow(self, *key, node)
            })
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// Iterator over the roots of this tree mutable
    ///
    /// A root is any node that has no parent
    pub fn roots_mut<'a>(&'a self) -> impl Iterator<Item = Result<NodeRefMut<'a, '_, T>>> {
        let inner = self.inner.borrow();

        inner
            .roots
            .iter()
            .map(|key| {
                let node = inner.nodes.get(*key).ok_or(Error::Missing)?;
                NodeRefMut::try_borrow(self, *key, node)
            })
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// Iterate over the keys of all the roots in this tree
    ///
    /// A root is any node that has no parent
    pub fn root_keys(&self) -> impl Iterator<Item = TreeKey> {
        self.inner
            .borrow()
            .roots
            .clone()
            .into_iter()
    }

    /// Get the parent key of a node identified by the provided key
    pub fn parent_key_of(&self, child: TreeKey) -> Option<TreeKey> {
        self.inner.borrow().parents.get(child).copied()
    }

    /// Get the child keys of a node identified by the provided key
    pub fn child_keys_of(&self, parent: TreeKey) -> impl Iterator<Item = TreeKey> {
        self.inner
            .borrow()
            .children
            .get(parent)
            .cloned()
            .unwrap_or_default()
            .into_iter()
    }
}

impl<T> Tree<T> {
    /// Create a new child of a node from the provided value
    pub fn new_child(&self, item: T, parent: TreeKey) {
        self.inner.borrow_mut().new_child(item, parent);
    }

    /// Add a new root to the tree initialized with the provided value
    pub fn add_root(&self, item: T) -> TreeKey {
        let mut rc = self.inner.borrow_mut();

        let new_node = RefCell::new(item);

        let new_node = unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(new_node))) };

        let new_key = rc.nodes.insert(new_node);
        rc.roots.push(new_key);
        new_key
    }
}

fn recurse_tree<T: /* ?Sized + */ fmt::Debug>(
    f: &mut fmt::Formatter<'_>,
    indent: usize,
    node: Result<NodeRef<'_, '_, T>>,
) -> fmt::Result {
    match node {
        Ok(node) => {
            writeln!(f, "{}Node {{ {:?} }}", " ".repeat(indent), &*node)?;
            for child in node.children() {
                recurse_tree(f, indent + 4, child)?;
            }
        }
        Err(_) => writeln!(f, "{}Node {{ (Borrowed) }}", " ".repeat(indent))?,
    }
    Ok(())
}

impl<T: /* ?Sized + */ fmt::Debug> fmt::Debug for Tree<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for node in self.roots() {
            recurse_tree(f, 0, node)?;
        }
        Ok(())
    }
}

impl<T: /* ?Sized */> Default for Tree<T> {
    fn default() -> Self {
        Tree {
            inner: RefCell::new(InnerTree::new()),
        }
    }
}

struct Node<T> {
    idx: u32,
    item: T,
}

#[derive(Clone, Debug)]
struct InnerTree<T> {
    nodes: SlotMap<TreeKey, NonNull<RefCell<T>>>,
    parents: SecondaryMap<TreeKey, TreeKey>,
    children: SecondaryMap<TreeKey, Vec<TreeKey>>,
    roots: Vec<TreeKey>,
}

impl<T: /* ?Sized */> InnerTree<T> {
    fn new() -> InnerTree<T> {
        InnerTree {
            nodes: SlotMap::with_key(),
            parents: SecondaryMap::new(),
            children: SecondaryMap::new(),
            roots: Vec::new(),
        }
    }

    #[cfg(feature = "unstable")]
    fn new_child_from<U: Unsize<T>>(&mut self, item: U, parent: TreeKey) {
        let new_node = RefCell::from(item);

        // SAFETY: Box::into_raw is guaranteed to return non-null pointer
        let new_node =
            unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(new_node) as Box<RefCell<T>>)) };

        let new_key = self.nodes.insert(new_node);

        self.children
            .entry(parent)
            .unwrap()
            .or_default()
            .push(new_key);

        self.parents.insert(new_key, parent);
    }

    fn set_child(&mut self, parent: TreeKey, child: TreeKey) {
        let old_parent = self.parents.get(child);

        // Remove child's existing parent (remove it as a root, if it had no parent)
        match old_parent {
            Some(&old_parent) => self.children[old_parent].retain(|&k| k != child),
            None => self.roots.retain(|&k| k != child),
        }

        self.parents.insert(child, parent);
        self.children
            .entry(parent)
            .unwrap()
            .or_default()
            .push(child);
    }

    fn remove_child(&mut self, parent: TreeKey, child: TreeKey) {
        self.children[parent].retain(|&k| k != child);
        self.parents.remove(child);
        self.roots.push(child);
    }

    fn remove_node_recursive(&mut self, node: TreeKey) -> Result<()> {
        let node = unsafe { self.nodes[node].as_ref().try_borrow_mut()? };
        todo!()
    }
}

impl<T> InnerTree<T> {
    fn new_child(&mut self, item: T, parent: TreeKey) {
        let new_node = RefCell::new(item);

        // SAFETY: Box::into_raw is guaranteed to return non-null pointer
        let new_node = unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(new_node))) };

        let new_key = self.nodes.insert(new_node);

        self.children
            .entry(parent)
            .unwrap()
            .or_default()
            .push(new_key);

        self.parents.insert(new_key, parent);
    }
}

impl<T: /* ?Sized */> Drop for InnerTree<T> {
    fn drop(&mut self) {
        for i in self.nodes.values() {
            unsafe {
                Box::from_raw(i.as_ptr());
            }
        }
    }
}
