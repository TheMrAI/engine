/// Generate a [Vector](crate::vector::Vector) similar to the [vec!] macro.
///
/// It has three forms.
///  - A convenience/reference default initialization
/// ```
/// use lina::vector::{Vector, v};
///
/// let preferred: Vector::<usize, 3> = v![];
/// let over = Vector::<usize, 3>::new();
/// ```
///
///  - Create a [Vector](crate::vector::Vector) with default value D and N elements
/// ```
/// use lina::vector::v;
///
/// let V = v![3; 4];
/// assert_eq!(V, [3, 3, 3, 3]);
/// ```
///
/// - Create a [Vector](crate::vector::Vector) from a list of values
/// ```
/// use lina::vector::v;
///
/// let V = v![1,2,3,4,5];
/// assert_eq!(V, [1,2,3,4,5]);
/// ```
#[macro_export]
macro_rules! v {
    () => {
        $crate::vector::Vector::new()
    };
    ($default_value:expr; $n:expr) => {
        $crate::vector::Vector::<_, $n>::from_value($default_value)
    };
    ($($element:expr),+$(,)?) => {
        $crate::vector::Vector::from_array([$($element),+])
    }
}

// Re-export the macro, avoiding the need for #[macro_use].
pub use v;
