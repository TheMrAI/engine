use super::vector::Vector;

impl<ValueType, const LENGTH: usize> std::ops::AddAssign for Vector<ValueType, LENGTH>
where
    ValueType: std::ops::AddAssign<ValueType>,
{
    /// Implement `Vector<T> += Vector<T>` operation.
    fn add_assign(&mut self, rhs: Self) {
        self.data.iter_mut().zip(rhs.data).for_each(|(lhs, rhs)| {
            *lhs += rhs;
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::v;

    #[test]
    fn add_assign() {
        let mut lhs = v![1, 2, 3];
        let rhs = v![4, 5, 6];

        lhs += rhs;
        assert_eq!(lhs.as_slice(), &[5, 7, 9]);
    }
}
