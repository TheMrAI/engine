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
mod div;
mod div_assign;
mod from;
mod length;
mod mul;
mod mul_assign;
mod sub;
mod sub_assign;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Quaternion<ValueType> {
    scalar: ValueType,
    vector: Vector<ValueType, 3>,
}

impl<ValueType> Quaternion<ValueType>
where
    ValueType: Copy,
{
    /// For a quaternion `q = [s, v]`, query `s`.
    pub fn scalar(&self) -> ValueType {
        self.scalar
    }

    /// For a quaternion `q = [s, v]`, query `v`.
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
    ValueType: Copy + std::convert::From<i8>,
{
    /// Create a quaternion from a 3 element long [Vector].
    ///
    /// For a given [Vector] `v` a quaternion `p` will be created:
    /// ```text
    /// q = [0, v]
    /// ```
    ///
    /// ```
    /// # use quaternion::Quaternion;
    /// # use lina::v;
    /// let q = Quaternion::<i32>::from_vector(v![1, 2, 3]);
    ///
    /// assert_eq!(q.scalar(), 0);
    /// assert_eq!(q.vector().as_slice(), [1, 2, 3]);
    /// ```
    pub fn from_vector(v: Vector<ValueType, 3>) -> Quaternion<ValueType> {
        Quaternion {
            scalar: 0.into(),
            vector: v,
        }
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
    /// It will calculate the length/norm on the second power `n(q)^2`:
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

impl<ValueType> Quaternion<ValueType>
where
    ValueType: Copy
        + Default
        + std::ops::Add<Output = ValueType>
        + std::ops::Mul<Output = ValueType>
        + std::convert::From<i8>,
    Vector<ValueType, 3>: Copy + std::ops::Mul<ValueType, Output = Vector<ValueType, 3>>,
    Quaternion<ValueType>: std::ops::Div<ValueType, Output = Quaternion<ValueType>>
        + std::ops::Mul<Output = Quaternion<ValueType>>,
{
    /// Calculate the inverse of a quaternion.
    ///
    /// For a given quaternion `q`, calculate the inverse `q^-1`, such that:
    /// ```text
    /// q = [s, v]
    /// q*(q^-1) = (q^-1)*q = 1 = [1, 0]
    /// ```
    ///
    /// ```
    /// # use std::f32::consts::PI;
    /// # use quaternion::Quaternion;
    /// # use lina::v;
    /// # use float_eq::assert_float_eq;
    /// let q = Quaternion::<f32>::new_parts(2.39, v![1.0, 2.0, 3.0]);
    ///
    /// let a = q * q.inverse();
    ///  assert_float_eq!(a.scalar(), 1.0, ulps <= 1);
    /// a.vector().as_slice().iter().zip([0.0, 0.0, 0.0]).for_each(|(l, r)| assert_float_eq!(*l, r, ulps <= 1));
    ///
    /// // Commutative
    /// let b = q.inverse() * q;
    ///
    /// assert_float_eq!(a.scalar(), b.scalar(), ulps <= 1);
    /// a.vector().as_slice().iter().zip(b.vector().as_slice()).for_each(|(l, r)| assert_float_eq!(*l, *r, ulps <= 1));
    /// ```
    ///
    /// For a **unit** quaternion the inverse is equal to its conjugate:
    /// ```
    /// # use std::f32::consts::PI;
    /// # use quaternion::Quaternion;
    /// # use lina::v;
    /// # use float_eq::assert_float_eq;
    /// let q = Quaternion::<f32>::new_unit(PI/2.0, v![1.0, 2.0, 3.0]);
    ///
    /// let q_inverse = q.inverse();
    /// let q_conjugate = q.conjugate();
    ///
    /// assert_float_eq!(q_inverse.scalar(), q_conjugate.scalar(), ulps <= 1);
    /// q_inverse.vector().as_slice().iter().zip(q_conjugate.vector().as_slice()).for_each(|(l, r)| assert_float_eq!(*l, *r, ulps <= 1));
    /// ```
    pub fn inverse(&self) -> Quaternion<ValueType> {
        self.conjugate() / self.length_squared()
    }

    /// Produce a quaternion representing the "conjugate by" operation.
    ///
    /// For quaternion `q` and `p`:
    /// ```text
    /// q = [s, v]
    /// |q| = 1
    /// p = [s', v']
    /// ```
    /// it will perform operation:
    /// ```text
    /// qp(q^-1)
    /// ```
    ///
    /// Rotate `p` by `q`, while preserving it's length.
    ///
    /// `q` must be a **unit** quaternion or the operation is
    /// undefined.
    ///
    /// ```
    /// # use std::f32::consts::PI;
    /// # use quaternion::Quaternion;
    /// # use lina::v;
    /// # use float_eq::assert_float_eq;
    ///
    /// let p = Quaternion::<f32>::from_vector(v!{1.0, 0.0, 0.0});
    /// // A quaternion to rotate 90 degrees around the Y axis.
    /// // Notice the rotation axis isn't normalized.
    /// let q = Quaternion::<f32>::new_unit(PI/2.0, v![0.0, 1.0, 0.0]);
    ///
    /// let rotated_p = p.conjugate_by(q);
    ///
    /// assert_float_eq!(rotated_p.length(), 1.0, ulps <= 1);
    /// rotated_p.vector().as_slice().iter().zip([0.0, 0.0, -1.0]).for_each(|(l, r)| assert_float_eq!(*l, r, ulps <= 1));
    /// ```
    pub fn conjugate_by(self, q: Quaternion<ValueType>) -> Quaternion<ValueType> {
        q * self * q.inverse()
    }
}

macro_rules! impl_float_restricted_members {
    ($($T: ty),* $(,)*) => {$(
        impl Quaternion<$T> {
            /// Create a quaternion from a **tensor** and a **versor**.
            ///
            /// Given a quaternion `q` of the form:
            /// ```text
            /// q = B/A * (cos(theta) + v * sin(theta))
            /// ```
            /// the ratio `B/A` is called a **tensor**, while
            /// `cos(theta) + v * sin(theta)` is called a **rotor**.
            ///
            /// It would be very unwieldy to demand that **rotor** be provided
            /// precalculated by the user. This is why `theta` and `rotation_axis`
            /// are required to be provided separately.
            ///
            /// `theta` is rotation degrees in radians. The function internally
            /// divides this value by 2, ensuring that the resulting quaternion
            /// only rotates `theta` degrees.
            /// `rotation_axis` is internally normalized.
            pub fn new(tensor: $T, theta: $T, rotation_axis: Vector<$T, 3>) -> Quaternion<$T> {
                let theta = theta / 2.0;

                Quaternion {
                    scalar: tensor * theta.cos(),
                    vector: tensor * theta.sin() * rotation_axis,
                }
            }

            /// Create a **unit** quaternion.
            ///
            /// `theta` is rotation degrees in radians. The function internally
            /// divides this value by 2, ensuring that the resulting quaternion
            /// only rotates `theta` degrees.
            /// `rotation_axis` is internally normalized.
            ///
            /// ```
            /// # use std::f32::consts::PI;
            /// # use quaternion::Quaternion;
            /// # use lina::v;
            /// # use float_eq::assert_float_eq;
            /// let q = Quaternion::<f32>::new_unit(PI/3.0, v![1.0, 2.0, 3.0]);
            ///
            /// assert_float_eq!(q.length(), 1.0, ulps <= 1);
            /// ```
            pub fn new_unit(theta: $T, rotation_axis: Vector<$T, 3>) -> Quaternion<$T> {
                let theta = theta / 2.0;
                let normalized = rotation_axis.normalized();

                Quaternion {
                    scalar: theta.cos(),
                    vector: theta.sin() * normalized,
                }
            }
        }
    )*};
}

impl_float_restricted_members!(f32, f64);

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
