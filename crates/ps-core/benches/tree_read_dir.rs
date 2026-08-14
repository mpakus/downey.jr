use std::fs::File;
use std::hint::black_box;
use std::path::Path;
use std::time::{Duration, Instant};

use criterion::{Criterion, criterion_group, criterion_main};
use ps_core::tree;

const ENTRY_COUNT: usize = 50_000;
const BUDGET: Duration = Duration::from_millis(120);

fn benchmark_tree_read_dir(criterion: &mut Criterion) {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    std::fs::create_dir(&root).expect("project directory");
    for index in 0..ENTRY_COUNT {
        File::create(root.join(format!("file{index}.md"))).expect("benchmark file");
    }

    let started = Instant::now();
    let nodes = tree::read_dir(&root, Path::new(""), false).expect("tree nodes");
    let elapsed = started.elapsed();
    assert_eq!(nodes.len(), ENTRY_COUNT);
    assert!(
        elapsed < BUDGET,
        "reading {ENTRY_COUNT} entries took {elapsed:?}, budget is {BUDGET:?}"
    );

    criterion.bench_function("read one tree level with 50,000 files", |bencher| {
        bencher.iter(|| tree::read_dir(black_box(&root), black_box(Path::new("")), false));
    });
}

criterion_group!(benches, benchmark_tree_read_dir);
criterion_main!(benches);
