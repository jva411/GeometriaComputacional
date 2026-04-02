use tarefa01::{math::vec3::Vec3, sort::{merge_sort::merge_sort, quick_sort::quick_sort, selection_sort::selection_sort}};

pub fn check_sort<T: Ord>(array: &[T]) -> bool {
  for i in 1..array.len() {
    if array[i] < array[i - 1] {
      return false;
    }
  }

  return true;
}

fn main() {
  let n = 1000;
  let mut array = Vec::with_capacity(n);
  for _ in 0..n {
    array.push(rand::random_range(1..=n))
  }

  let mut temp_array = array.clone();
  quick_sort(&mut temp_array);
  let quick_sort_result = check_sort(&temp_array);

  let mut temp_array = array.clone();
  selection_sort(&mut temp_array);
  let selection_sort_result = check_sort(&temp_array);

  let mut temp_array = array.clone();
  merge_sort(&mut temp_array);
  let merge_sort_result = check_sort(&temp_array);

  println!("Quick Sort: {}", quick_sort_result);
  println!("Selection Sort: {}", selection_sort_result);
  println!("Merge Sort: {}", merge_sort_result);

  let a = Vec3::X;
  let b = Vec3::Y;
  let c = a.cross(b);

  println!("A: {:?}", a);
  println!("B: {:?}", b);
  println!("C: {:?}", c);

  let temp = c + a + b;
  println!("Sum vectors: {:?}", temp);

  let temp = c - a - b;
  println!("Subtract vectors: {:?}", temp);

  let temp = c * a * b;
  println!("Multiply vectors: {:?}", temp);

  let temp = (c / a) / b;
  println!("Divide vectors: {:?}", temp);

  let temp = c.dot(a);
  println!("C dot A: {:?}", temp);

  let temp = c.length_squared();
  println!("C length squared: {:?}", temp);

  let temp = c.length();
  println!("C length: {:?}", temp);

  let temp = (c * 10.0).normalize();
  println!("C normalized: {:?}", temp);

  let temp = c * 10.0;
  println!("C * 10: {:?}", temp);

  let temp = c / 10.0;
  println!("C / 10: {:?}", temp);

  let temp = c + 10.0;
  println!("C + 10: {:?}", temp);

  let temp = c - 10.0;
  println!("C - 10: {:?}", temp);

  let temp = c % 10.0;
  println!("C % 10: {:?}", temp);

  let temp = c % 0.5;
  println!("C % 0.5: {:?}", temp);

  let temp = -c;
  println!("Negative C: {:?}", temp);
}
