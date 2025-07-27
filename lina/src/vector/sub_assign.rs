use super::vector::Vector;

impl<ValueType, const LENGTH: usize> std::ops::SubAssign<Vector<ValueType, LENGTH>>
    for Vector<ValueType, LENGTH>
where
    ValueType: std::ops::SubAssign + Copy,
{
    /// Implement `Vector<T> -= Vector<T>` operation.
    fn sub_assign(&mut self, rhs: Vector<ValueType, LENGTH>) {
        self.data.iter_mut().zip(rhs.data).for_each(|(lhs, rhs)| {
            *lhs -= rhs;
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::v;

    #[test]
    fn sub() {
        let mut lhs = v![1, 2, 3];
        let rhs = v![4, 5, 6];

        lhs -= rhs;
        assert_eq!(lhs.as_slice(), &[-3, -3, -3]);
    }
}
