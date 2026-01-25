use lina::vector::Vector;

use crate::Quaternion;

impl<ValueType> Quaternion<ValueType>
where
    Vector<ValueType, 3>: Copy + std::ops::Mul<ValueType, Output = Vector<ValueType, 3>>,
    ValueType: Copy + std::convert::From<i8>,
{
    /// Generate the conjugate.
    ///
    /// Given a quaternion `q`:
    /// ```text
    /// q = [s, v]
    /// ```
    /// the conjugate will be:
    /// ```text
    /// q* = [s, -v]
    /// ```
    pub fn conjugate(&self) -> Quaternion<ValueType> {
        Quaternion {
            scalar: self.scalar,
            vector: self.vector * ValueType::from(-1),
        }
    }
}

#[cfg(test)]
mod tests {
    use lina::v;

    use crate::Quaternion;

    #[test]
    fn conjugate() {
        let q = Quaternion::new_parts(1, v![2, 3, 4]);
        let q_conjugate = q.conjugate();

        assert_eq!(q_conjugate.scalar(), 1);
        assert_eq!(q_conjugate.vector().as_slice(), [-2, -3, -4]);
        assert_eq!(q_conjugate.conjugate(), q);
    }
}
