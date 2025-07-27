use super::Vector;

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
    use super::Vector;

    #[test]
    fn default() {
        let v_int = Vector::<i32, 5>::default();
        assert_eq!(v_int.as_slice(), &[0, 0, 0, 0, 0]);
    }
}
