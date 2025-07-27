/// General [Vector] structure
///
/// A simple `vector` type, which does not attempt
/// to make assumptions about its purpose. In fact
/// internally it is no more than a simple `array`.
///
/// ## Requirements
///
/// `ValueType` must implement [std::marker::Copy] trait.
/// This is to support standard operator implementations
/// and uninitialized memory allocation.
/// Otherwise [Vector] does not impose other requirements
/// only those that are necessary for each trait implementation.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Vector<ValueType, const LENGTH: usize> {
    pub(crate) data: [ValueType; LENGTH],
}

impl<ValueType, const LENGTH: usize> Vector<ValueType, LENGTH>
where
    ValueType: Default + Copy,
{
    /// Create a new [Vector] filled with [Default](std::default::Default) of `ValueType`.
    ///
    /// Example
    /// ```
    /// # use lina::vector::Vector;
    /// let v1 : Vector<i32, 3> = Vector::new();
    /// // or
    /// let v2 = Vector::<i32, 3>::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }
}

impl<ValueType, const LENGTH: usize> Vector<ValueType, LENGTH>
where
    ValueType: Copy,
{
    /// Create a new [Vector] filled with `default_value`.
    ///
    /// Example
    /// ```
    /// # use lina::vector::Vector;
    /// let v1 : Vector<i32, 3> = Vector::from_value(3);
    /// // or
    /// let v2 = Vector::<i32, 3>::from_value(3);
    /// ```
    pub fn from_value(default_value: ValueType) -> Self {
        Self {
            data: [default_value; LENGTH],
        }
    }
}

impl<ValueType, const LENGTH: usize> Vector<ValueType, LENGTH> {
    /// Create a slice into the internal data
    pub fn as_slice(&self) -> &[ValueType] {
        &self.data
    }

    /// Construct a [Vector] from the given slice
    pub fn from_array(values: [ValueType; LENGTH]) -> Self {
        Self { data: values }
    }
}

impl<ValueType, const LENGTH: usize> PartialEq<[ValueType; LENGTH]> for Vector<ValueType, LENGTH>
where
    ValueType: PartialEq,
{
    fn eq(&self, other: &[ValueType; LENGTH]) -> bool {
        &self.data == other
    }
}

impl<ValueType, const LENGTH: usize> From<[ValueType; LENGTH]> for Vector<ValueType, LENGTH> {
    /// Create a [Vector] from an array
    ///
    /// ```
    /// # use lina::vector::Vector;
    /// let array = [1, 2, 3];
    /// let v = Vector::from(array);
    /// ```
    /// or using the generated [Into] trait.
    /// ```
    /// # use lina::vector::Vector;
    /// let array = [1,2,3];
    /// let v : Vector<i32, 3> = array.into();
    /// ```
    fn from(values: [ValueType; LENGTH]) -> Self {
        Vector::from_array(values)
    }
}

#[cfg(test)]
mod tests {
    use crate::v;

    use super::*;

    #[test]
    fn macro_init_empty_vector() {
        let vector: Vector<usize, 3> = v![];
        assert_eq!(vector.as_slice(), &[0, 0, 0]);
    }

    #[test]
    fn macro_init_with_default_value_and_count() {
        let vector = v![0; 5];
        assert_eq!(vector.as_slice(), &[0, 0, 0, 0, 0]);
    }

    #[test]
    fn macro_init_with_set_values() {
        let vector = v![1, 2, 3, 4];
        assert_eq!(vector.as_slice(), &[1, 2, 3, 4]);
    }

    #[test]
    fn eq() {
        let v1 = v![1, 2, 3];
        let v2 = v![1, 2, 3];
        assert!(v1 == v2);
    }
}
