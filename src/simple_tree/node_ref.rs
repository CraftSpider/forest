use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use core::borrow::{Borrow, BorrowMut};
use crate::simple_tree::{Node, Tree, TreeKey};

macro_rules! impl_common {
    ($ty:ident) => {
        impl<T> $ty<'_, T> {
            pub fn parent(&self) -> Option<TreeKey> {
                self.node().parent()
            }

            pub fn children(&self) -> &[TreeKey] {
                self.node().children()
            }
        }

        impl<T> Deref for $ty<'_, T> {
            type Target = T;

            fn deref(&self) -> &Self::Target {
                self.node().val()
            }
        }

        impl<T> AsRef<T> for $ty<'_, T> {
            fn as_ref(&self) -> &T {
                self
            }
        }

        impl<T> Borrow<T> for $ty<'_, T> {
            fn borrow(&self) -> &T {
                self
            }
        }
    }
}

macro_rules! impl_mut {
    ($ty:ident) => {
        impl<T> DerefMut for $ty<'_, T> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.node_mut().val_mut()
            }
        }

        impl<T> AsMut<T> for $ty<'_, T> {
            fn as_mut(&mut self) -> &mut T {
                self
            }
        }

        impl<T> BorrowMut<T> for $ty<'_, T> {
            fn borrow_mut(&mut self) -> &mut T {
                self
            }
        }
    }
}

macro_rules! impl_traverse {
    ($ty:ident) => {
        impl<T> $ty<'_, T> {
            pub fn traverse_parent(&self) -> Option<NodeRef<'_, T>> {
                let parent_key = self.parent()?;
                self.tree().try_get(parent_key)
            }

            pub fn traverse_child(&self, child: TreeKey) -> Option<NodeRef<'_, T>> {
                if !self.children().contains(&child) {
                    return None;
                }

                self.tree().try_get(child)
            }

            pub fn traverse_children(&self) -> impl Iterator<Item = NodeRef<'_, T>> + '_ {
                self.children()
                    .iter()
                    .map(|&key| self.tree().try_get(key).unwrap())
            }
        }
    }
}

macro_rules! impl_traverse_mut {
    ($ty:ident) => {
        impl<T> $ty<'_, T> {
            pub fn traverse_parent_mut(&mut self) -> Option<NodeMut<'_, T>> {
                let parent_key = self.parent()?;
                self.node = None;
                self.tree_mut().try_get_mut(parent_key)
            }

            pub fn traverse_child_mut(&mut self, child: TreeKey) -> Option<NodeMut<'_, T>> {
                if !self.children().contains(&child) {
                    return None;
                }
                self.node = None;
                self.tree_mut().try_get_mut(child)
            }

            // pub fn traverse_children_mut(
            //     &mut self
            // ) -> impl Iterator<Item = NodeMutLimited<'_, T>> + '_ {
            //     self.children()
            //         .to_owned()
            //         .into_iter()
            //         .map(move |key| {
            //             let node = self.tree_mut().try_get_mut(key).unwrap().downgrade();
            //             core::mem::transmute::<NodeMutLimited<'_, T>, NodeMutLimited<'_, T>>(node)
            //         })
            // }
        }
    }
}

pub struct NodeRef<'a, T> {
    tree: &'a Tree<T>,
    node: &'a Node<T>,
}

impl<'a, T> NodeRef<'a, T> {
    pub(crate) fn new(tree: &'a Tree<T>, node: &'a Node<T>) -> NodeRef<'a, T> {
        NodeRef {
            tree,
            node,
        }
    }

    fn tree(&self) -> &Tree<T> {
        self.tree
    }

    fn node(&self) -> &Node<T> {
        self.node
    }
}

impl_common!(NodeRef);
impl_traverse!(NodeRef);

pub struct NodeMut<'a, T> {
    tree: NonNull<Tree<T>>,
    node: Option<NonNull<Node<T>>>,
    key: TreeKey,
    _phantom: PhantomData<&'a mut T>,
}

impl<'a, T> NodeMut<'a, T> {
    pub(crate) fn new(tree: NonNull<Tree<T>>, node: NonNull<Node<T>>, key: TreeKey) -> NodeMut<'a, T> {
        NodeMut {
            tree,
            node: Some(node),
            key,
            _phantom: PhantomData,
        }
    }

    fn downgrade(mut self) -> NodeMutLimited<'a, T> {
        let r = self.node_mut();
        let r = unsafe { &mut *(r as *mut Node<T>) };
        NodeMutLimited::new(r)
    }

    fn tree(&self) -> &Tree<T> {
        unsafe { self.tree.as_ref() }
    }

    fn tree_mut(&mut self) -> &mut Tree<T> {
        unsafe { self.tree.as_mut() }
    }

    fn node(&self) -> &Node<T> {
        match self.node {
            None => {
                self.tree()
                    .raw_nodes()
                    .get(self.key)
                    .unwrap()
            }
            Some(ptr) => unsafe { ptr.as_ref() },
        }
    }

    fn node_mut(&mut self) -> &mut Node<T> {
        match self.node {
            None => {
                let tree = unsafe { self.tree.as_mut() };
                let node = tree
                    .raw_nodes_mut()
                    .get_mut(self.key)
                    .unwrap();
                self.node = Some(NonNull::from(&mut *node));
                node
            }
            Some(mut ptr) => unsafe { ptr.as_mut() }
        }
    }
}

impl_common!(NodeMut);
impl_mut!(NodeMut);
impl_traverse!(NodeMut);
impl_traverse_mut!(NodeMut);

pub struct NodeMutLimited<'a, T> {
    node: &'a mut Node<T>,
}

impl<'a, T> NodeMutLimited<'a, T> {
    pub(crate) fn new(node: &'a mut Node<T>) -> NodeMutLimited<'a, T> {
        NodeMutLimited { node }
    }

    fn node(&self) -> &Node<T> {
        self.node
    }

    fn node_mut(&mut self) -> &mut Node<T> {
        self.node
    }
}

impl_common!(NodeMutLimited);
impl_mut!(NodeMutLimited);
