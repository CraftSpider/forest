use criterion::{BenchmarkId, black_box, Criterion, criterion_group};
use craft_forest::object_tree::Tree;
use crate::criterion;

pub fn add_node(c: &mut Criterion) {
    c.bench_function("ObjectTree::add_root", |b| b.iter_with_setup(
        || Tree::new(),
        |tree| {
            tree.add_root(black_box(5))
        }
    ));
    c.bench_function("ObjectTree::add_child", |b| b.iter_with_setup(
        || {
            let t = Tree::new();
            let root = t.add_root(0);
            (t, root)
        },
        |(tree, root)| {
            tree.add_child(5, root)
        }
    ));
}

pub fn remove_node(c: &mut Criterion) {
    let mut group = c.benchmark_group("ObjectTree::remove_recursive");
    group.bench_function(BenchmarkId::from_parameter("single_root"), |b| {
        b.iter_with_setup(
            || {
                let t = Tree::new();
                let root = t.add_root(5);
                (t, root)
            },
            |(tree, root)| {
                tree.remove_recursive(root)
            }
        )
    });
    group.bench_function(BenchmarkId::from_parameter("root_with_children"), |b| {
        b.iter_with_setup(
            || {
                let t = Tree::new();
                let root = t.add_root(0);
                for i in 1..10 {
                    t.add_child(i, root);
                }
                (t, root)
            },
            |(tree, root)| {
                tree.remove_recursive(root)
            }
        )
    });
    group.bench_function(BenchmarkId::from_parameter("single_child"), |b| {
        b.iter_with_setup(
            || {
                let t = Tree::new();
                let root = t.add_root(0);
                let child = t.add_child(1, root);
                (t, child)
            },
            |(tree, child)| {
                tree.remove_recursive(child)
            }
        )
    });
    group.bench_function(BenchmarkId::from_parameter("child_with_children"), |b| {
        b.iter_with_setup(
            || {
                let t = Tree::new();
                let root = t.add_root(0);
                let child = t.add_child(1, root);
                for i in 1..10 {
                    t.add_child(i, child);
                }
                (t, child)
            },
            |(tree, child)| {
                tree.remove_recursive(child)
            }
        )
    });
}

criterion_group!(
    name = object_tree;
    config = criterion();
    targets = add_node, remove_node
);
