use lina::{m, matrix::Matrix};

use crate::Quaternion;

macro_rules! impl_into_matrix_for_float_types {
    ($($T: ty),* $(,)*) => {$(
        /// Convert the quaternion into a 4x4 transformation matrix.
        ///
        /// The resulting `Mq` transformation matrix implements the
        /// [conjugate_by](crate::Quaternion::conjugate_by) function in matrix form, enabling
        /// its integration into a chain of matrix transformations.
        ///
        /// Given a [Vector](lina::vector::Vector) `v` using homogeneous coordinates, it can be turned into
        /// a quaternion `p` by transferring only its **x**, **y** and **z**
        /// coordinates as the imaginary part and setting the real part to 0.
        /// Resulting in
        /// ```text
        /// p = 0 + ix + jy + kz
        /// ```
        /// Then using quaternion `q` describing the rotation the following
        /// two operations will be equivalent:
        /// ```text
        /// Mq * v = VR
        /// q * p * (q^-1) = PR
        /// ```
        /// where `VR` is the rotated 4D vector using homogeneous coordinates
        /// and `PR` is a quaternion containing the **x**, **y** and **z**
        /// components of `VR` as its imaginary part.
        ///
        /// ```
        /// use lina::v;
        /// use quaternion::Quaternion;
        /// use lina::matrix::Matrix;
        /// use std::f32::consts::PI;
        ///
        /// let v = v![1.0, 2.0, 3.0, 1.0];
        /// let p = Quaternion::from_vector(v.xyz().unwrap());
        /// let q = Quaternion::new_unit(PI / 2.0, v![1.0, 0.0, 0.0]);
        ///
        /// let mq: Matrix<f32, 4, 4> = q.into();
        ///
        /// let with_mq = mq * v;
        /// let with_conjugate = p.conjugate_by(q);
        ///
        /// //assert_eq!(with_mq.xyz().unwrap(), with_conjugate.vector());
        /// ```
        impl std::convert::Into<Matrix<$T, 4, 4>> for Quaternion<$T> {
            fn into(self) -> Matrix<$T, 4, 4> {
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
    )*};
}

impl_into_matrix_for_float_types!(f32, f64);
