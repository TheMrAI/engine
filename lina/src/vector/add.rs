use std::mem;

use super::vector::Vector;

impl<ValueType, const LENGTH: usize> std::ops::Add<Vector<ValueType, LENGTH>>
    for Vector<ValueType, LENGTH>
where
    ValueType: std::ops::Add<Output = ValueType> + Copy,
{
    type Output = Vector<ValueType, LENGTH>;

    /// Implement `Vector<T> + T` operation.
    fn add(self, rhs: Vector<ValueType, LENGTH>) -> Self::Output {
        let mut data = [mem::MaybeUninit::<ValueType>::uninit(); LENGTH];

        for (elem, (lhs, rhs)) in data.iter_mut().zip(self.data.iter().zip(rhs.data.iter())) {
            elem.write(*lhs + *rhs);
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
    fn add_by_value() {
        let lhs = v![1, 2, 3];
        let rhs = v![4, 5, 6];

        let result = lhs + rhs;
        assert_eq!(result.as_slice(), &[5, 7, 9]);
    }
}
