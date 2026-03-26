// Common functionalities (creation, math operations, etc.)
#[macro_export]
macro_rules! vec_implement_common_functions {
  ($size:literal, $( $index:tt => $fields:ident: $ftype:ty),*) => {
    #[inline] pub const fn new($($fields: f64),*) -> Self { Self { $($fields),* } }
    #[inline] pub const fn splat(v: f64) -> Self { Self { $($fields: v),* } }

    #[inline] pub const fn to_array(self) -> [f64; $size] { [$(self.$fields),*] }
    #[inline] pub const fn to_tuple(self) -> ($($ftype),*) { ($(self.$fields),*) }
    #[inline] pub fn to_vec(self) -> Vec<f64> { vec![$(self.$fields),*] }
  };
}

#[macro_export]
macro_rules! vec_declare_constants {
  () => {
    pub const ZERO: Self = Self::splat(0.0);
    pub const ONE: Self = Self::splat(1.0);
    pub const X: Self = Self::new(1.0, 0.0, 0.0);
    pub const Y: Self = Self::new(0.0, 1.0, 0.0);
    pub const Z: Self = Self::new(0.0, 0.0, 1.0);

    pub const NEG_ZERO: Self = Self::splat(-0.0);
    pub const NEG_ONE: Self = Self::splat(-1.0);
    pub const NEG_X: Self = Self::new(-1.0, 0.0, 0.0);
    pub const NEG_Y: Self = Self::new(0.0, -1.0, 0.0);
    pub const NEG_Z: Self = Self::new(0.0, 0.0, -1.0);

    pub const MIN: Self = Self::splat(f64::MIN);
    pub const MAX: Self = Self::splat(f64::MAX);
    pub const NAN: Self = Self::splat(f64::NAN);
    pub const INFINITY: Self = Self::splat(f64::INFINITY);
    pub const NEG_INFINITY: Self = Self::splat(f64::NEG_INFINITY);
    pub const EPSILON: Self = Self::splat(f64::EPSILON);
  };
}

#[macro_export]
macro_rules! vec_implement_common_traits {
  ($type:ty, $size:literal, $( $index:tt => $fields:ident: $ftype:ty),*) => {
    // Default trait, used to define a default value for this struct
    impl Default for $type {
      #[inline]
      fn default() -> Self { Self::ZERO }
    }

    // Index trait, used to access elements by index
    impl std::ops::Index<usize> for $type {
      type Output = f64;
      #[inline]
      fn index(&self, index: usize) -> &Self::Output {
        match index {
          $(
            $index => &self.$fields,
          )*
          _ => panic!("Index out of range"),
        }
      }
    }

    impl std::ops::IndexMut<usize> for $type {
      #[inline]
      fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
          $(
            $index => &mut self.$fields,
          )*
          _ => panic!("Index out of range"),
        }
      }
    }

    // From trait, used to convert Vectors between equivalent types
    impl From<[f64; $size]> for $type {
      #[inline]
      fn from(arr: [f64; $size]) -> Self { Self::new(arr[0], arr[1], arr[2]) }
    }

    impl From<$type> for [f64; $size] {
      #[inline]
      fn from(vec: $type) -> Self { [vec.x, vec.y, vec.z] }
    }

    impl From<Vec<f64>> for $type {
      #[inline]
      fn from(vec: Vec<f64>) -> Self { Self::new(vec[0], vec[1], vec[2]) }
    }

    impl From<$type> for Vec<f64> {
      #[inline]
      fn from(vec: $type) -> Self { vec.into() }
    }

    impl From<($($ftype),*)> for $type {
      #[inline]
      fn from(tuple: ($($ftype),*)) -> Self { Self::new($(tuple.$index),*) }
    }

    impl From<$type> for ($($ftype),*) {
      #[inline]
      fn from(vec: $type) -> Self { (vec.x, vec.y, vec.z) }
    }

    // Sum trait, used to accumulate elements in an iterator
    impl std::iter::Sum for $type {
      #[inline]
      fn sum<I>(iter: I) -> Self where I: Iterator<Item = Self> {
        iter.fold(Self::ZERO, std::ops::Add::add)
      }
    }

    impl<'a> std::iter::Sum<&'a Self> for $type {
      #[inline]
      fn sum<I>(iter: I) -> Self where I: Iterator<Item = &'a Self> {
        iter.fold(Self::ZERO, std::ops::Add::add)
      }
    }

    impl std::iter::Sum<f64> for $type {
      #[inline]
      fn sum<I>(iter: I) -> Self where I: Iterator<Item = f64> {
        iter.fold(Self::ZERO, std::ops::Add::add)
      }
    }
  }
}

