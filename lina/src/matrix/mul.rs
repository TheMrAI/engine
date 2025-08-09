use std::mem;

use super::Matrix;

impl<ValueType, const COLS: usize, const ROWS: usize> std::ops::Mul<Matrix<ValueType, ROWS, COLS>>
    for Matrix<ValueType, COLS, ROWS>
where
    ValueType:
        std::ops::Mul<ValueType> + std::iter::Sum<<ValueType as std::ops::Mul>::Output> + Copy,
{
    type Output = Matrix<ValueType, ROWS, ROWS>;

    /// Implement `Matrix<T> * Matrix<T>` operation.
    fn mul(self, rhs: Matrix<ValueType, ROWS, COLS>) -> Self::Output {
        let mut data = [[mem::MaybeUninit::<ValueType>::uninit(); ROWS]; ROWS];

        // Not entirely sure if transposition or just reading the values
        // would be best, but probably there are faster algorithms anyways.
        // This will work for now.
        let rhs = rhs.transpose();
        // We want the needless range loops, as we use the value to index multiple times.
        #[allow(clippy::needless_range_loop)]
        for i in 0..ROWS {
            #[allow(clippy::needless_range_loop)]
            for j in 0..ROWS {
                data[i][j].write(
                    self.data[i]
                        .iter()
                        .zip(rhs.data[j].iter())
                        .map(|(l, r)| *l * *r)
                        .sum(),
                );
            }
        }

        let ptr = &mut data as *mut _ as *mut [[ValueType; ROWS]; ROWS];
        let transmuted = unsafe { ptr.read() };

        Matrix { data: transmuted }
    }
}

impl<ValueType, const COLS: usize, const ROWS: usize> std::ops::Mul<ValueType>
    for Matrix<ValueType, COLS, ROWS>
where
    ValueType: std::ops::Mul<ValueType, Output = ValueType> + Copy,
{
    type Output = Matrix<ValueType, COLS, ROWS>;

    /// Implement `Matrix<T> * T` operation.
    fn mul(self, rhs: ValueType) -> Self::Output {
        let mut data = [[mem::MaybeUninit::<ValueType>::uninit(); COLS]; ROWS];

        for (elem, lhs) in data.iter_mut().flatten().zip(self.data.iter().flatten()) {
            elem.write(*lhs * rhs);
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
    fn square_matrix() {
        let lhs = m![[1, 2], [3, 4]];
        let rhs = m![[5, 6], [7, 8]];

        let result = lhs * rhs;
        assert_eq!(result.as_slices(), &[[19, 22], [43, 50]]);
    }

    #[test]
    fn rectangular_matrix() {
        let lhs = m![[1, 2, 3], [4, 5, 6]];
        let rhs = m![[7, 8], [9, 10], [11, 12]];

        let result = lhs * rhs;
        assert_eq!(result.as_slices(), &[[58, 64], [139, 154]]);
    }

    #[test]
    fn scalar_mul() {
        let lhs = m![[1, 2], [3, 4]];
        let result = lhs * 3;
        assert_eq!(result.as_slices(), &[[3, 6], [9, 12]]);
    }

    #[test]
    fn scalar_mul_lhs() {
        let rhs = m![[1, 2], [3, 4]];
        let result = 3i32 * rhs;
        assert_eq!(result.as_slices(), &[[3, 6], [9, 12]]);
    }
}
