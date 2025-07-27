use super::Vector;
use std::ops::DivAssign;

impl<ValueType, const LENGTH: usize> DivAssign<ValueType> for Vector<ValueType, LENGTH>
where
    ValueType: std::ops::DivAssign<ValueType> + Copy,
{
    /// Implement Vector<T> /= T operation.
    fn div_assign(&mut self, rhs: ValueType) {
        for elem in self.data.iter_mut() {
            *elem /= rhs;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::v;

    #[test]
    fn scalar_div() {
        let mut v = v![2, 4, 7];
        v /= 2;
        assert_eq!(v.as_slice(), [1, 2, 3]);
    }
}
