use lina::vector::Vector;

use crate::Quaternion;

impl<ValueType> std::ops::MulAssign<ValueType> for Quaternion<ValueType>
where
    ValueType: std::ops::MulAssign<ValueType> + Copy,
{
    /// Perform the `Quaternion<T> *= T` operation
    fn mul_assign(&mut self, rhs: ValueType) {
        self.scalar *= rhs;
        self.vector *= rhs;
    }
}

impl<ValueType> std::ops::MulAssign<Quaternion<ValueType>> for Quaternion<ValueType>
where
    ValueType: Default
        + Copy
        + std::ops::Mul<Output = ValueType>
        + std::ops::Sub<Output = ValueType>
        + std::ops::Add<Output = ValueType>,
    Vector<ValueType, 3>:
        std::ops::Mul<Output = ValueType> + std::ops::Mul<ValueType, Output = Vector<ValueType, 3>>,
{
    /// Perform the `Quaternion<T> *= Quaternion<T>` operation.
    fn mul_assign(&mut self, rhs: Quaternion<ValueType>) {
        let scalar = self.scalar;
        let vector = self.vector;

        self.scalar = (scalar * rhs.scalar) - (vector * rhs.vector);
        self.vector = vector.cross(rhs.vector) + (rhs.vector * scalar) + (self.vector * rhs.scalar);
    }
}

#[cfg(test)]
mod tests {
    use lina::v;

    use crate::Quaternion;

    #[test]
    fn scalar() {
        let mut q = Quaternion::new_parts(1, v![2, 3, 4]);
        q *= 3;

        assert_eq!(q.scalar(), 3);
        assert_eq!(q.vector().as_slice(), [6, 9, 12]);
    }

    #[test]
    fn quaternion() {
        let mut q: Quaternion<i32> = Quaternion::new_parts(1, v![2, 3, 4]);
        let p: Quaternion<i32> = Quaternion::new_parts(5, v![6, 7, 8]);
        q *= p;

        assert_eq!(q.scalar(), -60);
        assert_eq!(q.vector().as_slice(), [12, 30, 24]);
    }
}
