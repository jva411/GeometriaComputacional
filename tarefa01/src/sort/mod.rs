pub mod merge_sort;
pub mod quick_sort;
pub mod selection_sort;


#[cfg(test)]
mod tests {
  use super::*;

  const ARRAY_SIZE: usize = 10_000;

  fn is_sorted<T: PartialOrd>(array: &[T]) -> bool {
    for i in 1..array.len() {
      if array[i - 1] > array[i] {
        return false;
      }
    }

    return true;
  }

  fn generate_random_array(size: usize) -> Vec<usize> {
    let mut random_array: Vec<usize> = Vec::with_capacity(size);
    for _ in 0..size {
      random_array.push(rand::random_range(1..=size));
    }
    return random_array;
  }

  #[test]
  fn test_merge_sort() {
    let mut array = generate_random_array(ARRAY_SIZE);
    merge_sort::merge_sort(&mut array);
    assert!(is_sorted(&array));
  }

  #[test]
  fn test_quick_sort() {
    let mut array = generate_random_array(ARRAY_SIZE);
    quick_sort::quick_sort(&mut array);
    assert!(is_sorted(&array));
  }

  #[test]
  fn test_selection_sort() {
    let mut array = generate_random_array(ARRAY_SIZE);
    selection_sort::selection_sort(&mut array);
    assert!(is_sorted(&array));
  }
}
