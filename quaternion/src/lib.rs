//! # Quaternion
//!
//! Used for representing unique orientations in 3D space.
//!
//! A quaternion has the following form:
//! ```text
//! q = B/A(cos(Theta) + u*sin(Theta))
//! ```
//! where the scalar `B/A` is the tensor (scaling factor)
//! and `(cost(Theta) + u*sin(Theta))` is the versor (the rotation).
//!
//! ```text
//! S(q) = B/A * cos(Theta)
//! V(q) = u * B/A * sin(Theta)
//!
//! q = S(q) + V(q) = s + (ix + jy + kz) =
//! = [s, v]
//! ```
//!
//! Some sources swap the `[s, v]` component order in their definitions `[v, s]`.
//!
//! This form is useful for observing/understanding the transformation encoded
//! within the quaternion. But for defining operations on a quaternion a different
//! form is used.
//!
//! ```text
//! q = (s, V) = s + i * Vx + j * Vy + k * Vz
//! i^2 = j^2 = k^2 = -1
//! jk = -kj = i
//! ki = -ik = j
//! ij = -ji = k
//! ```
//!
//! Where `s` is the real and `V` is the imaginary part.
//!
//! Some good resources on quaternions:
//! - [Quaternion by Song Ho Ahn](https://www.songho.ca/math/quaternion/quaternion.html)
//! - [Real Time Rendering, quaternion chapter](https://www.realtimerendering.com/)

use std::ops::{Add, Mul};

use lina::vector::Vector;

mod add;
mod add_assign;
mod conjugate;
mod default;
mod from;
mod length;
mod mul;
mod mul_assign;

#[derive(Copy, Clone, Debug, PartialEq)]
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

    /// Construct a quaternion by supplying the scalar and vector parts directly.
    ///
    /// Given a quaternion q:
    /// ```text
    /// q = (s, v) = s + ix + jy + kz
    /// ```
    ///
    /// Where `s` is the scalar and `v = [x, y, z]` vector.
    pub fn new_parts(scalar: ValueType, vector: Vector<ValueType, 3>) -> Quaternion<ValueType> {
        Quaternion { scalar, vector }
    }
}

impl<ValueType> Quaternion<ValueType>
where
    ValueType: Default + Copy + Add<Output = ValueType> + Mul<Output = ValueType>,
{
    /// Calculate the second power of the length/norm.
    ///
    /// For a given quaternion q:
    /// ```text
    /// q = s + ix + jy + kz
    /// ```
    /// It will calculate the length/norm `n(q)^2`:
    /// ```text
    /// n(q)^2 = s^2 + x^2 + y^2 + z^2
    /// ```
    ///
    /// Instead of having to call [Quaternion::length] and raising it to the second
    /// power the function calculates the value directly.
    pub fn length_squared(&self) -> ValueType {
        self.vector
            .as_slice()
            .iter()
            .fold(self.scalar * self.scalar, |acc, value| {
                acc + (*value * *value)
            })
    }
}

impl Quaternion<f32> {
    pub fn new(tensor: f32, theta: f32, rotation_axis: Vector<f32, 3>) -> Quaternion<f32> {
        let theta = theta / 2.0;

        Quaternion {
            scalar: tensor * theta.cos(),
            vector: tensor * theta.sin() * rotation_axis,
        }
    }

    pub fn new_unit(theta: f32, rotation_axis: Vector<f32, 3>) -> Quaternion<f32> {
        // half theta internally so it is easier to comprehend
        let theta = theta / 2.0;

        Quaternion {
            scalar: theta.cos(),
            vector: theta.sin() * rotation_axis,
        }
    }

    pub fn from_vector(v: Vector<f32, 3>) -> Quaternion<f32> {
        Quaternion {
            scalar: 0.0,
            vector: v,
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

#[cfg(test)]
mod tests {
    use lina::v;

    use crate::Quaternion;

    #[test]
    fn length_squared() {
        let q = Quaternion::new_parts(1, v![2, 3, 4]);
        assert_eq!(q.length_squared(), 30);
    }
}
