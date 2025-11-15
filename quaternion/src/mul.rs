use lina::vector::Vector;

use crate::Quaternion;

impl<ValueType> std::ops::Mul<ValueType> for Quaternion<ValueType>
where
    ValueType: Default + Copy + std::ops::Mul<Output = ValueType>,
{
    type Output = Quaternion<ValueType>;

    /// Performs the `Quaternion<T> * T` operation
    ///
    /// For a given quaternion q:
    /// ```text
    /// q = [s, v]
    /// ```
    /// `q * r` will produce:
    /// ```text
    /// q*r = [r*s, r*v]
    /// ```
    fn mul(self, rhs: ValueType) -> Self::Output {
        Quaternion::new_parts(self.scalar * rhs, self.vector * rhs)
    }
}

macro_rules! lhs_scalar_mul_impl {
    ($($T: ty),* $(,)*) => {$(
        impl std::ops::Mul<Quaternion<$T>> for $T
        where
            Quaternion<$T>: std::ops::Mul<$T, Output = Quaternion<$T>>,
        {
            type Output = Quaternion<$T>;

            /// Perform the `T * Quaternion<T>` operation
            ///
            /// The commutative pair for `Quaternion<T> * T` operations.
            fn mul(self, rhs: Quaternion<$T>) -> Self::Output {
                rhs * self
            }
        }
    )*};
}

lhs_scalar_mul_impl!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64
);

impl<ValueType> std::ops::Mul<Quaternion<ValueType>> for Quaternion<ValueType>
where
    ValueType: Default
        + Copy
        + std::ops::Mul<Output = ValueType>
        + std::ops::Sub<Output = ValueType>
        + std::ops::Add<Output = ValueType>,
    Vector<ValueType, 3>:
        std::ops::Mul<Output = ValueType> + std::ops::Mul<ValueType, Output = Vector<ValueType, 3>>,
{
    type Output = Quaternion<ValueType>;

    /// Perform the `Quaternion<T> * Quaternion<T>` operation.
    ///
    /// For quaternion `q` and `p`:
    /// ```text
    /// q = [s, v]
    /// p = [s', v']
    /// qp = [ss' - v*v', v x v' + sv' + s'v]
    /// ```
    ///
    /// The operation is not commutative.
    /// ```text
    /// qp != pq
    /// ```
    fn mul(self, rhs: Quaternion<ValueType>) -> Self::Output {
        let scalar = (self.scalar * rhs.scalar) - (self.vector * rhs.vector);
        let vector =
            self.vector.cross(rhs.vector) + (rhs.vector * self.scalar) + (self.vector * rhs.scalar);

        Quaternion::new_parts(scalar, vector)
    }
}

#[cfg(test)]
mod tests {
    use lina::v;

    use crate::Quaternion;

    #[test]
    fn scalar_mul() {
        let q = Quaternion::new_parts(1, v![2, 3, 4]);
        let result = q * 3;

        assert_eq!(result.scalar(), 3);
        assert_eq!(result.vector().as_slice(), [6, 9, 12]);
    }

    #[test]
    fn scalar_mul_lhs() {
        let q: Quaternion<i32> = Quaternion::new_parts(1, v![2, 3, 4]);
        let result = 3i32 * q;

        assert_eq!(result.scalar(), 3);
        assert_eq!(result.vector().as_slice(), [6, 9, 12]);
    }

    #[test]
    fn quaternion_mul() {
        let q: Quaternion<i32> = Quaternion::new_parts(1, v![2, 3, 4]);
        let p: Quaternion<i32> = Quaternion::new_parts(5, v![6, 7, 8]);
        let result = q * p;

        assert_eq!(result.scalar(), -60);
        assert_eq!(result.vector().as_slice(), [12, 30, 24]);
    }

    #[test]
    fn quaternion_mul_non_commutative() {
        let q: Quaternion<i32> = Quaternion::new_parts(1, v![2, 3, 4]);
        let p: Quaternion<i32> = Quaternion::new_parts(5, v![6, 7, 8]);

        assert_ne!(p * q, q * p);
    }
}
