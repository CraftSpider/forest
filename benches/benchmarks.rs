use criterion::{criterion_main, Criterion};
use pprof::criterion::{PProfProfiler, Output};

fn criterion() -> Criterion {
    Criterion::default()
        .with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)))
        .configure_from_args()
}

mod object_tree;
mod stable;

criterion_main!(object_tree::object_tree, stable::cell);
