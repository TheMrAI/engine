use crate::Quaternion;

impl<ValueType> std::ops::Sub<Quaternion<ValueType>> for Quaternion<ValueType>
where
    ValueType: Copy + std::ops::Sub<Output = ValueType>,
{
    type Output = Quaternion<ValueType>;

    /// Implement `Quaternion<T> - Quaternion<T>` operation.
    ///
    /// For quaternion `q` and `p`:
    /// ```text
    /// q = [s, v]
    /// p = [s', v']
    /// q - p = [s - s', v - v']
    /// ```
    fn sub(self, rhs: Quaternion<ValueType>) -> Self::Output {
        let scalar = self.scalar - rhs.scalar;
        let vector = self.vector - rhs.vector;

        Quaternion::new_parts(scalar, vector)
    }
}

#[cfg(test)]
mod tests {
    use lina::v;

    use crate::Quaternion;

    #[test]
    fn sub() {
        let q: Quaternion<i32> = Quaternion::new_parts(1, v![2, 3, 4]);
        let p: Quaternion<i32> = Quaternion::new_parts(5, v![6, 7, 8]);
        let result = q - p;

        assert_eq!(result.scalar(), -4);
        assert_eq!(result.vector().as_slice(), [-4, -4, -4]);
    }
}
