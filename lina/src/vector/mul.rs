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
}