// Operators Overloading

#[macro_export]
macro_rules! vec_implement_binary_operator {
  ($type:ty, $vec_func:ident, $vec_func2:ident, $op:ident, $op_func:ident, $( $fields:ident),*) => {
    impl std::ops::$op for $type {
      type Output = Self;
      #[inline]
      fn $op_func(self, other: Self) -> Self {
        Self {
          $($fields: self.$fields.$op_func(other.$fields)),*
        }
      }
    }

    impl<'a> std::ops::$op<&'a Self> for $type {
      type Output = Self;
      #[inline]
      fn $op_func(self, other: &'a Self) -> Self {
        Self {
          $($fields: self.$fields.$op_func(other.$fields)),*
        }
      }
    }

    impl std::ops::$op<f64> for $type {
      type Output = Self;
      #[inline]
      fn $op_func(self, other: f64) -> Self {
        Self {
          $($fields: self.$fields.$op_func(other)),*
        }
      }
    }

    impl<'a> std::ops::$op<&'a f64> for $type {
      type Output = Self;
      #[inline]
      fn $op_func(self, other: &'a f64) -> Self {
        Self {
          $($fields: self.$fields.$op_func(*other)),*
        }
      }
    }
  };
}
#[macro_export]
macro_rules! vec_implement_assign_operator {
  ($type:ty, $vec_func:ident, $vec_func2:ident, $op:ident, $op_func:ident, $($fields:ident),*) => {
    impl std::ops::$op for $type {
      #[inline]
      fn $op_func(&mut self, other: Self) {
        $(
          self.$fields.$op_func(other.$fields);
        )*
      }
    }

    impl<'a> std::ops::$op<&'a Self> for $type {
      #[inline]
      fn $op_func(&mut self, other: &'a Self) {
        $(
          self.$fields.$op_func(other.$fields);
        )*
      }
    }

    impl std::ops::$op<f64> for $type {
      #[inline]
      fn $op_func(&mut self, other: f64) {
        $(
          self.$fields.$op_func(other);
        )*
      }
    }

    impl<'a> std::ops::$op<&'a f64> for $type {
      #[inline]
      fn $op_func(&mut self, other: &'a f64) {
        $(
          self.$fields.$op_func(other);
        )*
      }
    }
  };
}

#[macro_export]
macro_rules! vec_implement_unary_operator {
  ($type:ty, $op:ident, $op_func:ident, $($fields:ident), *) => {
    impl std::ops::$op for $type {
      type Output = Self;
      #[inline]
      fn $op_func(self) -> Self { Self { $($fields: self.$fields.$op_func()),* } }
    }
  };
}

#[macro_export]
macro_rules! vec_implement_operators_overloading {
  ($type:ty, $( $fields:ident),*) => {
    // +
    $crate::vec_implement_binary_operator!($type, add, add_f64, Add, add, $( $fields),*);
    // +=
    $crate::vec_implement_assign_operator!($type, add, add_f64, AddAssign, add_assign, $( $fields),*);

    // -
    $crate::vec_implement_binary_operator!($type, sub, sub_f64, Sub, sub, $( $fields),*);
    // -=
    $crate::vec_implement_assign_operator!($type, sub, sub_f64, SubAssign, sub_assign, $( $fields),*);

    // *
    $crate::vec_implement_binary_operator!($type, mul, mul_f64, Mul, mul, $( $fields),*);
    // *=
    $crate::vec_implement_assign_operator!($type, mul, mul_f64, MulAssign, mul_assign, $( $fields),*);

    // /
    $crate::vec_implement_binary_operator!($type, div, div_f64, Div, div, $( $fields),*);
    // /=
    $crate::vec_implement_assign_operator!($type, div, div_f64, DivAssign, div_assign, $( $fields),*);

    // %
    $crate::vec_implement_binary_operator!($type, rem, rem_f64, Rem, rem, $( $fields),*);
    // %=
    $crate::vec_implement_assign_operator!($type, rem, rem_f64, RemAssign, rem_assign, $( $fields),*);

    // - (unary)
    $crate::vec_implement_unary_operator!($type, Neg, neg, $( $fields),*);
  };
}
