use lina::vector::Vector;

use crate::Quaternion;

impl<ValueType> std::ops::Add<Quaternion<ValueType>> for Quaternion<ValueType>
where
    ValueType: Default + Copy + std::ops::Add<Output = ValueType>,
    Vector<ValueType, 3>: std::ops::Add<Output = Vector<ValueType, 3>>,
{
    type Output = Quaternion<ValueType>;

    /// Perform the `Quaternion<T> + Quaternion<T>` operation.
    ///
    /// For quaternion `q` and `p`:
    /// ```text
    /// q = [s, v]
    /// p = [s', v']
    /// q + p = [s + s', v + v']
    /// ```
    fn add(self, rhs: Quaternion<ValueType>) -> Self::Output {
        let scalar = self.scalar + rhs.scalar;
        let vector = self.vector + rhs.vector;

        Quaternion::new_parts(scalar, vector)
    }
}

#[cfg(test)]
mod tests {
    use lina::v;

    use crate::Quaternion;

    #[test]
    fn add() {
        let q: Quaternion<i32> = Quaternion::new_parts(1, v![2, 3, 4]);
        let p: Quaternion<i32> = Quaternion::new_parts(5, v![6, 7, 8]);
        let result = q + p;

        assert_eq!(result.scalar(), 6);
        assert_eq!(result.vector().as_slice(), [8, 10, 12]);
    }
}
