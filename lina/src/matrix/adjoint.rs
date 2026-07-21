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

impl<ValueType> Matrix<ValueType, 4, 4>
where
    ValueType: Copy
        + std::ops::Add<Output = ValueType>
        + std::ops::Mul<Output = ValueType>
        + std::ops::Sub<Output = ValueType>
        + std::ops::Neg<Output = ValueType>,
{
    /// Generate the adjoint matrix.
    pub fn adjoint(&self) -> Matrix<ValueType, 4, 4> {
        let d00 = self[(1, 1)] * self[(2, 2)] * self[(3, 3)]
            + self[(2, 1)] * self[(3, 2)] * self[(1, 3)]
            + self[(1, 2)] * self[(2, 3)] * self[(3, 1)]
            - self[(3, 1)] * self[(2, 2)] * self[(1, 3)]
            - self[(2, 1)] * self[(1, 2)] * self[(3, 3)]
            - self[(3, 2)] * self[(2, 3)] * self[(1, 1)];
        let d10 = self[(0, 1)] * self[(2, 2)] * self[(3, 3)]
            + self[(2, 1)] * self[(3, 2)] * self[(0, 3)]
            + self[(0, 2)] * self[(2, 3)] * self[(3, 1)]
            - self[(3, 1)] * self[(2, 2)] * self[(0, 3)]
            - self[(2, 1)] * self[(0, 2)] * self[(3, 3)]
            - self[(3, 2)] * self[(2, 3)] * self[(0, 1)];
        let d20 = self[(0, 1)] * self[(1, 2)] * self[(3, 3)]
            + self[(1, 1)] * self[(3, 2)] * self[(0, 3)]
            + self[(0, 2)] * self[(1, 3)] * self[(3, 1)]
            - self[(3, 1)] * self[(1, 2)] * self[(0, 3)]
            - self[(1, 1)] * self[(0, 2)] * self[(3, 3)]
            - self[(3, 2)] * self[(1, 3)] * self[(0, 1)];
        let d30 = self[(0, 1)] * self[(1, 2)] * self[(2, 3)]
            + self[(1, 1)] * self[(2, 2)] * self[(0, 3)]
            + self[(0, 2)] * self[(1, 3)] * self[(2, 1)]
            - self[(2, 1)] * self[(1, 2)] * self[(0, 3)]
            - self[(1, 1)] * self[(0, 2)] * self[(2, 3)]
            - self[(2, 2)] * self[(1, 3)] * self[(0, 1)];
        let d01 = self[(1, 0)] * self[(2, 2)] * self[(3, 3)]
            + self[(2, 0)] * self[(3, 2)] * self[(1, 3)]
            + self[(1, 2)] * self[(2, 3)] * self[(3, 0)]
            - self[(3, 0)] * self[(2, 2)] * self[(1, 3)]
            - self[(3, 2)] * self[(2, 3)] * self[(1, 0)]
            - self[(2, 0)] * self[(1, 2)] * self[(3, 3)];
        let d11 = self[(0, 0)] * self[(2, 2)] * self[(3, 3)]
            + self[(2, 0)] * self[(3, 2)] * self[(0, 3)]
            + self[(0, 2)] * self[(2, 3)] * self[(3, 0)]
            - self[(3, 0)] * self[(2, 2)] * self[(0, 3)]
            - self[(3, 2)] * self[(2, 3)] * self[(0, 0)]
            - self[(2, 0)] * self[(0, 2)] * self[(3, 3)];
        let d21 = self[(0, 0)] * self[(1, 2)] * self[(3, 3)]
            + self[(1, 0)] * self[(3, 2)] * self[(0, 3)]
            + self[(0, 2)] * self[(1, 3)] * self[(3, 0)]
            - self[(3, 0)] * self[(1, 2)] * self[(0, 3)]
            - self[(3, 2)] * self[(1, 3)] * self[(0, 0)]
            - self[(1, 0)] * self[(0, 2)] * self[(3, 3)];
        let d31 = self[(0, 0)] * self[(1, 2)] * self[(2, 3)]
            + self[(1, 0)] * self[(2, 2)] * self[(0, 3)]
            + self[(0, 2)] * self[(1, 3)] * self[(2, 0)]
            - self[(2, 0)] * self[(1, 2)] * self[(0, 3)]
            - self[(2, 2)] * self[(1, 3)] * self[(0, 0)]
            - self[(1, 0)] * self[(0, 2)] * self[(2, 3)];
        let d02 = self[(1, 0)] * self[(2, 1)] * self[(3, 3)]
            + self[(2, 0)] * self[(3, 1)] * self[(1, 3)]
            + self[(1, 1)] * self[(2, 3)] * self[(3, 0)]
            - self[(3, 0)] * self[(2, 1)] * self[(1, 3)]
            - self[(3, 1)] * self[(2, 3)] * self[(1, 0)]
            - self[(2, 0)] * self[(1, 1)] * self[(3, 3)];
        let d12 = self[(0, 0)] * self[(2, 1)] * self[(3, 3)]
            + self[(0, 1)] * self[(2, 3)] * self[(3, 0)]
            + self[(2, 0)] * self[(3, 1)] * self[(0, 3)]
            - self[(3, 0)] * self[(2, 1)] * self[(0, 3)]
            - self[(3, 1)] * self[(2, 3)] * self[(0, 0)]
            - self[(2, 0)] * self[(0, 1)] * self[(3, 3)];
        let d22 = self[(0, 0)] * self[(1, 1)] * self[(3, 3)]
            + self[(1, 0)] * self[(3, 1)] * self[(0, 3)]
            + self[(0, 1)] * self[(1, 3)] * self[(3, 0)]
            - self[(3, 0)] * self[(1, 1)] * self[(0, 3)]
            - self[(3, 1)] * self[(1, 3)] * self[(0, 0)]
            - self[(1, 0)] * self[(0, 1)] * self[(3, 3)];
        let d32 = self[(0, 0)] * self[(1, 1)] * self[(2, 3)]
            + self[(1, 0)] * self[(2, 1)] * self[(0, 3)]
            + self[(0, 1)] * self[(1, 3)] * self[(2, 0)]
            - self[(2, 0)] * self[(1, 1)] * self[(0, 3)]
            - self[(2, 1)] * self[(1, 3)] * self[(0, 0)]
            - self[(1, 0)] * self[(0, 1)] * self[(2, 3)];
        let d03 = self[(1, 0)] * self[(2, 1)] * self[(3, 2)]
            + self[(1, 1)] * self[(2, 2)] * self[(3, 0)]
            + self[(2, 0)] * self[(3, 1)] * self[(1, 2)]
            - self[(3, 0)] * self[(2, 1)] * self[(1, 2)]
            - self[(3, 1)] * self[(2, 2)] * self[(1, 0)]
            - self[(2, 0)] * self[(1, 1)] * self[(3, 2)];
        let d13 = self[(0, 0)] * self[(2, 1)] * self[(3, 2)]
            + self[(2, 0)] * self[(3, 1)] * self[(0, 2)]
            + self[(0, 1)] * self[(2, 2)] * self[(3, 0)]
            - self[(3, 0)] * self[(2, 1)] * self[(0, 2)]
            - self[(2, 0)] * self[(0, 1)] * self[(3, 2)]
            - self[(3, 1)] * self[(2, 2)] * self[(0, 0)];
        let d23 = self[(0, 0)] * self[(1, 1)] * self[(3, 2)]
            + self[(1, 0)] * self[(3, 1)] * self[(0, 2)]
            + self[(0, 1)] * self[(1, 2)] * self[(3, 0)]
            - self[(3, 0)] * self[(1, 1)] * self[(0, 2)]
            - self[(1, 0)] * self[(0, 1)] * self[(3, 2)]
            - self[(3, 1)] * self[(1, 2)] * self[(0, 0)];
        let d33 = self[(0, 0)] * self[(1, 1)] * self[(2, 2)]
            + self[(1, 0)] * self[(2, 1)] * self[(0, 2)]
            + self[(0, 1)] * self[(1, 2)] * self[(2, 0)]
            - self[(2, 0)] * self[(1, 1)] * self[(0, 2)]
            - self[(1, 0)] * self[(0, 1)] * self[(2, 2)]
            - self[(2, 1)] * self[(1, 2)] * self[(0, 0)];

        Matrix::from_matrix([
            [d00, -d10, d20, -d30],
            [-d01, d11, -d21, d31],
            [d02, -d12, d22, -d32],
            [-d03, d13, -d23, d33],
        ])
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use crate::m;

    #[test]
    fn adjoint_3x3_int_zero() {
        let m = m![[3, 3, 3], [3, 3, 3], [3, 3, 3]];
        let result_adjoint = m.adjoint();
        let expected_adjoint = m![[0, 0, 0], [0, 0, 0], [0, 0, 0]];

        assert_eq!(result_adjoint, expected_adjoint);
    }

    #[test]
    fn adjoint_3x3_int() {
        let m = m![[1, 2, 3], [4, 5, 6], [0, 0, 9]];
        let result_adjoint = m.adjoint();
        let expected_adjoint = m![[45, -18, -3], [-36, 9, 6], [0, 0, -3]];

        assert_eq!(result_adjoint, expected_adjoint);
    }

    #[test]
    fn adjoint_3x3_f32() {
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

    #[test]
    fn adjoint_4x4_int() {
        let m = m![
            [1, 2, 3, 4],
            [5, 6, 7, 8],
            [9, 10, 11, 12],
            [13, 14, 15, 16]
        ];
        let result_adjoint = m.adjoint();
        let expected_adjoint = m![[0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]];

        assert_eq!(result_adjoint, expected_adjoint);
    }

    #[test]
    fn adjoint_4x4_f32() {
        let m = m![
            [1.2f32, 2.0, 2.1, 3.0],
            [-3.2, 4.5, 5.6, 1.9],
            [2.21, 4.6, 1.2, 0.3],
            [1.8, 1.23, 0.78, 0.22]
        ];
        let result_adjoint = m.adjoint();
        let expected_adjoint = m![
            [0.547, -5.0457, -7.8053, 46.761],
            [0.6235001, -1.1596204, 29.3676, -38.5341],
            [-12.38273, 15.8059, -29.4764, 72.545],
            [35.94102, -8.272768, 4.177199, -43.796494]
        ];

        result_adjoint
            .as_slices()
            .iter()
            .flatten()
            .zip(expected_adjoint.as_slices().iter().flatten())
            .for_each(|(l, r)| assert_float_eq!(l, r, ulps <= 1));
    }
}
