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

impl<'addition, ValueType, const LENGTH: usize> std::ops::Sub
    for &'addition Vector<ValueType, LENGTH>
where
    ValueType: Default + Copy,
    &'addition ValueType: std::ops::Sub<&'addition ValueType, Output = ValueType>,
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
    }
}
