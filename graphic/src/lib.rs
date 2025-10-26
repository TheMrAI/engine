//! Provide all common functionality necessary for basic
//! rendering operations. Building basic transformation
//! matrices and transforming/mapping memory to GPU format.
//!
//! # Coordinate system
//!
//! ```text
//!         Y
//!         ^
//!         |
//!         |
//!         x -----> X
//!        /
//!       /
//!      v
//!     Z
//! ```
//!
//! This library uses the right handed, Y vector up convention for the coordinate system and
//! all matrices are defined in column-major form.
//!
//! In case a row-major format is necessary, then that can be achieved by simply transposing
//! each matrix.
//!
//! # Point versus vector
//!
//! To represent points and vectors, we use 4D vectors with the homogeneous notation.
//!
//! A point v: `v = (vx, vy, vz, 0)`
//! A vector (direction) v: `v = (vx, vy, vz, 1)`
//!
//! Notice the 4th element is a 0 for a point and 1 for a vector. In principle a point is
//! an end point of a vector, but without any directional properties. A point has no direction
//! , but a vector does. A vector points from the origo in the direction of the point.
//! That 0 and 1 represent this distinction mathematically. Due to the nature of linear
//! transformation this is all we need to properly apply transformation to points and vectors
//! alike with the same matrices.
//!
//! # Transformation matrix inversion
//!
//! Whenever possible all transformation matrices will be accompanied by their inverse matrix.
//! This is for two reasons. Calculating an inversion matrix isn't necessarily cheap nor precise.
//! But more importantly it is entirely unnecessary. As long as the developer is aware that a chain
//! of matrix transformation will be inverted in the end, they can simply generate the inverted matrix
//! equivalent transformation right from the start.

use lina::{m, matrix::Matrix, v, vector::Vector};
pub mod camera;
pub mod transform;

#[rustfmt::skip]
pub fn rotate_x(rad_angle: f32) -> Matrix<f32, 4, 4> {
    let cosine = rad_angle.cos();
    let sine = rad_angle.sin();
    m![
        [1.0, 0.0,    0.0,    0.0],
        [0.0, cosine, -sine,  0.0], 
        [0.0, sine,   cosine, 0.0],
        [0.0, 0.0,    0.0,    1.0]    
    ]
}

#[rustfmt::skip]
pub fn rotate_y(rad_angle: f32) -> Matrix<f32, 4, 4> {
    let cosine = rad_angle.cos();
    let sine = rad_angle.sin();
    m![ 
        [cosine, 0.0,  sine,    0.0],
        [0.0,    1.0,  0.0,     0.0], 
        [-sine,  0.0,  cosine,  0.0],
        [0.0,    0.0,  0.0,     1.0] 
    ]
}

#[rustfmt::skip]
pub fn rotate_z(rad_angle: f32) -> Matrix<f32, 4, 4> {
    let cosine = rad_angle.cos();
    let sine = rad_angle.sin();
    m![
         [cosine, -sine,  0.0, 0.0],
         [sine,   cosine, 0.0, 0.0],
         [0.0,    0.0,    1.0, 0.0], 
         [0.0,    0.0,    0.0, 1.0] 
    ]
}

#[rustfmt::skip]
pub fn scale(scale_x: f32, scale_y: f32, scale_z: f32) -> Matrix<f32, 4, 4> {
    m![
        [scale_x, 0.0,     0.0,     0.0],
        [0.0,     scale_y, 0.0,     0.0],
        [0.0,     0.0,     scale_z, 0.0],
        [0.0,     0.0,     0.0,     1.0]
    ]
}

#[rustfmt::skip]
pub fn identity_matrix() -> Matrix<f32, 4, 4> {
    m![
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0]
    ]
}

#[rustfmt::skip]
pub fn orthographic_projection(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    z_near: f32,
    z_far: f32,
) -> Matrix<f32, 4, 4> {
    m![
        [2.0/(right - left), 0.0,                0.0,                  (right + left)  /(left - right)],
        [0.0,                2.0/(top - bottom), 0.0,                  (top + bottom) / (bottom - top)],
        [0.0,                0.0,                1.0/(z_near - z_far), z_near / (z_near - z_far)],
        [0.0,                0.0,                0.0,                  1.0]
    ]
}

