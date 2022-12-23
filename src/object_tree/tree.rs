
use super::error::{Error, Result};
use super::{NodeRef, NodeRefMut};

use core::fmt;
#[cfg(feature = "unstable")]
use core::marker::Unsize;
use alloc::vec::Vec;
use slotmap::{new_key_type, SlotMap, SecondaryMap};
use crate::object_tree::{Stable, Cell};

new_key_type! {
    /// Key for a node in a tree. Altering the tree will not invalidate the key, as long
    /// as the node it references isn't removed
    pub struct TreeKey;
}

/// An implementation of a tree data structure, with the ability to get mutable references to
/// multiple nodes at once. Supports access via slot keys, or by traversing immutable or mutable
/// node references.
pub struct Tree<T: ?Sized> {
    nodes: Cell<SlotMap<TreeKey, Stable<T>>>,
    parents: Cell<SecondaryMap<TreeKey, TreeKey>>,
    children: Cell<SecondaryMap<TreeKey, Vec<TreeKey>>>,
    roots: Cell<Vec<TreeKey>>,
}

impl<T: ?Sized> Tree<T> {
    /// Create a new tree
    #[must_use]
    pub fn new() -> Tree<T> {
        Tree::default()
    }

    /// Get the length of this tree, the total number of nodes
    pub fn len(&self) -> usize {
        self.nodes.borrow().len()
    }

    /// Check whether this tree is empty (contains no nodes)
    pub fn is_empty(&self) -> bool {
        self.nodes.borrow().is_empty()
    }

    /// Add a new root from a type that unsizes into the type of the tree
    #[cfg(feature = "unstable")]
    pub fn add_root_from<U: Unsize<T>>(&self, item: U) -> TreeKey {
        let mut nodes = self.nodes.borrow_mut();

        let cell = Stable::new_from(item);

        let new_key = nodes.insert(cell);
        
        self.roots.borrow_mut().push(new_key);

        new_key
    }

    /// Create a new child of a node from a type that unsizes into the type of the tree
    #[cfg(feature = "unstable")]
    pub fn new_child_from<U: Unsize<T>>(&self, item: U, parent: TreeKey) -> Option<TreeKey> {
        let cell = Stable::new_from(item);

        let new_key = self.nodes
            .borrow_mut()
            .insert(cell);

        self.children
            .borrow_mut()
            .entry(parent)?
            .or_default()
            .push(new_key);

        self.parents
            .borrow_mut()
            .insert(new_key, parent);

        Some(new_key)
    }

    /// Set the first node as the parent of the second node,
    /// unsetting the current parent if there is one
    pub fn set_child(&self, parent: TreeKey, child: TreeKey) {
        let mut children = self.children.borrow_mut();
        let mut parents = self.parents.borrow_mut();

        let old_parent = parents.get(child);

        // Remove child's existing parent (remove it as a root, if it had no parent)
        match old_parent {
            Some(&old_parent) => children[old_parent].retain(|&k| k != child),
            None => self.roots.borrow_mut().retain(|&k| k != child),
        }

        parents.insert(child, parent);
        children
            .entry(parent)
            .unwrap()
            .or_default()
            .push(child);
    }

    /// Remove the second node as a child of the first node
    pub fn remove_child(&self, parent: TreeKey, child: TreeKey) {
        self.children.borrow_mut()[parent].retain(|&k| k != child);
        self.parents.borrow_mut().remove(child);
        self.roots.borrow_mut().push(child);
    }

    /// Remove a node from the tree, removing all children as well. Fails if the node or any
    /// of its children are currently borrowed.
    pub fn remove_node_recursive(&self, node: TreeKey) {
        let mut nodes = self.nodes
            .borrow_mut();
        let mut children = self.children
            .borrow_mut();
        let mut parents = self.parents
            .borrow_mut();

        recurse_remove(node, &mut nodes, &mut parents, &mut children)
    }

