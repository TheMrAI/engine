use crate::{Vector, v};

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

// Can't do this because of Orphan rule
// impl<ValueType, const LENGTH: usize> std::ops::Mul<Vector<ValueType, LENGTH>>
//     for ValueType
// where
//     ValueType: std::ops::Mul<Output = ValueType> + Default + Copy,
// {
//     type Output = Vector<ValueType, LENGTH>;

//     fn mul(self, rhs: Vector<ValueType, LENGTH>) -> Self::Output {
//         rhs * self
//     }
// }

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
