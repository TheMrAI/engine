use super::Matrix;

pub trait Resize<
    ValueType,
    const COLS: usize,
    const ROWS: usize,
    const TARGET_COLS: usize,
    const TARGET_ROWS: usize,
> where
    ValueType: Default + Copy,
{
    /// Generate a `resized` matrix.
    ///
    /// The resulting matrix has the same underlying type and any
    /// new elements, will be initialized to their [Default] values.
    ///
    /// ## Example
    ///
    /// ```
    /// # use lina::matrix::{m, Matrix, Resize};
    /// let original = m![[1, 2, 3], [4, 5, 6], [7, 8, 9]];
    /// let resized: Matrix<i8, 3, 3> = original.resize();
    /// ```
    ///
    /// For terser invocation consider using [resize!](super::macros::resize).
    fn resize(&self) -> Matrix<ValueType, TARGET_COLS, TARGET_ROWS>;
}

impl<
    ValueType,
    const COLS: usize,
    const ROWS: usize,
    const TARGET_COLS: usize,
    const TARGET_ROWS: usize,
> Resize<ValueType, COLS, ROWS, TARGET_COLS, TARGET_ROWS> for Matrix<ValueType, COLS, ROWS>
where
    ValueType: Default + Copy,
{
    fn resize(&self) -> Matrix<ValueType, TARGET_COLS, TARGET_ROWS> {
        let mut data = [[ValueType::default(); TARGET_COLS]; TARGET_ROWS];

        let columns = std::cmp::min(COLS, TARGET_COLS);
        unsafe {
            for (i, row) in self
                .data
                .iter()
                .enumerate()
                .take(std::cmp::min(ROWS, TARGET_ROWS))
            {
                std::ptr::copy(row.as_ptr(), data[i].as_mut_ptr(), columns);
            }
        }

        Matrix { data }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        m,
        matrix::{Matrix, Resize},
        resize,
    };

    #[test]
    fn same_size() {
        let original = m![[1, 2, 3], [4, 5, 6], [7, 8, 9]];
        let resized: Matrix<i8, 3, 3> = original.resize();

        assert_eq!(resized, m![[1, 2, 3], [4, 5, 6], [7, 8, 9]]);
    }

    #[test]
    fn slice_both() {
        let original = m![[1, 2, 3], [4, 5, 6], [7, 8, 9]];
        let resized: Matrix<i8, 2, 2> = original.resize();

        assert_eq!(resized, m![[1, 2], [4, 5]]);
    }

    #[test]
    fn expand_column_slice_rows() {
        let original = m![[1, 2, 3], [4, 5, 6], [7, 8, 9]];
        let resized: Matrix<i8, 5, 2> = original.resize();

        assert_eq!(resized, m![[1, 2, 3, 0, 0], [4, 5, 6, 0, 0]]);
    }

    #[test]
    fn expand_rows_slice_columns() {
        let original = m![[1, 2, 3], [4, 5, 6], [7, 8, 9]];
        let resized: Matrix<i8, 2, 5> = original.resize();

        assert_eq!(resized, m![[1, 2], [4, 5], [7, 8], [0, 0], [0, 0]]);
    }

    #[test]
    fn expand_both() {
        let original = m![[1, 2, 3], [4, 5, 6], [7, 8, 9]];
        let resized: Matrix<i8, 5, 5> = original.resize();

        assert_eq!(
            resized,
            m![
                [1, 2, 3, 0, 0],
                [4, 5, 6, 0, 0],
                [7, 8, 9, 0, 0],
                [0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0]
            ]
        );
    }

    #[test]
    fn resize_same_size() {
        let original = m![[1, 2, 3], [4, 5, 6], [7, 8, 9]];
        let resized = resize!(original, 3, 3);

        assert_eq!(resized, m![[1, 2, 3], [4, 5, 6], [7, 8, 9]]);
    }

    #[test]
    fn resize_slice_both() {
        let original = m![[1, 2, 3], [4, 5, 6], [7, 8, 9]];
        let resized = resize!(original, 2, 2);

        assert_eq!(resized, m![[1, 2], [4, 5]]);
    }

    #[test]
    fn resize_expand_column_slice_rows() {
        let original = m![[1, 2, 3], [4, 5, 6], [7, 8, 9]];
        let resized = resize!(original, 5, 2);

        assert_eq!(resized, m![[1, 2, 3, 0, 0], [4, 5, 6, 0, 0]]);
    }

    #[test]
    fn resize_expand_rows_slice_columns() {
        let original = m![[1, 2, 3], [4, 5, 6], [7, 8, 9]];
        let resized = resize!(original, 2, 5);

        assert_eq!(resized, m![[1, 2], [4, 5], [7, 8], [0, 0], [0, 0]]);
    }

    #[test]
    fn resize_expand_both() {
        let original = m![[1, 2, 3], [4, 5, 6], [7, 8, 9]];
        let resized = resize!(original, 5, 5);

        assert_eq!(
            resized,
            m![
                [1, 2, 3, 0, 0],
                [4, 5, 6, 0, 0],
                [7, 8, 9, 0, 0],
                [0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0]
            ]
        );
    }
}
