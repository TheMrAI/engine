use crate::Quaternion;

impl<ValueType> std::ops::Div<ValueType> for Quaternion<ValueType>
where
    ValueType: Copy + std::ops::Div<ValueType, Output = ValueType>,
{
    type Output = Quaternion<ValueType>;

    /// Implement `Quaternion<T> / T` operation.
    fn div(self, rhs: ValueType) -> Self::Output {
        Quaternion::new_parts(self.scalar / rhs, self.vector / rhs)
    }
}

#[cfg(test)]
mod tests {
    use lina::v;

    use crate::Quaternion;

    #[test]
    fn div() {
        let q = Quaternion::new_parts(1, v![2, 4, 7]);
        let result = q / 2;

        assert_eq!(result.scalar, 0);
        assert_eq!(result.vector.as_slice(), [1, 2, 3]);
    }
}
