use std::hint::black_box;

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use rand::seq::SliceRandom;
use tarefa01::sort::{merge_sort::merge_sort, quick_sort::quick_sort, selection_sort::selection_sort};

fn bench_sorts(c: &mut Criterion) {
  let mut group = c.benchmark_group("Algoritmos de Ordenação");
  let sizes = [8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192];

  for size in sizes {
    // Random
    let mut random_array: Vec<usize> = Vec::with_capacity(size);
    for _ in 0..size {
      random_array.push(rand::random_range(1..=size));
    }

    // Random Without Duplicates
    let mut random_without_duplicates_array: Vec<usize> = (1..=size).collect();
    random_without_duplicates_array.shuffle(&mut rand::rng());

    // Random With 30% Duplicates
    let mut random_with_duplicates_array: Vec<usize> = (1..=(size as f64 * 0.7).max(1.0) as usize).collect();
    random_with_duplicates_array.shuffle(&mut rand::rng());

    // Sorted
    let sorted_array: Vec<usize> = (1..=size).collect();

    // Reversed
    let mut reversed_array: Vec<usize> = (1..=size).collect();
    reversed_array.reverse();

    let cenarios = [
      ("Random", random_array),
      ("Random Without Duplicates", random_without_duplicates_array),
      ("Random With 30% Duplicates", random_with_duplicates_array),
      ("Sorted", sorted_array),
      ("Reversed", reversed_array),
    ];

    for (cenario, base_array) in cenarios {
      group.bench_with_input(BenchmarkId::new(format!("Quick Sort - {cenario}"), size), &size, |b, _| {
        b.iter_batched(
          || base_array.clone(),
          | mut array | quick_sort(black_box(&mut array)),
          BatchSize::SmallInput,
        )
      });

      group.bench_with_input(BenchmarkId::new(format!("Merge Sort - {cenario}"), size), &size, |b, _| {
        b.iter_batched(
          || base_array.clone(),
          | mut array | merge_sort(black_box(&mut array)),
          BatchSize::SmallInput,
        )
      });

      group.bench_with_input(BenchmarkId::new(format!("Selection Sort - {cenario}"), size), &size, |b, _| {
        b.iter_batched(
          || base_array.clone(),
          | mut array | selection_sort(black_box(&mut array)),
          BatchSize::SmallInput,
        )
      });
    }
  }

  group.finish();
}

criterion_group!(benches, bench_sorts);
criterion_main!(benches);
