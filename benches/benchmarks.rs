use std::cell::RefCell;
use std::rc::Rc;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pprof::criterion::{PProfProfiler, Output};
use craft_forest::object_tree::Tree;
use craft_forest::stable::cell::StableCell;

pub fn add_remove_node(c: &mut Criterion) {
    c.bench_function("add_root", |b| b.iter_with_setup(
        || Tree::new(),
        |tree| {
            let new_node = tree.add_root(black_box(5));
            tree.remove_node_recursive(new_node);
        }
    ));
}

pub fn stable_vs_rc(c: &mut Criterion) {
    c.bench_function("Rc<RefCell<i32>>", |b| {
        b.iter_with_setup(
            || Rc::new(RefCell::new(1)),
            |val| {
                let cur = *black_box(&val).try_borrow().unwrap();
                *black_box(&val).try_borrow_mut().unwrap() = (cur + 1) % 10;
            }
        )
    });
    c.bench_function("StableCell<i32>", |b| {
        b.iter_with_setup(
            || StableCell::new(1),
            |val| {
                let cur = *black_box(&val).try_borrow().unwrap();
                *black_box(&val).try_borrow_mut().unwrap() = (cur + 1) % 10;
            }
        )
    });
}

pub fn stable_borrow(c: &mut Criterion) {
    c.bench_function("StableCell::try_borrow", |b| {
        b.iter_with_setup(
            || StableCell::new(-1),
            |cell| black_box(cell.try_borrow()),
        )
    });
    c.bench_function("StableCell::try_borrow_mut", |b| {
        b.iter_with_setup(
            || StableCell::new(-1),
            |cell| black_box(cell.try_borrow_mut()),
        )
    });
}

criterion_group!(object_tree, add_remove_node);

criterion_group!(
    name = stable_cell;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = stable_borrow, stable_vs_rc
);

criterion_main!(object_tree, stable_cell);
