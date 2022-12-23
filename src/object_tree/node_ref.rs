
use super::{Tree, TreeKey};
use super::error::Result;

use core::fmt;
use core::ops::{Deref, DerefMut};
use core::borrow::{Borrow, BorrowMut};
#[cfg(feature = "unstable")]
use core::marker::Unsize;
use alloc::vec::Vec;
use crate::object_tree::{Error, Stable, StableRef, StableMut};

macro_rules! ref_common {
    ($ty:ty) => {
        impl<'a, 'b, T: ?Sized> $ty {
            /// Get the key of this node
            #[must_use]
            pub fn key(&self) -> TreeKey {
                self.mykey
            }

            /// Attempt to get a reference to the parent of this node
            pub fn parent(&self) -> Result<Option<NodeRef<'a, 'b, T>>> {
                self.tree
                    .parent_key_of(self.key())
                    .map(|key| self.tree.try_get(key))
                    .transpose()
            }

            /// Attempt to get a mutable reference to the parent of this node
            pub fn parent_mut(&self) -> Result<Option<NodeRefMut<'a, 'b, T>>> {
                self.tree
                    .parent_key_of(self.key())
                    .map(|key| self.tree.try_get_mut(key))
                    .transpose()
            }

            /// Attempt to get references to the children of this node
            pub fn children(&self) -> impl Iterator<Item = Result<NodeRef<'a, 'b, T>>> {
                self.tree
                    .child_keys_of(self.key())
                    .map(|key| self.tree.try_get(key))
                    .collect::<Vec<_>>()
                    .into_iter()
            }

            /// Attempt to get mutable references to the children of this node
            pub fn children_mut(&self) -> impl Iterator<Item = Result<NodeRefMut<'a, 'b, T>>> {
                self.tree
                    .child_keys_of(self.key())
                    .map(|key| self.tree.try_get_mut(key))
                    .collect::<Vec<_>>()
                    .into_iter()
            }
        }

        impl<'a, 'b, T: ?Sized> Deref for $ty {
            type Target = T;

            fn deref(&self) -> &Self::Target {
                &*self.node
            }
        }

        impl<'a, 'b, T: ?Sized> AsRef<T> for $ty {
            fn as_ref(&self) -> &T {
                &*self.node
            }
        }

        impl<'a, 'b, T: ?Sized> Borrow<T> for $ty {
            fn borrow(&self) -> &T {
                &*self.node
            }
        }
    };
}

/// A reference to a node in a [`Tree`], with helpers to traverse nodes relative to this one
pub struct NodeRef<'a, 'b, T: ?Sized> {
    tree: &'a Tree<T>,
    mykey: TreeKey,
    node: StableRef<'b, T>,
}

ref_common! { NodeRef<'a, 'b, T> }

impl<'a, 'b, T: ?Sized> NodeRef<'a, 'b, T> {
    pub(super) fn try_borrow(
        tree: &'a Tree<T>,
        key: TreeKey,
        cell: &'_ Stable<T>,
    ) -> Result<NodeRef<'a, 'b, T>> {
        Ok(NodeRef {
            tree,
            mykey: key,
            node: cell.try_borrow().ok_or(Error::CantBorrow)?,
        })
    }

    /// Attempt to promote this immutable ref into a mutable ref
    pub fn try_promote(self) -> Result<NodeRefMut<'a, 'b, T>> {
        drop(self.node);
        self.tree.try_get_mut(self.mykey)
    }

    /// Promote this immutable ref into a mutable ref, panicking on failure
    pub fn promote(self) -> NodeRefMut<'a, 'b, T> {
        drop(self.node);
        self.tree.try_get_mut(self.mykey)
            .expect("Could not promote immutable ref")
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for NodeRef<'_, '_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeRef")
            .field("mykey", &self.mykey)
            .field("node", &self.node)
            .finish()
    }
}

/// A mutable reference to a node in a [`Tree`], with helpers to traverse nodes relative to this
/// one as well as alter the node's relationships.
pub struct NodeRefMut<'a, 'b, T: ?Sized> {
    tree: &'a Tree<T>,
    mykey: TreeKey,
    node: StableMut<'b, T>,
}

ref_common! { NodeRefMut<'a, 'b, T> }

impl<'a, 'b, T: ?Sized> NodeRefMut<'a, 'b, T> {
    pub(super) fn try_borrow(
        tree: &'a Tree<T>,
        key: TreeKey,
        cell: &'_ Stable<T>,
    ) -> Result<NodeRefMut<'a, 'b, T>> {
        Ok(NodeRefMut {
            tree,
            mykey: key,
            node: cell.try_borrow_mut().ok_or(Error::CantBorrow)?,
        })
    }

    /// Demote this mutable ref to an immutable ref
    pub fn demote(self) -> NodeRef<'a, 'b, T> {
        core::mem::drop(self.node);
        self.tree.try_get(self.mykey)
            .expect("This should always work, as we have unique access")
    }

    /// Create a new child of this node from a type that unsizes into the type of the tree
    #[cfg(feature = "unstable")]
    pub fn new_child_from<U: Unsize<T>>(&mut self, child: U) {
        self.tree.new_child_from(child, self.key());
    }

    /// Set the parent of this node, unsetting the current one as necessary
    pub fn set_parent(&mut self, parent: &NodeRef<'_, '_, T>) {
        self.tree.set_child(parent.key(), self.key());
    }

    /// Add a node as a child of this node, replacing its existing parent as necessary
    pub fn add_child(&mut self, child: &NodeRef<'_, '_, T>) {
        self.tree.set_child(self.key(), child.key());
    }

    /// Remove a node as a child of this node, turning it into a root node
    pub fn remove_child(&mut self, child: &NodeRef<'_, '_, T>) {
        self.tree.remove_child(self.key(), child.key());
    }
}

impl<T> NodeRefMut<'_, '_, T> {
    /// Create a new child of this node from the provided value
    pub fn new_child(&mut self, child: T) {
        self.tree.new_child(child, self.key());
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for NodeRefMut<'_, '_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeRefMut")
            .field("mykey", &self.mykey)
            .field("node", &self.node)
            .finish()
    }
}

impl<T: ?Sized> DerefMut for NodeRefMut<'_, '_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.node
    }
}

impl<T: ?Sized> AsMut<T> for NodeRefMut<'_, '_, T> {
    fn as_mut(&mut self) -> &mut T {
        &mut *self.node
    }
}

impl<T: ?Sized> BorrowMut<T> for NodeRefMut<'_, '_, T> {
    fn borrow_mut(&mut self) -> &mut T {
        &mut *self.node
    }
}
