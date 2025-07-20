use super::{Vector, v};

impl<ValueType, const LENGTH: usize> std::ops::Mul<ValueType> for Vector<ValueType, LENGTH>
where
    ValueType: std::ops::Mul<Output = ValueType> + Default + Copy,
{
    type Output = Vector<ValueType, LENGTH>;

    fn mul(self, rhs: ValueType) -> Self::Output {
        let mut result = v![ValueType::default(); LENGTH];
        self.data.iter().enumerate().for_each(|(i, value)| {
            result.data[i] = *value * rhs;
        });
        result
    }
}

// Unfortunately we can't implement the other direction where a
// a number is on the LHS and a Vector on the RHS, because of the Orphan
// rule. In this case both the LHS and the std::ops::Mul are foreign entities.

#[cfg(test)]
mod tests {
    use crate::v;

    #[test]
    fn scalar_mul() {
        let v = v![1, 2, 3];
        let result = v * 3;
        assert_eq!(result.as_slice(), [3, 6, 9]);
    }
}
