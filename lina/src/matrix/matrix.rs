#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Matrix<ValueType, const COLS: usize, const ROWS: usize> {
    pub(crate) data: [[ValueType; COLS]; ROWS],
}

impl<ValueType, const COLS: usize, const ROWS: usize> Matrix<ValueType, COLS, ROWS>
where
    ValueType: Default + Copy,
{
    /// Create a new [Matrix] filled with [Default](std::default::Default) of `ValueType`.
    ///
    /// Example
    /// ```
    /// # use lina::matrix::Matrix;
    /// let v1 : Matrix<i32, 3, 3> = Matrix::new();
    /// // or
    /// let v2 = Matrix::<i32, 3, 3>::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }
}

impl<ValueType, const COLS: usize, const ROWS: usize> Matrix<ValueType, COLS, ROWS>
where
    ValueType: Copy,
{
    /// Create a new [Matrix] filled with `default_value`.
    ///
    /// Example
    /// ```
    /// # use lina::matrix::Matrix;
    /// let v1 : Matrix<i32, 3, 3> = Matrix::from_value(3);
    /// // or
    /// let v2 = Matrix::<i32, 3, 3>::from_value(3);
    /// ```
    pub fn from_value(default_value: ValueType) -> Self {
        Self {
            data: [[default_value; COLS]; ROWS],
        }
    }

    pub fn transpose(&self) -> Matrix<ValueType, ROWS, COLS> {
        let mut data = [[std::mem::MaybeUninit::<ValueType>::uninit(); ROWS]; COLS];

        // We want the needless range loops, as we use the value to index multiple times.
        #[allow(clippy::needless_range_loop)]
        for i in 0..ROWS {
            #[allow(clippy::needless_range_loop)]
            for j in 0..COLS {
                data[j][i].write(self.data[i][j]);
            }
        }

        let ptr = &mut data as *mut _ as *mut [[ValueType; ROWS]; COLS];
        let transmuted = unsafe { ptr.read() };

        Matrix { data: transmuted }
    }
}

impl<ValueType, const COLS: usize, const ROWS: usize> Matrix<ValueType, COLS, ROWS> {
    /// Create a slice into the internal data
    pub fn as_slices(&self) -> &[[ValueType; COLS]; ROWS] {
        &self.data
    }

    pub fn from_matrix(values: [[ValueType; COLS]; ROWS]) -> Self {
        Self { data: values }
    }
}

#[cfg(test)]
mod tests {
    use crate::m;
    use crate::matrix::Matrix;

    #[test]
    fn macro_init_empty_matrix() {
        let matrix: Matrix<usize, 2, 2> = m![];
        assert_eq!(matrix.as_slices(), &[[0, 0], [0, 0]]);
    }

    #[test]
    fn macro_init_with_default_value_and_count() {
        let matrix = m![1; 2, 2];
        assert_eq!(matrix.as_slices(), &[[1, 1], [1, 1]]);
    }

    #[test]
    fn macro_init_with_set_values() {
        let matrix = m![[1, 2], [3, 4]];
        assert_eq!(matrix.as_slices(), &[[1, 2], [3, 4]]);
    }
}
