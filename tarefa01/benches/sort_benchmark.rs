use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion, BatchSize};
use tarefa01::sort::{merge_sort::merge_sort, quick_sort::quick_sort, selection_sort::selection_sort};

fn bench_sorts(c: &mut Criterion) {
  let n = 10000;

  let mut base_array: Vec<i32> = Vec::with_capacity(n);
  for _ in 0..n {
    base_array.push(rand::random_range(1..=n as i32));
  }

  let mut group = c.benchmark_group("Algoritmos de Ordenação");

  group.bench_function("Quick Sort", |b| {
    b.iter_batched(
      || base_array.clone(),
      | mut array | quick_sort(black_box(&mut array)),
      BatchSize::SmallInput,
    )
  });

  group.bench_function("Merge Sort", |b| {
    b.iter_batched(
      || base_array.clone(),
      | mut array | merge_sort(black_box(&mut array)),
      BatchSize::SmallInput,
    )
  });

  group.bench_function("Selection Sort", |b| {
    b.iter_batched(
      || base_array.clone(),
      | mut array | selection_sort(black_box(&mut array)),
      BatchSize::SmallInput,
    )
  });

  group.bench_function("Rust Standard Sort", |b| {
    b.iter_batched(
      || base_array.clone(),
      | mut array | black_box(&mut array).sort(),
      BatchSize::SmallInput,
    )
  });

  group.finish();
}

criterion_group!(benches, bench_sorts);
criterion_main!(benches);
