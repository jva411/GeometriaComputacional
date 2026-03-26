pub fn selection_sort<T: PartialOrd>(array: &mut [T]) {
  let n = array.len();
  if n <= 1 {
    return;
  }

  for i in 0..n - 1 {
    let mut min_index = i;
    for j in i + 1..n {
      if array[j] < array[min_index] {
        min_index = j;
      }
    }

    array.swap(i, min_index);
  }
}
