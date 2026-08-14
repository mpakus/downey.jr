use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use ps_render::render;

const KIB: usize = 1024;
const MIB: usize = 1024 * KIB;
const CHUNK_SEPARATOR: &str = "</section>\n<section class=\"chunk\">";
const MARKDOWN_SEED: &str = concat!(
    "# Section\n\n",
    "Paragraph with **strong text**, `code`, and [a link](notes/next.md). ",
    "This representative prose keeps the benchmark close to a long-form document.\n\n",
    "A second paragraph includes *emphasis*, ~~deleted text~~, and inline math $x + y$. ",
    "It also gives each section enough body text to avoid a heading-heavy synthetic case.\n\n",
    "The renderer still maps every top-level block to source bytes and hashes its input. ",
    "Repeated sections exercise duplicate heading identifiers without dominating the file.\n\n",
    "- first task\n",
    "- second task\n",
    "- third task\n\n",
    "> A short quotation with a hard break.  \n",
    "> The second line stays inside the same top-level block.\n\n",
    "```rust\n",
    "fn example() -> usize { 1537 }\n",
    "```\n\n",
);

fn markdown_of_size(bytes: usize) -> String {
    let mut markdown = MARKDOWN_SEED.repeat(bytes.div_ceil(MARKDOWN_SEED.len()));
    markdown.truncate(bytes);
    markdown
}

fn benchmark_render_sizes(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("render markdown");

    for (label, bytes) in [
        ("10 KiB", 10 * KIB),
        ("100 KiB", 100 * KIB),
        ("1 MiB", MIB),
        ("5 MiB", 5 * MIB),
    ] {
        let markdown = markdown_of_size(bytes);
        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_with_input(
            BenchmarkId::new("document", label),
            &markdown,
            |bencher, input| {
                bencher.iter(|| render(black_box(input)));
            },
        );
    }

    group.finish();
}

fn benchmark_first_chunk(criterion: &mut Criterion) {
    let markdown = markdown_of_size(5 * MIB);

    criterion.bench_function("first chunk from 5 MiB document", |bencher| {
        bencher.iter(|| {
            let mut html = render(black_box(&markdown));
            if let Some(separator) = html.find(CHUNK_SEPARATOR) {
                html.truncate(separator + "</section>".len());
            }
            black_box(html)
        });
    });
}

criterion_group!(benches, benchmark_render_sizes, benchmark_first_chunk);
criterion_main!(benches);
