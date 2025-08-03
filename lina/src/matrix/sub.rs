use std::mem;

use super::Matrix;

impl<ValueType, const COLS: usize, const ROWS: usize> std::ops::Sub<Matrix<ValueType, COLS, ROWS>>
    for Matrix<ValueType, COLS, ROWS>
where
    ValueType: std::ops::Sub<Output = ValueType> + Copy,
{
    type Output = Matrix<ValueType, COLS, ROWS>;

    /// Implement `Matrix<T> - Matrix<T>` operation.
    fn sub(self, rhs: Matrix<ValueType, COLS, ROWS>) -> Self::Output {
        let mut data = [[mem::MaybeUninit::<ValueType>::uninit(); COLS]; ROWS];

        // Given that the two matrices have the same shape, we can simply flatten the internal structures
        // and apply the operation per element.
        for (elem, (lhs, rhs)) in data
            .iter_mut()
            .flatten()
            .zip(self.data.iter().flatten().zip(rhs.data.iter().flatten()))
        {
            elem.write(*lhs - *rhs);
        }

        let ptr = &mut data as *mut _ as *mut [[ValueType; COLS]; ROWS];
        let transmuted = unsafe { ptr.read() };

        Matrix { data: transmuted }
    }
}

#[cfg(test)]
mod tests {
    use crate::m;

    #[test]
    fn add_by_value() {
        let lhs = m![[1, 2], [3, 4]];
        let rhs = m![[5, 6], [7, 8]];

        let result = lhs - rhs;
        assert_eq!(result.as_slices(), &[[-4, -4], [-4, -4]]);
    }
}
