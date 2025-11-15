use crate::Quaternion;

impl<ValueType> std::ops::SubAssign<Quaternion<ValueType>> for Quaternion<ValueType>
where
    ValueType: Copy + std::ops::SubAssign,
{
    /// Implement `Quaternion<T> -= Quaternion<T>` operation.
    fn sub_assign(&mut self, rhs: Quaternion<ValueType>) {
        self.scalar -= rhs.scalar;
        self.vector -= rhs.vector;
    }
}

#[cfg(test)]
mod tests {
    use lina::v;

    use crate::Quaternion;

    #[test]
    fn sub_assign() {
        let mut q: Quaternion<i32> = Quaternion::new_parts(1, v![2, 3, 4]);
        let p: Quaternion<i32> = Quaternion::new_parts(5, v![6, 7, 8]);
        q -= p;

        assert_eq!(q.scalar(), -4);
        assert_eq!(q.vector().as_slice(), [-4, -4, -4]);
    }
}
