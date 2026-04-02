pub fn quick_sort<T: PartialOrd + Clone>(array: &mut [T]) {
  let n = array.len();
  if n <= 1 {
    return;
  }

  let pivot_index = rand::random_range(0..n);
  let pivot = array[pivot_index].clone();

  let (part_i, part_j) = partition(array, pivot);
  let (left, right) = array.split_at_mut(part_i);
  let (_, right) = right.split_at_mut(part_j - part_i);

  quick_sort(left);
  quick_sort(&mut right[1..]);
}

fn partition<T: PartialOrd + Clone>(array: &mut [T], pivot: T) -> (usize, usize) {
  let mut lt = 0;
  let mut i = 0;
  let mut gt = array.len() - 1;

  while i <= gt {
    if array[i] < pivot {
      array.swap(lt, i);
      lt += 1;
      i += 1;
    } else if array[i] > pivot {
      array.swap(i, gt);

      if gt == 0 {
        break;
      }
      gt -= 1;
    } else {
      i += 1;
    }
  }

  return (lt, gt);
}
