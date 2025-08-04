use crate::{v, vector::sqrt::Sqrt};

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
    pub fn from_array(values: [ValueType; LENGTH]) -> Vector<ValueType, LENGTH> {
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

impl<ValueType, const LENGTH: usize> Vector<ValueType, LENGTH>
where
    ValueType: Copy
        + std::ops::Mul<ValueType, Output = ValueType>
        + std::ops::Div<ValueType, Output = ValueType>
        + std::iter::Sum
        + Sqrt<Output = ValueType>,
{
    /// Normalize the current vector in-place
    ///
    /// A convenience function for writing inline calculations.
    /// ```
    /// # use lina::vector::Vector;
    /// let v = v![4.0, 4.0, 4.0].norm() * 2.0;
    /// ```
    pub fn norm(&mut self) -> &mut Vector<ValueType, LENGTH> {
        let length = self.length();
        for value in self.data.iter_mut() {
            *value = *value / length;
        }
        self
    }

    /// Generate a normal vector without modifying the current one
    pub fn normalized(&self) -> Vector<ValueType, LENGTH> {
        let mut vector = *self;
        *vector.norm()
    }

    /// Length of the vector
    pub fn length(&self) -> ValueType {
        self.length_squared().square_root()
    }

    /// Squared length of the vector
    ///
    /// A convenience function when the squared length of the vector is
    /// required. Avoiding unnecessary multiplication operation.
    pub fn length_squared(&self) -> ValueType {
        self.data.iter().map(|value| *value * *value).sum()
    }
}

/// Cross product for a Vector<f32, 3>
///
/// As far as I could see a cross product only exists in 3 and 7 dimensions,
/// but for now support for only three is enough.
impl Vector<f32, 3> {
    pub fn cross(self, rhs: Vector<f32, 3>) -> Vector<f32, 3> {
        v![
            self.data[1] * rhs.data[2] - self.data[2] * rhs.data[1],
            self.data[2] * rhs.data[0] - self.data[0] * rhs.data[2],
            self.data[0] * rhs.data[1] - self.data[1] * rhs.data[0]
        ]
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
