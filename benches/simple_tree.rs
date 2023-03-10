
use criterion::{BenchmarkId, black_box, Criterion, criterion_group};
use craft_forest::tree::simple::Tree;
use crate::criterion;

pub fn add_node(c: &mut Criterion) {
    c.bench_function("SimpleTree::add_root", |b| b.iter_with_setup(
        || Tree::new(),
        |mut tree| {
            tree.add_root(black_box(5))
        }
    ));
    c.bench_function("SimpleTree::add_child", |b| b.iter_with_setup(
        || {
            let mut t = Tree::new();
            let root = t.add_root(0);
            (t, root)
        },
        |(mut tree, root)| {
            tree.add_child(5, root)
        }
    ));
}

pub fn remove_node(c: &mut Criterion) {
    let mut group = c.benchmark_group("SimpleTree::remove_recursive");
    group.bench_function(BenchmarkId::from_parameter("single_root"), |b| {
        b.iter_with_setup(
            || {
                let mut t = Tree::new();
                let root = t.add_root(5);
                (t, root)
            },
            |(mut tree, root)| {
                tree.remove_recursive(root)
            }
        )
    });
    group.bench_function(BenchmarkId::from_parameter("root_with_children"), |b| {
        b.iter_with_setup(
            || {
                let mut t = Tree::new();
                let root = t.add_root(0);
                for i in 1..10 {
                    t.add_child(i, root);
                }
                (t, root)
            },
            |(mut tree, root)| {
                tree.remove_recursive(root)
            }
        )
    });
    group.bench_function(BenchmarkId::from_parameter("single_child"), |b| {
        b.iter_with_setup(
            || {
                let mut t = Tree::new();
                let root = t.add_root(0);
                let child = t.add_child(1, root)
                    .unwrap();
                (t, child)
            },
            |(mut tree, child)| {
                tree.remove_recursive(child)
            }
        )
    });
    group.bench_function(BenchmarkId::from_parameter("child_with_children"), |b| {
        b.iter_with_setup(
            || {
                let mut t = Tree::new();
                let root = t.add_root(0);
                let child = t.add_child(1, root)
                    .unwrap();
                for i in 1..10 {
                    t.add_child(i, child);
                }
                (t, child)
            },
            |(mut tree, child)| {
                tree.remove_recursive(child)
            }
        )
    });
}

criterion_group!(
    name = simple_tree;
    config = criterion();
    targets = add_node, remove_node
);

