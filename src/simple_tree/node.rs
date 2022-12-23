use std::ops::{Deref, DerefMut};
use crate::simple_tree::TreeKey;

pub struct Node<T: ?Sized> {
    parent: Option<TreeKey>,
    children: Vec<TreeKey>,
    val: T,
}

impl<T: ?Sized> Node<T> {
    pub(crate) fn children_mut(&mut self) -> &mut Vec<TreeKey> {
        &mut self.children
    }

    pub(crate) fn set_parent(&mut self, parent: Option<TreeKey>) {
        self.parent = parent;
    }

    pub fn parent(&self) -> Option<TreeKey> {
        self.parent
    }

    pub fn children(&self) -> &[TreeKey] {
        &self.children
    }

    pub fn val(&self) -> &T {
        &self.val
    }

    pub fn val_mut(&mut self) -> &mut T {
        &mut self.val
    }
}

impl<T> Node<T> {
    pub(crate) fn new(val: T, parent: Option<TreeKey>) -> Node<T> {
        Node {
            parent,
            children: Vec::new(),
            val,
        }
    }
}

impl<T: ?Sized> Deref for Node<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

impl<T: ?Sized> DerefMut for Node<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}