    /// Try to get an immutable reference to a node identified by the provided key
    pub fn try_get<'b>(&self, key: TreeKey) -> Result<NodeRef<'_, 'b, T>> {
        let nodes = self.nodes.borrow();
        let rc = nodes.get(key).ok_or(Error::Missing)?;
        NodeRef::try_borrow(self, key, rc)
    }

    /// Try to get a mutable reference to a node identified by the provided key
    pub fn try_get_mut<'b>(&self, key: TreeKey) -> Result<NodeRefMut<'_, 'b, T>> {
        let nodes = self.nodes.borrow();
        let rc = nodes.get(key).ok_or(Error::Missing)?;
        NodeRefMut::try_borrow(self, key, rc)
    }

    /// Iterate over all nodes in this tree, in no particular order
    pub fn unordered_iter(&self) -> impl Iterator<Item = Result<NodeRef<'_, '_, T>>> {
        self.nodes
            .borrow()
            .iter()
            .map(|(key, item)| {
                NodeRef::try_borrow(self, key, item)
            })
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// Iterate over all nodes in this tree mutably, in no particular order
    pub fn unordered_iter_mut(&self) -> impl Iterator<Item = Result<NodeRefMut<'_, '_, T>>> {
        self.nodes
            .borrow()
            .iter()
            .map(|(key, item)| NodeRefMut::try_borrow(self, key, item))
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// Iterator over the keys of all nodes in this tree, in no particular order
    pub fn unordered_keys(&self) -> impl Iterator<Item = TreeKey> {
        self.nodes
            .borrow()
            .keys()
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// Iterate over the roots of this tree.
    ///
    /// A root is any node that has no parent
    pub fn roots(&self) -> impl Iterator<Item = Result<NodeRef<'_, '_, T>>> + '_ {
        let nodes = self.nodes.borrow();

        self.roots
            .borrow()
            .iter()
            .map(|key| {
                let node = nodes.get(*key).ok_or(Error::Missing)?;
                NodeRef::try_borrow(self, *key, node)
            })
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// Iterator over the roots of this tree mutable
    ///
    /// A root is any node that has no parent
    pub fn roots_mut(&self) -> impl Iterator<Item = Result<NodeRefMut<'_, '_, T>>> + '_ {
        let nodes = self.nodes.borrow();

        self.roots
            .borrow()
            .iter()
            .map(|key| {
                let node = nodes.get(*key).ok_or(Error::Missing)?;
                NodeRefMut::try_borrow(self, *key, node)
            })
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// Iterate over the keys of all the roots in this tree
    ///
    /// A root is any node that has no parent
    pub fn root_keys(&self) -> impl Iterator<Item = TreeKey> {
        self.roots.borrow().clone().into_iter()
    }

    /// Get the parent key of a node identified by the provided key
    pub fn parent_key_of(&self, child: TreeKey) -> Option<TreeKey> {
        self.parents.borrow().get(child).copied()
    }

    /// Get the child keys of a node identified by the provided key
    pub fn child_keys_of(&self, parent: TreeKey) -> impl Iterator<Item = TreeKey> {
        self.children
            .borrow()
            .get(parent)
            .cloned()
            .unwrap_or_default()
            .into_iter()
    }
}

impl<T> Tree<T> {
    /// Create a new child of a node from the provided value
    pub fn new_child(&self, item: T, parent: TreeKey) {
        let cell = Stable::new(item);

        let new_key = self.nodes.borrow_mut().insert(cell);

        self.children
            .borrow_mut()
            .entry(parent)
            .unwrap()
            .or_default()
            .push(new_key);

        self.parents
            .borrow_mut()
            .insert(new_key, parent);
    }

    /// Add a new root to the tree initialized with the provided value
    pub fn add_root(&self, item: T) -> TreeKey {
        let mut nodes = self.nodes.borrow_mut();

        let cell = Stable::new(item);

        let new_key = nodes.insert(cell);

        self.roots.borrow_mut().push(new_key);

        new_key
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for Tree<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for node in self.roots() {
            recurse_tree(f, 0, node)?;
        }
        Ok(())
    }
}

impl<T: ?Sized> Default for Tree<T> {
    fn default() -> Self {
        Tree {
            nodes: Cell::new(SlotMap::with_key()),
            parents: Cell::new(SecondaryMap::new()),
            children: Cell::new(SecondaryMap::new()),
            roots: Cell::new(Vec::new()),
        }
    }
}

fn recurse_tree<T: ?Sized + fmt::Debug>(
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

fn recurse_remove<T: ?Sized>(
    node: TreeKey,
    nodes: &mut SlotMap<TreeKey, Stable<T>>,
    parents: &mut SecondaryMap<TreeKey, TreeKey>,
    children: &mut SecondaryMap<TreeKey, Vec<TreeKey>>,
) {
    nodes.remove(node);
    parents.remove(node);
    if let Some(node_children) = children.remove(node) {
        for child in node_children {
            recurse_remove(child, nodes, parents, children)
        }
    }
}
