use crate::matrix::Matrix;

impl<ValueType> Matrix<ValueType, 3, 3>
where
    ValueType: Copy
        + std::ops::Mul<Output = ValueType>
        + std::ops::Sub<Output = ValueType>
        + std::ops::Add<Output = ValueType>,
{
    pub fn determinant(&self) -> ValueType {
        self[(0, 0)] * self[(1, 1)] * self[(2, 2)]
            + self[(0, 1)] * self[(1, 2)] * self[(2, 0)]
            + self[(0, 2)] * self[(1, 0)] * self[(2, 1)]
            - self[(0, 2)] * self[(1, 1)] * self[(2, 0)]
            - self[(0, 1)] * self[(1, 0)] * self[(2, 2)]
            - self[(0, 0)] * self[(1, 2)] * self[(2, 1)]
    }
}

#[cfg(test)]
mod tests{
    use float_eq::assert_float_eq;

    use crate::m;

    #[test]
    fn determinant_zero_int() {
        let m = m![[1, 2, 3], [4, 5, 6], [7, 8, 9]];
        let result_determinant = m.determinant();
        let expected_determinant = 0;
        
        assert_eq!(result_determinant, expected_determinant);
    }

    #[test]
    fn determinant_non_zero_int() {
        let m = m![[1, 2, 3], [4, 5, 6], [0, 0, 9]];
        let result_determinant = m.determinant();
        let expected_determinant = -27;
        
        assert_eq!(result_determinant, expected_determinant);
    }

    #[test]
    fn determinant_non_zero_f32() {
        let m = m![[1.2f32, -2.1, 5.6], [0.0, 1.0, -2.4], [-1.2, 0.8, 3.0]];
        let result_determinant = m.determinant();
        let expected_determinant = 6.576;
        
        assert_float_eq!(result_determinant, expected_determinant, ulps <= 1);
    }
}