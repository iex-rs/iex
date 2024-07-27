use criterion::{black_box, criterion_group, criterion_main, Criterion};
use iex::{iex, Outcome};

#[iex]
fn unwind(n: i32) -> Result<(), &'static str> {
    if n > 0 {
        black_box(Ok(unwind(n - 1)?))
    } else {
        Err("Overflow")
    }
}

fn start_unwind(n: i32) {
    let _ = unwind(n).into_result();
}

fn result(n: i32) -> Result<(), &'static str> {
    if n > 0 {
        black_box(Ok(result(n - 1)?))
    } else {
        Err("Overflow")
    }
}

fn start_result(n: i32) {
    let _ = result(n);
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("depth 100");
    group.bench_function("unwind", |b| b.iter(|| start_unwind(black_box(100))));
    group.bench_function("result", |b| b.iter(|| start_result(black_box(100))));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