#[rustfmt::skip]
pub fn perspective_projection(fov_rad: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> Matrix<f32, 4, 4> {
    let f = (std::f32::consts::PI * 0.5 - 0.5 * fov_rad).tan();
    let range_inverse = 1.0 / (z_near - z_far);

    m![
        [f / aspect_ratio, 0.0, 0.0,                    0.0],
        [0.0,              f,   0.0,                    0.0],
        [0.0,              0.0, z_far * range_inverse,  z_near * z_far * range_inverse],
        [0.0,              0.0, -1.0,                   0.0]
    ]
}

/// Convenience function for generating cross product for 4D vectors
///
/// As the cross product doesn't exist for 4D vectors, this function takes the first
/// three elements from each and calculates the cross product on those.
/// Finally padding it out with a zero and returning a 4D vector.
pub fn cross(lhs: Vector<f32, 4>, rhs: Vector<f32, 4>) -> Vector<f32, 4> {
    let lhs_3 = Vector::from_array([lhs[0], lhs[1], lhs[2]]);
    let rhs_3 = Vector::from_array([rhs[0], rhs[1], rhs[2]]);

    let cross_product = lhs_3.cross(rhs_3);
    v![cross_product[0], cross_product[1], cross_product[2], 0.0]
}

#[rustfmt::skip]
pub fn point_at(
    position: Vector<f32, 3>,
    target: Vector<f32, 3>,
    up: Vector<f32, 3>,
) -> Matrix<f32, 4, 4> {
    // forward, Z axis
    let forward = (position - target).normalized();
    // right, X axis
    let right = up.cross(forward).normalized();
    // up - Y axis
    let up = forward.cross(right).normalized();

    m![
        [right[0], up[0], forward[0], position[0]],
        [right[1], up[1], forward[1], position[1]],
        [right[2], up[2], forward[2], position[2]],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

pub fn look_at(
    position: Vector<f32, 3>,
    target: Vector<f32, 3>,
    up: Vector<f32, 3>,
) -> Matrix<f32, 4, 4> {
    inverse(point_at(position, target, up))
}

pub fn inverse(a: Matrix<f32, 4, 4>) -> Matrix<f32, 4, 4> {
    let mut inverse = Matrix::<f32, 4, 4>::from_value(0.0);

    let m00 = a[(0, 0)];
    let m01 = a[(0, 1)];
    let m02 = a[(0, 2)];
    let m03 = a[(0, 3)];
    let m10 = a[(1, 0)];
    let m11 = a[(1, 1)];
    let m12 = a[(1, 2)];
    let m13 = a[(1, 3)];
    let m20 = a[(2, 0)];
    let m21 = a[(2, 1)];
    let m22 = a[(2, 2)];
    let m23 = a[(2, 3)];
    let m30 = a[(3, 0)];
    let m31 = a[(3, 1)];
    let m32 = a[(3, 2)];
    let m33 = a[(3, 3)];

    let tmp0 = m22 * m33;
    let tmp1 = m32 * m23;
    let tmp2 = m12 * m33;
    let tmp3 = m32 * m13;
    let tmp4 = m12 * m23;
    let tmp5 = m22 * m13;
    let tmp6 = m02 * m33;
    let tmp7 = m32 * m03;
    let tmp8 = m02 * m23;
    let tmp9 = m22 * m03;
    let tmp10 = m02 * m13;
    let tmp11 = m12 * m03;
    let tmp12 = m20 * m31;
    let tmp13 = m30 * m21;
    let tmp14 = m10 * m31;
    let tmp15 = m30 * m11;
    let tmp16 = m10 * m21;
    let tmp17 = m20 * m11;
    let tmp18 = m00 * m31;
    let tmp19 = m30 * m01;
    let tmp20 = m00 * m21;
    let tmp21 = m20 * m01;
    let tmp22 = m00 * m11;
    let tmp23 = m10 * m01;

    let t0 = (tmp0 * m11 + tmp3 * m21 + tmp4 * m31) - (tmp1 * m11 + tmp2 * m21 + tmp5 * m31);
    let t1 = (tmp1 * m01 + tmp6 * m21 + tmp9 * m31) - (tmp0 * m01 + tmp7 * m21 + tmp8 * m31);
    let t2 = (tmp2 * m01 + tmp7 * m11 + tmp10 * m31) - (tmp3 * m01 + tmp6 * m11 + tmp11 * m31);
    let t3 = (tmp5 * m01 + tmp8 * m11 + tmp11 * m21) - (tmp4 * m01 + tmp9 * m11 + tmp10 * m21);

    let d = 1.0 / (m00 * t0 + m10 * t1 + m20 * t2 + m30 * t3);

    inverse[(0, 0)] = d * t0;
    inverse[(0, 1)] = d * t1;
    inverse[(0, 2)] = d * t2;
    inverse[(0, 3)] = d * t3;

    inverse[(1, 0)] =
        d * ((tmp1 * m10 + tmp2 * m20 + tmp5 * m30) - (tmp0 * m10 + tmp3 * m20 + tmp4 * m30));
    inverse[(1, 1)] =
        d * ((tmp0 * m00 + tmp7 * m20 + tmp8 * m30) - (tmp1 * m00 + tmp6 * m20 + tmp9 * m30));
    inverse[(1, 2)] =
        d * ((tmp3 * m00 + tmp6 * m10 + tmp11 * m30) - (tmp2 * m00 + tmp7 * m10 + tmp10 * m30));
    inverse[(1, 3)] =
        d * ((tmp4 * m00 + tmp9 * m10 + tmp10 * m20) - (tmp5 * m00 + tmp8 * m10 + tmp11 * m20));

    inverse[(2, 0)] =
        d * ((tmp12 * m13 + tmp15 * m23 + tmp16 * m33) - (tmp13 * m13 + tmp14 * m23 + tmp17 * m33));
    inverse[(2, 1)] =
        d * ((tmp13 * m03 + tmp18 * m23 + tmp21 * m33) - (tmp12 * m03 + tmp19 * m23 + tmp20 * m33));
    inverse[(2, 2)] =
        d * ((tmp14 * m03 + tmp19 * m13 + tmp22 * m33) - (tmp15 * m03 + tmp18 * m13 + tmp23 * m33));
    inverse[(2, 3)] =
        d * ((tmp17 * m03 + tmp20 * m13 + tmp23 * m23) - (tmp16 * m03 + tmp21 * m13 + tmp22 * m23));

    inverse[(3, 0)] =
        d * ((tmp14 * m22 + tmp17 * m32 + tmp13 * m12) - (tmp16 * m32 + tmp12 * m12 + tmp15 * m22));
    inverse[(3, 1)] =
        d * ((tmp20 * m32 + tmp12 * m02 + tmp19 * m22) - (tmp18 * m22 + tmp21 * m32 + tmp13 * m02));
    inverse[(3, 2)] =
        d * ((tmp18 * m12 + tmp23 * m32 + tmp15 * m02) - (tmp22 * m32 + tmp14 * m02 + tmp19 * m12));
    inverse[(3, 3)] =
        d * ((tmp22 * m22 + tmp16 * m02 + tmp21 * m12) - (tmp20 * m12 + tmp23 * m22 + tmp17 * m02));

    inverse
}
