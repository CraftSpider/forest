use slotmap::{new_key_type, SlotMap};
use alloc::vec::Vec;

new_key_type! {
    /// Key for a node in a tree. Altering the tree will not invalidate the key, as long
    /// as the node it references isn't removed
    pub struct TreeKey;
}

struct Node<T> {
    parent: Option<TreeKey>,
    children: Vec<TreeKey>,
    val: T,
}

impl<T> Node<T> {
    fn new(val: T, parent: Option<TreeKey>) -> Node<T> {
        Node {
            parent,
            children: Vec::new(),
            val,
        }
    }
}

pub struct Tree<T> {
    nodes: SlotMap<TreeKey, Node<T>>,
    roots: Vec<TreeKey>,
}

impl<T> Tree<T> {
    pub fn new() -> Tree<T> {
        Tree::default()
    }

    /// Get the length of this tree, the total number of nodes
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check whether this tree is empty (contains no nodes)
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn add_root(&mut self, val: T) -> TreeKey {
        let new_root = self.nodes.insert(Node::new(val, None));
        self.roots.push(new_root);
        new_root
    }

    pub fn add_child(&mut self, val: T, parent: TreeKey) -> Option<TreeKey> {
        if !self.nodes.contains_key(parent) {
            return None;
        }
        let new_child = self.nodes.insert(Node::new(val, Some(parent)));
        self.nodes.get_mut(parent)?.children.push(new_child);
        Some(new_child)
    }

    /// Set the first node as the parent of the second node,
    /// unsetting the current parent if there is one
    pub fn set_child(&mut self, parent: TreeKey, child: TreeKey) -> Option<()> {
        let old_parent = self.nodes.get(child)?.parent;

        // Remove child's existing parent (remove it as a root, if it had no parent)
        match old_parent {
            Some(old_parent) => {
                let old_parent = self.nodes.get_mut(old_parent)?;
                old_parent.children.retain(|&k| k != child)
            },
            None => self.roots.retain(|&k| k != child),
        }

        self.nodes.get_mut(child)?.parent = Some(parent);
        self.nodes.get_mut(parent)?.children.push(child);

        Some(())
    }

    /// Remove the second node as a child of the first node
    pub fn remove_child(&mut self, parent: TreeKey, child: TreeKey) -> Option<()> {
        let parent = self.nodes.get_mut(parent)?;
        parent.children.retain(|&k| k != child);
        let child_node = self.nodes.get_mut(child)?;
        child_node.parent = None;
        self.roots.push(child);
        Some(())
    }

    /// Remove a node from the tree, removing all children as well. Fails if the node or any
    /// of its children are currently borrowed.
    pub fn remove_recursive(&mut self, node_id: TreeKey) -> Option<()> {
        let node = self.nodes.remove(node_id)?;

        for child in node.children {
            drop(self.remove_recursive(child));
        }

        if let Some(parent) = node.parent {
            self.remove_child(parent, node_id);
        } else {
            self.roots.retain(|&k| k != node_id);
        }

        Some(())
    }

    /// Try to get an immutable reference to a node identified by the provided key
    pub fn try_get(&self, key: TreeKey) -> Option<&T> {
        self.nodes.get(key).map(|node| &node.val)
    }

    /// Try to get a mutable reference to a node identified by the provided key
    pub fn try_get_mut(&mut self, key: TreeKey) -> Option<&mut T> {
        self.nodes.get_mut(key).map(|node| &mut node.val)
    }

    /// Iterate over all nodes in this tree, in no particular order
    pub fn unordered_iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.nodes
            .iter()
            .map(|(_, item)| {
                &item.val
            })
    }

    /// Iterate over all nodes in this tree mutably, in no particular order
    pub fn unordered_iter_mut(&mut self) -> impl Iterator<Item = &mut T> + '_ {
        self.nodes
            .iter_mut()
            .map(|(_, item)| &mut item.val)
    }

    /// Iterator over the keys of all nodes in this tree, in no particular order
    pub fn unordered_keys(&self) -> impl Iterator<Item = TreeKey> {
        self.nodes
            .keys()
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// Iterate over the roots of this tree.
    ///
    /// A root is any node that has no parent
    pub fn roots(&self) -> impl Iterator<Item = &T> + '_ {
        self.roots
            .iter()
            .map(|key| {
                &self.nodes.get(*key).unwrap().val
            })
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// Iterator over the roots of this tree mutable
    ///
    /// A root is any node that has no parent
    pub fn roots_mut(&mut self) -> impl Iterator<Item = &mut T> + '_ {
        self.roots
            .iter()
            .map(|key| {
                let node_val = &mut self.nodes.get_mut(*key).unwrap().val;
                // SAFETY: We guarantee items in `roots` are unique
                unsafe { &mut *(node_val as *mut T) }
            })
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// Iterate over the keys of all the roots in this tree
    ///
    /// A root is any node that has no parent
    pub fn root_keys(&self) -> impl Iterator<Item = TreeKey> {
        self.roots.clone().into_iter()
    }

    /// Get the parent key of a node identified by the provided key
    pub fn parent_key_of(&self, child: TreeKey) -> Option<TreeKey> {
        self.nodes.get(child)?.parent
    }

    /// Get the child keys of a node identified by the provided key
    pub fn child_keys_of(&self, parent: TreeKey) -> Option<impl Iterator<Item = TreeKey>> {
        Some(self.nodes
            .get(parent)?
            .children
            .clone()
            .into_iter())
    }
}

impl<T> Default for Tree<T> {
    fn default() -> Self {
        Tree {
            nodes: SlotMap::with_key(),
            roots: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tree() {
        let tree = Tree::<()>::new();
        assert_eq!(tree.len(), 0);
        assert_eq!(tree.roots().collect::<Vec<_>>().len(), 0);
    }

    #[test]
    fn tree_roots() {
        let mut tree = Tree::new();
        tree.add_root(true);
        tree.add_root(false);

        assert_eq!(tree.len(), 2);
        assert_eq!(tree.roots().collect::<Vec<_>>().len(), 2);
    }

    #[test]
    fn tree_nodes() {
        let mut tree = Tree::new();
        let root = tree.add_root(true);

        {
            tree.add_child(true, root);
            tree.add_child(false, root);
        }

        assert_eq!(tree.len(), 3);

        let roots = tree.roots().collect::<Vec<_>>();

        assert_eq!(roots.len(), 1);
        assert_eq!(*roots[0], true);

        let children = tree.child_keys_of(root).unwrap().collect::<Vec<_>>();

        assert_eq!(children.len(), 2);
    }
}
