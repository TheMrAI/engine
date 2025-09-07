use crate::vector::Vector;

impl<ValueType, const LENGTH: usize> std::ops::IndexMut<usize> for Vector<ValueType, LENGTH> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

#[cfg(test)]
mod tests {
    use crate::v;

    #[test]
    fn index() {
        let mut v = v![0, 1, 2];
        v[0] = 1;
        v[2] = 1;
        assert_eq!(v[0], 1);
        assert_eq!(v[1], 1);
        assert_eq!(v[2], 1);
    }
}
