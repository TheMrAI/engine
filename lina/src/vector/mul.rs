use std::mem;

use super::Vector;

impl<ValueType, const LENGTH: usize> std::ops::Mul<ValueType> for Vector<ValueType, LENGTH>
where
    ValueType: std::ops::Mul<Output = ValueType> + Copy,
{
    type Output = Vector<ValueType, LENGTH>;

    /// Performs the `Vector<T> * T` operation
    fn mul(self, rhs: ValueType) -> Self::Output {
        let mut data = [mem::MaybeUninit::<ValueType>::uninit(); LENGTH];

        for (elem, lhs) in data.iter_mut().zip(self.data.iter()) {
            elem.write(*lhs * rhs);
        }

        let ptr = &mut data as *mut _ as *mut [ValueType; LENGTH];
        let transmuted = unsafe { ptr.read() };

        Vector { data: transmuted }
    }
}

#[cfg(test)]
mod tests {
    use crate::v;

    #[test]
    fn scalar_mul() {
        let v = v![1, 2, 3];
        let result = v * 3;
        assert_eq!(result.as_slice(), [3, 6, 9]);
    }

    #[test]
    fn scalar_mul_lhs() {
        let v = v![1, 2, 3];
        let result = 3i32 * v;
        assert_eq!(result.as_slice(), [3, 6, 9]);
    }
}
