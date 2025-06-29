#[derive(Debug, PartialEq)]
pub struct Vector<ValueType, const LENGTH: usize> {
    pub(crate) data: [ValueType; LENGTH],
}

impl<ValueType, const LENGTH: usize> Clone for Vector<ValueType, LENGTH>
where
    ValueType: Clone,
{
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl<ValueType, const LENGTH: usize> Vector<ValueType, LENGTH>
where
    ValueType: Default + Copy,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn as_slice(&self) -> &[ValueType] {
        &self.data
    }

    pub fn from_value(default_value: ValueType) -> Self {
        Self {
            data: [default_value; LENGTH],
        }
    }

    pub fn from_array(values: [ValueType; LENGTH]) -> Self {
        Self { data: values }
    }
}

#[macro_export]
macro_rules! v {
    () => {
        $crate::vector::vector::Vector::new()
    };
    ($default_value:expr; $n:expr) => {
        $crate::vector::vector::Vector::<_, $n>::from_value($default_value)
    };
    ($($element:expr),+$(,)?) => {
        $crate::vector::vector::Vector::from_array([$($element),+])
    }
}

#[cfg(test)]
mod tests {
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
}
