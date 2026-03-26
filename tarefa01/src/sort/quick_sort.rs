pub fn quick_sort<T: PartialOrd>(array: &mut [T]) {
  let n = array.len();
  if n <= 1 {
    return;
  }

  let pivot_index = rand::random_range(0..n);
  array.swap(pivot_index, n - 1);

  let pivot_index = partition(array);
  let (left, right) = array.split_at_mut(pivot_index);

  quick_sort(left);
  quick_sort(&mut right[1..]);
}

fn partition<T: PartialOrd>(array: &mut [T]) -> usize {
  let len = array.len();
  let pivot_index = len - 1;

  let mut i = 0;

  for j in 0..pivot_index {
    if array[j] <= array[pivot_index] {
      array.swap(i, j);
      i += 1;
    }
  }

  array.swap(i, pivot_index);

  i
}
