use super::vector::Vector;

impl<ValueType, const LENGTH: usize> std::ops::Add for Vector<ValueType, LENGTH>
where
    ValueType: Default + Copy + std::ops::Add<Output = ValueType>,
    for<'a> &'a ValueType: std::ops::Add<Output = ValueType>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        &self + &rhs
    }
}

impl<'addition, ValueType, const LENGTH: usize> std::ops::Add
    for &'addition Vector<ValueType, LENGTH>
where
    ValueType: Default + Copy,
    &'addition ValueType: std::ops::Add<&'addition ValueType, Output = ValueType>,
{
    type Output = Vector<ValueType, LENGTH>;

    fn add(self, rhs: Self) -> Self::Output {
        let mut data = [ValueType::default(); LENGTH];
        self.data
            .iter()
            .zip(rhs.data.iter())
            .enumerate()
            .for_each(|(i, (lhs, rhs))| {
                data[i] = lhs + rhs;
            });
        Vector::<ValueType, LENGTH> { data }
    }
}

impl<ValueType, const LENGTH: usize> std::ops::AddAssign for Vector<ValueType, LENGTH>
where
    ValueType: std::ops::AddAssign<ValueType>,
{
    fn add_assign(&mut self, rhs: Self) {
        self.data.iter_mut().zip(rhs.data).for_each(|(lhs, rhs)| {
            *lhs += rhs;
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::v;

    #[test]
    fn add_by_value() {
        let lhs = v![1, 2, 3];
        let rhs = v![4, 5, 6];

        let result = lhs + rhs;
        assert_eq!(result.as_slice(), &[5, 7, 9]);
        // error: used of moved value
        // let c = lhs + rhs;
    }

    #[test]
    fn add_by_ref() {
        let lhs = v![1, 2, 3];
        let rhs = v![4, 5, 6];

        let result = &lhs + &rhs;
        assert_eq!(result.as_slice(), &[5, 7, 9]);
    }

    #[test]
    fn add_assign() {
        let mut lhs = v![1, 2, 3];
        let rhs = v![4, 5, 6];

        lhs += rhs;
        assert_eq!(lhs.as_slice(), &[5, 7, 9]);
    }
}
