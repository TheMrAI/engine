use crate::Quaternion;

impl<ValueType> std::ops::DivAssign<ValueType> for Quaternion<ValueType>
where
    ValueType: Copy + std::ops::DivAssign<ValueType>,
{
    /// Implement `Quaternion<T> /= T` operation.
    fn div_assign(&mut self, rhs: ValueType) {
        self.scalar /= rhs;
        self.vector /= rhs;
    }
}

#[cfg(test)]
mod tests {
    use lina::v;

    use crate::Quaternion;

    #[test]
    fn div_assign() {
        let mut q = Quaternion::new_parts(1, v![2, 4, 7]);
        q /= 2;

        assert_eq!(q.scalar, 0);
        assert_eq!(q.vector.as_slice(), [1, 2, 3]);
    }
}
