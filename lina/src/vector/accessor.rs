use super::vector::Vector;

/// Create a new [Vector] taking the `x`, `y`, `z` coordinates of the original.
///
/// The result is an [Option] in case where the original [Vector] is
/// shorter than 3.
impl<ValueType, const LENGTH: usize> Vector<ValueType, LENGTH>
where
    ValueType: std::ops::AddAssign<ValueType> + Copy,
{
    pub fn xyz(&self) -> Option<Vector<ValueType, 3>> {
        if LENGTH < 3 {
            None
        } else {
            let ptr = self.data.as_ptr() as *const [ValueType; 3];
            let data = unsafe { *ptr };
            Some(Vector::from_array(data))
        }
    }
}
