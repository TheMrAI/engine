use crate::matrix::Matrix;

impl<ValueType, const ROW: usize, const COL: usize> std::ops::Index<(usize, usize)>
    for Matrix<ValueType, ROW, COL>
{
    type Output = ValueType;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.data[index.0][index.1]
    }
}
