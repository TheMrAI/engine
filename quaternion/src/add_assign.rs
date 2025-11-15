use lina::vector::Vector;

use crate::Quaternion;

impl<ValueType> std::ops::AddAssign<Quaternion<ValueType>> for Quaternion<ValueType>
where
    ValueType: Default + Copy + std::ops::AddAssign,
    Vector<ValueType, 3>: std::ops::AddAssign,
{
    /// Perform the `Quaternion<T> += Quaternion<T>` operation.
    fn add_assign(&mut self, rhs: Quaternion<ValueType>) {
        self.scalar += rhs.scalar;
        self.vector += rhs.vector;
    }
}

#[cfg(test)]
mod tests {
    use lina::v;

    use crate::Quaternion;

    #[test]
    fn add_assign() {
        let mut q: Quaternion<i32> = Quaternion::new_parts(1, v![2, 3, 4]);
        let p: Quaternion<i32> = Quaternion::new_parts(5, v![6, 7, 8]);
        q += p;

        assert_eq!(q.scalar(), 6);
        assert_eq!(q.vector().as_slice(), [8, 10, 12]);
    }
}
