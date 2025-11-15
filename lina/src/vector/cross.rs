use crate::vector::Vector;

impl<ValueType> Vector<ValueType, 3>
where
    ValueType: Copy + std::ops::Mul<Output = ValueType> + std::ops::Sub<Output = ValueType>,
{
    /// Cross product for 3D Vectors
    ///
    /// As far as I could see a cross product only exists in 3 and 7 dimensions,
    /// but for now support for only 3 is enough.
    pub fn cross(self, rhs: Vector<ValueType, 3>) -> Vector<ValueType, 3> {
        use crate::v;
        v![
            self.data[1] * rhs.data[2] - self.data[2] * rhs.data[1],
            self.data[2] * rhs.data[0] - self.data[0] * rhs.data[2],
            self.data[0] * rhs.data[1] - self.data[1] * rhs.data[0]
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::{v, vector::Vector};

    #[test]
    fn cross_on_basis_vectors() {
        let x: Vector<f32, 3> = v![1.0, 0.0, 0.0];
        let y: Vector<f32, 3> = v![0.0, 1.0, 0.0];
        let z: Vector<f32, 3> = v![0.0, 0.0, 1.0];

        assert_eq!(x.cross(y), z);
        assert_eq!(y.cross(x), -z);
        assert_eq!(y.cross(z), x);
        assert_eq!(z.cross(y), -x);
        assert_eq!(z.cross(x), y);
        assert_eq!(x.cross(z), -y);
    }
}
