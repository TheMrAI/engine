use lina::{m, matrix::Matrix, vector::Vector};

mod default;

///
/// A quaternion has the following form:
/// ```text
/// q = B/A(cos(Theta) + u*sin(Theta))
/// ```
/// where the scalar `B/A` is the tensor (scaling factor)
/// and `(cost(Theta) + u*sin(Theta))` is the versor (the rotation).
///
/// ```text
/// S(q) = B/A * cos(Theta)
/// V(q) = u * B/A * sin(Theta)
///
/// q = S(q) + V(q) = s + (ix + jy + kz) =
/// = [s, v]
/// ```
///
/// Some sources swap the `[s, v]` component order in their definitions `[v, s]`.
#[derive(Copy, Clone, Debug)]
pub struct Quaternion<ValueType> {
    scalar: ValueType,
    vector: Vector<ValueType, 3>,
}

impl<ValueType> Quaternion<ValueType>
where
    ValueType: Default + Copy,
{
    pub fn scalar(&self) -> ValueType {
        self.scalar
    }

    pub fn vector(&self) -> Vector<ValueType, 3> {
        self.vector
    }
}

impl Quaternion<f32> {
    pub fn new(tensor: f32, theta: f32, rotation_axis: Vector<f32, 3>) -> Quaternion<f32> {
        Quaternion {
            scalar: tensor * theta.cos(),
            vector: tensor * theta.sin() * rotation_axis,
        }
    }

    pub fn new_parts(scalar: f32, vector: Vector<f32, 3>) -> Quaternion<f32> {
        Quaternion { scalar, vector }
    }

    pub fn new_unit(theta: f32, rotation_axis: Vector<f32, 3>) -> Quaternion<f32> {
        // half theta internally so it is easier to comprehend
        let theta = theta / 2.0;

        Quaternion {
            scalar: theta.cos(),
            vector: theta.sin() * rotation_axis,
        }
    }

    pub fn length_squared(&self) -> f32 {
        self.vector
            .as_slice()
            .iter()
            .fold(self.scalar * self.scalar, |acc, value| {
                acc + (*value * *value)
            })
    }

    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn conjugate(&self) -> Quaternion<f32> {
        Quaternion {
            scalar: self.scalar,
            vector: -1.0 * self.vector,
        }
    }

    pub fn inverse(&self) -> Quaternion<f32> {
        self.conjugate() / self.length_squared()
    }

    /// The "conjugate by" operation for a quaternion.
    ///
    /// Rotate a quaternion by another quaternion and keep the length
    /// intact.
    /// Given `self` is a quaternion `p` and a quaternion `q` it implements
    /// the operation
    /// ```text
    /// qpq^-1
    /// ```
    pub fn conjugate_by(self, q: Quaternion<f32>) -> Quaternion<f32> {
        q * self * q.inverse()
    }
}

impl std::convert::Into<Matrix<f32, 4, 4>> for Quaternion<f32> {
    fn into(self) -> Matrix<f32, 4, 4> {
        let x = self.vector[0];
        let y = self.vector[1];
        let z = self.vector[2];
        let w = self.scalar;

        let v0_0 = w.powi(2) + x.powi(2) - y.powi(2) - z.powi(2);
        let v0_1 = 2.0 * x * y - 2.0 * w * z;
        let v0_2 = 2.0 * x * z + 2.0 * w * y;
        let v1_0 = 2.0 * x * y + 2.0 * w * z;
        let v1_1 = w.powi(2) - x.powi(2) + y.powi(2) - z.powi(2);
        let v1_2 = 2.0 * y * z - 2.0 * w * x;
        let v2_0 = 2.0 * x * z - 2.0 * w * y;
        let v2_1 = 2.0 * y * z + 2.0 * w * x;
        let v2_2 = w.powi(2) - x.powi(2) - y.powi(2) + z.powi(2);

        m!(
            [v0_0, v0_1, v0_2, 0.0],
            [v1_0, v1_1, v1_2, 0.0],
            [v2_0, v2_1, v2_2, 0.0],
            [0.0, 0.0, 0.0, 1.0]
        )
    }
}

impl std::ops::Mul<f32> for Quaternion<f32>
where
    f32: std::ops::Mul<Output = f32> + Copy,
{
    type Output = Quaternion<f32>;

    /// Performs the `Quaternion<T> * T` operation
    fn mul(self, rhs: f32) -> Self::Output {
        Quaternion::new_parts(self.scalar * rhs, self.vector * rhs)
    }
}

impl std::ops::Mul<Quaternion<f32>> for Quaternion<f32>
where
    f32: std::ops::Mul<Output = f32> + Copy,
{
    type Output = Quaternion<f32>;

    /// Performs the `Quaternion<T> * Quaternion<T>` operation
    fn mul(self, rhs: Quaternion<f32>) -> Self::Output {
        let scalar = (self.scalar * rhs.scalar) - (self.vector * rhs.vector);
        let vector =
            self.vector.cross(rhs.vector) + (self.scalar * rhs.vector) + (rhs.scalar * self.vector);

        Quaternion::new_parts(scalar, vector)
    }
}

impl std::ops::Div<f32> for Quaternion<f32>
where
    f32: std::ops::Div<f32, Output = f32> + Copy,
{
    type Output = Quaternion<f32>;

    /// Implement `Vector<T> / T` operation.
    fn div(self, rhs: f32) -> Self::Output {
        Quaternion::new_parts(self.scalar / rhs, self.vector / rhs)
    }
}
