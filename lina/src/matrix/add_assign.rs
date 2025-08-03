use super::Matrix;

impl<ValueType, const COLS: usize, const ROWS: usize> std::ops::AddAssign
    for Matrix<ValueType, COLS, ROWS>
where
    ValueType: std::ops::AddAssign<ValueType>,
{
    /// Implement `Vector<T> += Vector<T>` operation.
    fn add_assign(&mut self, rhs: Self) {
        // Given that the two matrices have the same shape, we can simply flatten the internal structures
        // and apply the operation per element.
        self.data
            .iter_mut()
            .flatten()
            .zip(rhs.data.into_iter().flatten())
            .for_each(|(lhs, rhs)| {
                *lhs += rhs;
            });
    }
}

#[cfg(test)]
mod tests {
    use crate::m;

    #[test]
    fn add_assign() {
        let mut lhs = m![[1, 2], [3, 4]];
        let rhs = m![[5, 6], [7, 8]];

        lhs += rhs;
        assert_eq!(lhs.as_slices(), &[[6, 8], [10, 12]]);
    }
}
