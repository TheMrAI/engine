use crate::matrix::Matrix;

impl<ValueType> Matrix<ValueType, 3, 3>
where
    ValueType: Copy
        + std::ops::Mul<Output = ValueType>
        + std::ops::Sub<Output = ValueType>
        + std::ops::Neg<Output = ValueType>,
{
    /// Generate the adjoint matrix.
    pub fn adjoint(&self) -> Matrix<ValueType, 3, 3> {
        let d00 = self[(1, 1)] * self[(2, 2)] - self[(1, 2)] * self[(2, 1)];
        let d10 = self[(0, 1)] * self[(2, 2)] - self[(0, 2)] * self[(2, 1)];
        let d20 = self[(0, 1)] * self[(1, 2)] - self[(0, 2)] * self[(1, 1)];
        let d01 = self[(1, 0)] * self[(2, 2)] - self[(1, 2)] * self[(2, 0)];
        let d11 = self[(0, 0)] * self[(2, 2)] - self[(0, 2)] * self[(2, 0)];
        let d21 = self[(0, 0)] * self[(1, 2)] - self[(0, 2)] * self[(1, 0)];
        let d02 = self[(1, 0)] * self[(2, 1)] - self[(1, 1)] * self[(2, 0)];
        let d12 = self[(0, 0)] * self[(2, 1)] - self[(0, 1)] * self[(2, 0)];
        let d22 = self[(0, 0)] * self[(1, 1)] - self[(0, 1)] * self[(1, 0)];

        Matrix::from_matrix([[d00, -d10, d20], [-d01, d11, -d21], [d02, -d12, d22]])
    }
}

#[cfg(test)]
mod tests {
    use crate::m;
    use float_eq::assert_float_eq;

    #[test]
    fn adjoint_int() {
        let m = m![[1, 2, 3], [4, 5, 6], [0, 0, 9]];
        let result_adjoint = m.adjoint();
        let expected_adjoint = m![[45, -18, -3], [-36, 9, 6], [0, 0, -3]];

        assert_eq!(result_adjoint, expected_adjoint);
    }

    #[test]
    fn adjoint_f32() {
        let m = m![[1.2f32, -2.1, 5.6], [0.0, 1.0, -2.4], [-1.2, 0.8, 3.0]];
        let result_adjoint = m.adjoint();
        let expected_adjoint = m![[4.92, 10.78, -0.56], [2.88, 10.32, 2.88], [1.2, 1.56, 1.2]];

        result_adjoint
            .as_slices()
            .iter()
            .flatten()
            .zip(expected_adjoint.as_slices().iter().flatten())
            .for_each(|(l, r)| assert_float_eq!(l, r, ulps <= 1));
    }
}
