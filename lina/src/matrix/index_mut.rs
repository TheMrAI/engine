use crate::matrix::Matrix;

impl<ValueType, const ROW: usize, const COL: usize> std::ops::IndexMut<(usize, usize)>
    for Matrix<ValueType, ROW, COL>
{
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.data[index.0][index.1]
    }
}
