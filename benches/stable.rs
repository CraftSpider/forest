use std::cell::RefCell;
use std::rc::Rc;
use criterion::{BenchmarkId, black_box, Criterion, criterion_group};

use craft_forest::stable::cell::StableCell;
use crate::criterion;

pub fn cell_vs_rc(c: &mut Criterion) {
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

pub fn cell_borrow(c: &mut Criterion) {
    let cell = StableCell::new(-1);
    let mut group = c.benchmark_group("StableCell::try_borrow");

    group.bench_function(
        BenchmarkId::from_parameter("no_borrows"),
        |b| b.iter(|| black_box(&cell).try_borrow()),
    );
    let _b = cell.try_borrow().unwrap();
    group.bench_function(
        BenchmarkId::from_parameter("existing_borrows"),
        |b| b.iter(|| black_box(&cell).try_borrow()),
    );
    drop(_b);
    let _m = cell.try_borrow_mut().unwrap();
    group.bench_function(
        BenchmarkId::from_parameter("existing_mut"),
        |b| b.iter(|| black_box(&cell).try_borrow()),
    );
    drop(_m);
}

pub fn cell_borrow_mut(c: &mut Criterion) {
    let cell = StableCell::new(-1);
    let mut group = c.benchmark_group("StableCell::try_borrow_mut");

    group.bench_function(
        BenchmarkId::from_parameter("no_borrows"),
        |b| b.iter(|| black_box(&cell).try_borrow_mut()),
    );
    let _b = cell.try_borrow().unwrap();
    group.bench_function(
        BenchmarkId::from_parameter("existing_borrows"),
        |b| b.iter(|| black_box(&cell).try_borrow_mut()),
    );
    drop(_b);
    let _m = cell.try_borrow_mut().unwrap();
    group.bench_function(
        BenchmarkId::from_parameter("existing_mut"),
        |b| b.iter(|| black_box(&cell).try_borrow_mut()),
    );
    drop(_m);
}

criterion_group!(
    name = cell;
    config = criterion();
    targets = cell_borrow, cell_borrow_mut, cell_vs_rc
);
