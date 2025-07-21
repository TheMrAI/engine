use super::{Vector, v};

// * operator for Vector<T> * T
impl<ValueType, const LENGTH: usize> std::ops::Mul<ValueType> for Vector<ValueType, LENGTH>
where
    ValueType: std::ops::Mul<Output = ValueType> + Default + Copy,
{
    type Output = Vector<ValueType, LENGTH>;

    /// Performs the `Vector<T> * T` operation.
    fn mul(self, rhs: ValueType) -> Self::Output {
        let mut result = v![ValueType::default(); LENGTH];
        self.data.iter().enumerate().for_each(|(i, value)| {
            result.data[i] = *value * rhs;
        });
        result
    }
}

impl<ValueType, const LENGTH: usize> std::ops::Mul<ValueType> for &Vector<ValueType, LENGTH>
where
    ValueType: std::ops::Mul<Output = ValueType> + Default + Copy,
{
    type Output = Vector<ValueType, LENGTH>;

    /// Performs the `&Vector<T> * T` operation.
    fn mul(self, rhs: ValueType) -> Self::Output {
        let mut result = v![ValueType::default(); LENGTH];
        self.data.iter().enumerate().for_each(|(i, value)| {
            result.data[i] = *value * rhs;
        });
        result
    }
}

impl<ValueType, const LENGTH: usize> std::ops::Mul<&ValueType> for Vector<ValueType, LENGTH>
where
    ValueType: std::ops::Mul<Output = ValueType> + Default + Copy,
{
    type Output = Vector<ValueType, LENGTH>;

    /// Performs the `Vector<T> * &T` operation.
    fn mul(self, rhs: &ValueType) -> Self::Output {
        let mut result = v![ValueType::default(); LENGTH];
        self.data.iter().enumerate().for_each(|(i, value)| {
            result.data[i] = *value * *rhs;
        });
        result
    }
}

impl<ValueType, const LENGTH: usize> std::ops::Mul<&ValueType> for &Vector<ValueType, LENGTH>
where
    ValueType: std::ops::Mul<Output = ValueType> + Default + Copy,
{
    type Output = Vector<ValueType, LENGTH>;

    /// Performs the `&Vector<T> * &T` operation.
    fn mul(self, rhs: &ValueType) -> Self::Output {
        let mut result = v![ValueType::default(); LENGTH];
        self.data.iter().enumerate().for_each(|(i, value)| {
            result.data[i] = *value * *rhs;
        });
        result
    }
}

// Implement the LHS scalar multiplication operators for built in types.
// For custom types the user must provide the implementation given the Orphan rule.

macro_rules! lhs_scalar_mul_impl {
    ($($T: ty),* $(,)*) => {$(
        // * operator for Vector<T> * T
        impl<const LENGTH: usize> std::ops::Mul<Vector<$T, LENGTH>> for $T
        where
            $T: std::ops::Mul<Output = $T> + Default + Copy,
        {
            type Output = Vector<$T, LENGTH>;

            /// Performs the `T * Vector<T>` operation.
            fn mul(self, rhs: Vector<$T, LENGTH>) -> Self::Output {
                rhs * self
            }
        }

        impl<const LENGTH: usize> std::ops::Mul<&Vector<$T, LENGTH>> for $T
        where
            $T: std::ops::Mul<Output = $T> + Default + Copy,
        {
            type Output = Vector<$T, LENGTH>;

            /// Performs the `T * &Vector<T>` operation.
            fn mul(self, rhs: &Vector<$T, LENGTH>) -> Self::Output {
                rhs * self
            }
        }

        impl<const LENGTH: usize> std::ops::Mul<Vector<$T, LENGTH>> for &$T
        where
            $T: std::ops::Mul<Output = $T> + Default + Copy,
        {
            type Output = Vector<$T, LENGTH>;

            /// Performs the `&T * Vector<T>` operation.
            fn mul(self, rhs: Vector<$T, LENGTH>) -> Self::Output {
                rhs * self
            }
        }

        impl<const LENGTH: usize> std::ops::Mul<&Vector<$T, LENGTH>> for &$T
        where
            $T: std::ops::Mul<Output = $T> + Default + Copy,
        {
            type Output = Vector<$T, LENGTH>;

            /// Performs the `&T * &Vector<T>` operation.
            fn mul(self, rhs: &Vector<$T, LENGTH>) -> Self::Output {
                rhs * self
            }
        }
    )*};
}

lhs_scalar_mul_impl!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64
);

#[cfg(test)]
mod tests {
    use crate::v;

    #[test]
    fn scalar_mul() {
        let v = v![1, 2, 3];
        let result = v * 3;
        assert_eq!(result.as_slice(), [3, 6, 9]);
    }

    #[test]
    fn scalar_mul_by_ref() {
        let v = v![1, 2, 3];
        let result = &v * 3;
        assert_eq!(result.as_slice(), [3, 6, 9]);
    }

    #[test]
    fn scalar_ref_mul() {
        let k: &i32 = &3;
        let v = v![1, 2, 3];
        let result = v * k;
        assert_eq!(result.as_slice(), [3, 6, 9]);
    }

    #[test]
    fn scalar_ref_mul_by_ref() {
        let k: &i32 = &3;
        let v = v![1, 2, 3];
        let result = &v * k;
        assert_eq!(result.as_slice(), [3, 6, 9]);
    }

    #[test]
    fn scalar_mul_lhs() {
        let v = v![1, 2, 3];
        let result = 3i32 * v;
        assert_eq!(result.as_slice(), [3, 6, 9]);
    }

    #[test]
    fn scalar_mul_by_ref_lhs() {
        let v = v![1, 2, 3];
        let result = 3i32 * &v;
        assert_eq!(result.as_slice(), [3, 6, 9]);
    }

    #[test]
    fn scalar_ref_mul_lhs() {
        let k: &i32 = &3;
        let v = v![1, 2, 3];
        let result = k * v;
        assert_eq!(result.as_slice(), [3, 6, 9]);
    }

    #[test]
    fn scalar_ref_mul_by_ref_lhs() {
        let k: &i32 = &3;
        let v = v![1, 2, 3];
        let result = k * &v;
        assert_eq!(result.as_slice(), [3, 6, 9]);
    }
}
