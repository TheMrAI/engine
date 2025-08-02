use super::Matrix;

impl<ValueType, const COLS: usize, const ROWS: usize> Default for Matrix<ValueType, COLS, ROWS>
where
    ValueType: Default + Copy,
{
    fn default() -> Self {
        Self {
            data: [[ValueType::default(); COLS]; ROWS],
        }
    }
}
