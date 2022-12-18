
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use craft_forest::object_tree::Tree;

pub fn add_remove_node(c: &mut Criterion) {
    let tree = Tree::new();
    c.bench_function("add_root", |b| b.iter(|| {
        let new_node = tree.add_root(black_box(5));
        tree.remove_node_recursive(new_node);
    }));
}

criterion_group!(benches, add_node);

criterion_main!(benches);
