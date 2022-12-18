//! A tree most similar to HTML's DOM. Every node can have data, as well as some number of child
//! nodes.
//!
//! ## Performance Characteristics
//!
//! |  Operation  |  Time  |
//! |-------------|--------|
//! | Adding Node | `O(1)` |
//!

//! Implementation of a nicely traversable tree that supports mutable references to multiple
//! nodes concurrently

mod error;
mod node_ref;
mod tree;

pub use error::Error;
pub use node_ref::{NodeRef, NodeRefMut};
pub use tree::{Tree, TreeKey};

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use super::*;
    use super::error::Result;

    #[test]
    fn new_tree() {
        let tree = Tree::<()>::new();
        assert_eq!(tree.len(), 0);
        assert_eq!(tree.roots().collect::<Vec<_>>().len(), 0);
    }

    #[test]
    fn tree_roots() {
        let tree = Tree::new();
        tree.add_root(true);
        tree.add_root(false);

        assert_eq!(tree.len(), 2);
        assert_eq!(tree.roots().collect::<Vec<_>>().len(), 2);
    }

    #[test]
    fn tree_nodes() {
        let tree = Tree::new();
        tree.add_root(true);

        {
            let mut root = tree.roots_mut().next().unwrap().unwrap();
            root.new_child(true);
            root.new_child(false);
        }

        assert_eq!(tree.len(), 3);

        let roots = tree.roots().collect::<Result<Vec<_>>>().unwrap();

        assert_eq!(roots.len(), 1);
        assert_eq!(*roots[0], true);

        let children = roots[0].children().collect::<Result<Vec<_>>>().unwrap();

        assert_eq!(children.len(), 2);
    }

    #[test]
    fn test_promote() {
        let tree = Tree::new();
        let id = tree.add_root(true);

        {
            let root = tree.try_get(id)
                .unwrap();

            let mut root = root.try_promote()
                .expect("Could promote unique reference");

            root.new_child(false);
        }

        {
            let root1 = tree.try_get(id)
                .unwrap();
            let _root2 = tree.try_get(id)
                .unwrap();

            root1.try_promote()
                .expect_err("Couldn't promote non-unique reference");
        }
    }

    #[test]
    fn test_demote() {
        let tree = Tree::new();
        let id = tree.add_root(true);

        {
            let mut root = tree.try_get_mut(id)
                .unwrap();

            root.new_child(false);

            let root = root.demote();
            assert_eq!(*root, true);
        }
    }
}
