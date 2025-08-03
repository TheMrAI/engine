use super::Matrix;

// We could provide a MulAssign for those cases where both LHS and RHS is a square matrix
// but I am unsure on how you could define the restriction that two const template variables
// should be equal.

impl<ValueType, const COLS: usize, const ROWS: usize> std::ops::MulAssign<ValueType>
    for Matrix<ValueType, COLS, ROWS>
where
    ValueType: std::ops::MulAssign<ValueType> + Copy,
{
    /// Implement `Matrix<T> *= T` operation.
    fn mul_assign(&mut self, rhs: ValueType) {
        for elem in self.data.iter_mut().flatten() {
            *elem *= rhs;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::m;

    #[test]
    fn scalar_mul() {
        let mut lhs = m![[1, 2], [3, 4]];
        let rhs = 3;

        lhs *= rhs;
        assert_eq!(lhs.as_slices(), &[[3, 6], [9, 12]]);
    }
}
