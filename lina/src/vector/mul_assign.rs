use super::Vector;
use std::ops::MulAssign;

impl<ValueType, const LENGTH: usize> MulAssign<ValueType> for Vector<ValueType, LENGTH>
where
    for<'a> &'a Vector<ValueType, LENGTH>:
        std::ops::Mul<ValueType, Output = Vector<ValueType, LENGTH>>,
{
    /// Implement the Vector<T> * T operation.
    fn mul_assign(&mut self, rhs: ValueType) {
        let result = &*self * rhs;
        *self = result;
    }
}

impl<ValueType, const LENGTH: usize> MulAssign<&ValueType> for Vector<ValueType, LENGTH>
where
    for<'a, 'b> &'a Vector<ValueType, LENGTH>:
        std::ops::Mul<&'b ValueType, Output = Vector<ValueType, LENGTH>>,
{
    /// Implement the Vector<T> * &T operation.
    fn mul_assign(&mut self, rhs: &ValueType) {
        let result = &*self * rhs;
        *self = result;
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

    #[test]
    fn scalar_ref_mul() {
        let s = &3;
        let mut v = v![1, 2, 3];
        v *= s;
        assert_eq!(v.as_slice(), [3, 6, 9]);
    }
}
