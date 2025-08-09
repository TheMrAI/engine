use crate::vector::Vector;

impl<ValueType, const LENGTH: usize> std::ops::Index<usize> for Vector<ValueType, LENGTH> {
    type Output = ValueType;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

#[cfg(test)]
mod tests {
    use crate::v;

    #[test]
    fn index() {
        let v = v![0, 1, 2];
        assert_eq!(v[0], 0);
        assert_eq!(v[1], 1);
        assert_eq!(v[2], 2);
    }
}
