use super::Vector;
use std::ops::MulAssign;

impl<ValueType, const LENGTH: usize> MulAssign<ValueType> for Vector<ValueType, LENGTH>
where
    ValueType: std::ops::MulAssign<ValueType> + Copy,
{
    /// Perform the `Vector<T> * T` operation
    fn mul_assign(&mut self, rhs: ValueType) {
        for elem in self.data.iter_mut() {
            *elem *= rhs;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::vector::v;

    #[test]
    fn scalar_mul() {
        let mut v = v![1, 2, 3];
        v *= 3;
        assert_eq!(v.as_slice(), [3, 6, 9]);
    }
}
