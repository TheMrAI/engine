use super::vector::Vector;

impl<ValueType, const LENGTH: usize> std::ops::Sub for Vector<ValueType, LENGTH>
where
    ValueType: Default + Copy + std::ops::Sub<Output = ValueType>,
    for<'a> &'a ValueType: std::ops::Sub<Output = ValueType>,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        &self - &rhs
    }
}

impl<'sub, ValueType, const LENGTH: usize> std::ops::Sub for &'sub Vector<ValueType, LENGTH>
where
    ValueType: Default + Copy,
    &'sub ValueType: std::ops::Sub<&'sub ValueType, Output = ValueType>,
{
    type Output = Vector<ValueType, LENGTH>;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut data = [ValueType::default(); LENGTH];
        self.data
            .iter()
            .zip(rhs.data.iter())
            .enumerate()
            .for_each(|(i, (lhs, rhs))| {
                data[i] = lhs - rhs;
            });
        Vector::<ValueType, LENGTH> { data }
    }
}

#[cfg(test)]
mod tests {
    use crate::v;

    #[test]
    fn sub() {
        let lhs = v![1, 2, 3];
        let rhs = v![4, 5, 6];

        let result = lhs - rhs;
        assert_eq!(result.as_slice(), &[-3, -3, -3]);
        // error: used of moved value
        // let c = lhs - rhs;
    }

    #[test]
    fn sub_by_ref() {
        let lhs = v![1, 2, 3];
        let rhs = v![4, 5, 6];

        let result = &lhs - &rhs;
        assert_eq!(result.as_slice(), &[-3, -3, -3]);
        let result_2 = &lhs - &rhs;
        assert_eq!(result_2.as_slice(), &[-3, -3, -3]);
    }
}
