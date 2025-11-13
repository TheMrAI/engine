use lina::vector::Vector;

use crate::Quaternion;

impl<ValueType> Default for Quaternion<ValueType>
where
    ValueType: Default + Copy,
    ValueType: std::convert::From<i32>,
{
    fn default() -> Self {
        Self {
            scalar: ValueType::from(1),
            vector: Vector::from_value(ValueType::from(0)),
        }
    }
}

#[cfg(test)]
mod tests {
    use lina::vector::Vector;

    use crate::Quaternion;

    #[test]
    fn default() {
        let q = Quaternion::<i32>::default();
        assert_eq!(q.scalar(), 1);
        assert_eq!(q.vector(), Vector::from_value(0));
    }
}
