use crate::Vector;

impl<ValueType, const LENGTH: usize> Default for Vector<ValueType, LENGTH>
where
    ValueType: Default + Copy,
{
    fn default() -> Self {
        Self {
            data: [ValueType::default(); LENGTH],
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Vector;

    #[test]
    fn default() {
        let v_int = Vector::<i32, 5>::default();
        assert_eq!(v_int.as_slice(), &[0, 0, 0, 0, 0]);

        let v_float = Vector::<f32, 5>::default();
        // Comparing with zero value, should always work even with floats
        // as that is a cleanly representable value.
        assert_eq!(v_float.as_slice(), &[0.0, 0.0, 0.0, 0.0, 0.0]);
    }
}
