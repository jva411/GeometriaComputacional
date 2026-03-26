pub fn merge_sort<T: PartialOrd + Clone>(array: &mut [T]) {
  let n = array.len();
  if n <= 1 {
    return;
  }

  let mid = n / 2;
  let (left, right) = array.split_at_mut(mid);
  merge_sort(left);
  merge_sort(right);
  merge(left, right);
}


pub fn merge<T: PartialOrd + Clone>(left: &mut [T], right: &mut [T]) {
  if left.is_empty() || right.is_empty() {
    return;
  }

  let mut merged_array = Vec::with_capacity(left.len() + right.len());
  let mut i = 0;
  let mut j = 0;
  while i < left.len() && j < right.len() {
    if left[i] < right[j] {
      merged_array.push(left[i].clone());
      i += 1;
    } else {
      merged_array.push(right[j].clone());
      j += 1;
    }
  }

  while i < left.len() {
    merged_array.push(left[i].clone());
    i += 1;
  }

  while j < right.len() {
    merged_array.push(right[j].clone());
    j += 1;
  }

  let (merged_left, merged_right) = merged_array.split_at_mut(left.len());
  left.swap_with_slice(merged_left);
  right.swap_with_slice(merged_right);
}
