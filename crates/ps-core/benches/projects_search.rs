use std::hint::black_box;
use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use ps_core::projects::Project;
use ps_core::search::ProjectSearch;

fn benchmark_project_search(criterion: &mut Criterion) {
    let mut search = ProjectSearch::new();
    for index in 0..10_000 {
        search.upsert(Project {
            id: format!("{index:026}"),
            name: format!("Project {index}"),
            path: PathBuf::from(format!("/Users/reader/Documents/project-{index}")),
            added_at: "2026-08-13T12:00:00Z".to_owned(),
            last_opened_at: None,
            pinned: false,
            accent: None,
            last_file: None,
            available: None,
        });
    }

    criterion.bench_function("search 10,000 projects", |bencher| {
        bencher.iter(|| search.search(black_box("prj 9999"), black_box(50)));
    });
}

criterion_group!(benches, benchmark_project_search);
criterion_main!(benches);
