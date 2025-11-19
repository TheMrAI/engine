use crate::matrix::Matrix;

impl<ValueType> Matrix<ValueType, 3, 3>
where
    ValueType: Copy
        + std::convert::From<i8>
        + std::cmp::PartialEq
        + std::ops::Add<Output = ValueType>
        + std::ops::Mul<Output = ValueType>
        + std::ops::Div<Output = ValueType>
        + std::ops::Sub<Output = ValueType>
        + std::ops::Neg<Output = ValueType>
        + std::ops::Mul<Matrix<ValueType, 3, 3>, Output = Matrix<ValueType, 3, 3>>,
{
    /// Calculate the inverse of [Matrix].
    ///
    /// The [Matrix] must be a square matrix and its
    /// determinant cannot be zero.
    ///
    /// None is returned if the determinant was zero otherwise the inverse is
    /// calculated.
    ///
    /// Given an `M` matrix, its inverse `M^-1` and identity matrix `I`:
    /// ```text
    /// M*M^-1 = I
    /// ```
    pub fn inverse(&self) -> Option<Matrix<ValueType, 3, 3>> {
        let determinant = self.determinant();
        if determinant == ValueType::from(0) {
            return None;
        }
        Some((ValueType::from(1) / self.determinant()) * self.adjoint())
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use crate::m;

    #[test]
    fn inverse_zero_int() {
        let m = m![[1, 2, 3], [4, 5, 6], [7, 8, 9]];
        let result_inverse = m.inverse();

        assert_eq!(result_inverse, None);
    }

    #[test]
    fn inverse_non_zero_int() {
        let m = m![[1, 2, 3], [4, 5, 6], [0, 0, 9]];
        let result_inverse = m.inverse().unwrap();
        let expected_inverse = m![[0, 0, 0], [0, 0, 0], [0, 0, 0]];

        assert_eq!(expected_inverse, result_inverse);
    }

    #[test]
    fn inverse_non_zero_f32() {
        let m = m![[1.2f32, -2.1, 5.6], [0.0, 1.0, -2.4], [-1.2, 0.8, 3.0]];
        let result_inverse = m.inverse().unwrap();
        let expected_inverse = m![
            [0.748175182481752, 1.639294403892944, -0.085158150851582],
            [0.437956204379562, 1.569343065693431, 0.437956204379562],
            [0.182481751824818, 0.237226277372263, 0.182481751824818]
        ];

        // assert_eq!(expected_inverse, result_inverse);
        result_inverse
            .as_slices()
            .iter()
            .flatten()
            .zip(expected_inverse.as_slices().iter().flatten())
            .for_each(|(l, r)| assert_float_eq!(l, r, ulps <= 2));
    }
}
