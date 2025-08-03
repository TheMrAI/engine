#[macro_export]
macro_rules! m {
    () => {
        $crate::matrix::Matrix::new()
    };
    ($default_value:expr; $cols:expr, $rows:expr) => {
        $crate::matrix::Matrix::<_, $cols, $rows>::from_value($default_value)
    };
    ($($element:expr),+$(,)?) => {
        $crate::matrix::Matrix::from_matrix([$($element),+])
    }
}

// Re-export the macro, avoiding the need for #[macro_use].
pub use m;
