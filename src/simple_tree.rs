mod tree;
mod node;
mod node_ref;

pub use node::Node;
pub use node_ref::{NodeRef, NodeMut, NodeMutLimited};
pub use tree::{Tree, TreeKey};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traverse_parent() {
        let mut tree = Tree::new();
        let root = tree.add_root(0);
        let child1 = tree.add_child(1, root)
            .unwrap();
        let child2 = tree.add_child(2, child1)
            .unwrap();

        let r1 = tree.try_get(child2)
            .unwrap();
        assert_eq!(*r1, 2);
        let r2 = r1.traverse_parent()
            .unwrap();
        assert_eq!(*r2, 1);
        let r3 = r2.traverse_parent()
            .unwrap();
        assert_eq!(*r3, 0);
        assert!(r3.traverse_parent().is_none());

        assert_eq!(*r1, 2);
    }

    #[test]
    fn test_traverse_parent_mut() {
        let mut tree = Tree::new();
        let root = tree.add_root(0);
        let child1 = tree.add_child(1, root)
            .unwrap();
        let child2 = tree.add_child(2, child1)
            .unwrap();

        let mut r1 = tree.try_get_mut(child2)
            .unwrap();
        assert_eq!(*r1, 2);
        let mut r2 = r1.traverse_parent_mut()
            .unwrap();
        assert_eq!(*r2, 1);
        let mut r3 = r2.traverse_parent_mut()
            .unwrap();
        assert_eq!(*r3, 0);
        assert!(r3.traverse_parent_mut().is_none());

        assert_eq!(*r1, 2);
    }
}
