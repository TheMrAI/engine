use super::Vector;
use std::{mem, ops::Div};

impl<ValueType, const LENGTH: usize> Div<ValueType> for Vector<ValueType, LENGTH>
where
    ValueType: std::ops::Div<ValueType, Output = ValueType> + Copy,
{
    type Output = Vector<ValueType, LENGTH>;

    /// Implement `Vector<T> / T` operation.
    fn div(self, rhs: ValueType) -> Self::Output {
        let mut data = [mem::MaybeUninit::<ValueType>::uninit(); LENGTH];

        for (elem, lhs) in data.iter_mut().zip(self.data.iter()) {
            elem.write(*lhs / rhs);
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
    fn scalar_div() {
        let v = v![2, 4, 7];
        let result = v / 2;
        assert_eq!(result.as_slice(), [1, 2, 3]);
    }
}
