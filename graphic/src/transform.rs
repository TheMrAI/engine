//! Basic transformation matrices.
//!
//! # Definitions
//!
//! To properly work with the transformation matrices, it is
//! vital to understand their properties. For which all relevant
//! definitions will be listed below.
//!
//! ## Linear transformation
//!
//! A linear transform is one that preserves vector addition and scalar
//! multiplication properties.
//!
//! Example:
//! ```text
//! f(x) + f(y) = f(x + y),
//! kf(x) = f(kx),
//! ```
//! where **f** is the transformation function (in practice the transformation matrix)
//! **x**, **y** the vectors to be transformed and **k** a scalar.
//!  
//! ## Affine transformation
//!
//! A transformation that performs a linear transformation and then
//! a translation.
//! It preserves the parallelism of lines but not necessarily the lengths
//! or the angles.
//! Can be constructed as a sequence of concatenations of individual affine
//! transforms.
//!
//! All translation, rotation, scaling, reflection and shearing matrices are affine.

use lina::{m, matrix::Matrix, vector::Vector};

/// Generate a T translation matrix given 3 scalars.
/// 
/// Move a point.
/// Affine.
/// Preserves handedness.
#[rustfmt::skip]
pub fn translate(translate_x: f32, translate_y: f32, translate_z: f32) -> Matrix<f32, 4, 4> {
    m![
        [1.0, 0.0, 0.0, translate_x],
        [0.0, 1.0, 0.0, translate_y],
        [0.0, 0.0, 1.0, translate_z],
        [0.0, 0.0, 0.0, 1.0]
    ]
}

/// Generate inverse of T translation matrix given 3 scalars.
///
/// Move a point in the inverse of T.
/// Affine.
/// Preserves handedness.
///
/// # Example:
/// ```
/// # use graphic::transform::translate;
/// # use graphic::transform::inverse_translate;
/// # use graphic::identity_matrix;
/// let T = translate(1.0, 2.0, 3.0);
/// let T_inv = inverse_translate(1.0, 2.0, 3.0);
///
/// let identity = identity_matrix();
///
/// assert_eq!(T * T_inv, identity);
/// ```
pub fn inverse_translate(
    translate_x: f32,
    translate_y: f32,
    translate_z: f32,
) -> Matrix<f32, 4, 4> {
    translate(-translate_x, -translate_y, -translate_z)
}

/// Generate a translation matrix by a given `t` [Vector].
/// 
/// Vector based wrapper for [translate].
#[rustfmt::skip]
pub fn translate_v(t: &Vector<f32, 3>) -> Matrix<f32, 4, 4> {
    translate(t[0], t[1], t[2])
}

/// Generate inverse of T translation matrix by a given `t` [Vector].
///
/// Vector based wrapper for [inverse_translate].
///
/// # Example:
/// ```
/// # use graphic::transform::translate_v;
/// # use graphic::transform::inverse_translate_v;
/// # use graphic::identity_matrix;
/// # use lina::v;
/// let v = v![1.0, 2.0, 3.0];
/// let T = translate_v(&v);
/// let T_inv = inverse_translate_v(&v);
///
/// let identity = identity_matrix();
///
/// assert_eq!(T * T_inv, identity);
/// ```
pub fn inverse_translate_v(t_inv: &Vector<f32, 3>) -> Matrix<f32, 4, 4> {
    inverse_translate(t_inv[0], t_inv[1], t_inv[2])
}

/// Generate counter-clockwise R rotation matrix by the given radians around the X axis.
/// 
/// Affine.
/// 
/// Prone to "Gimbal lock", if used with other matrix rotations.
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

/// Generate the inverse rotation of [rotate_x].
/// Rotate clockwise by the given radians around the X axis.
///
/// Affine.
///
/// # Example:
/// ```
/// # use std::f32::consts::PI;
/// # use graphic::transform::rotate_x;
/// # use graphic::transform::inverse_rotate_x;
/// # use graphic::identity_matrix;
/// let radians = PI/2.0;
/// let Rx = rotate_x(radians);
/// let Rx_inv = inverse_rotate_x(radians);
///
/// let identity = identity_matrix();
///
/// assert_eq!(Rx * Rx_inv, identity);
/// ```
pub fn inverse_rotate_x(rad_angle: f32) -> Matrix<f32, 4, 4> {
    rotate_x(-rad_angle)
}

/// Generate counter-clockwise R rotation matrix by the given radians around the Y axis.
/// 
/// Affine.
/// 
/// Prone to "Gimbal lock", if used with other matrix rotations. 
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

/// Generate the inverse rotation of [rotate_y].
/// Rotate clockwise by the given radians around the Y axis.
///
/// Affine.
///
/// # Example:
/// ```
/// # use std::f32::consts::PI;
/// # use graphic::transform::rotate_y;
/// # use graphic::transform::inverse_rotate_y;
/// # use graphic::identity_matrix;
/// let radians = PI/2.0;
/// let Rx = rotate_y(radians);
/// let Rx_inv = inverse_rotate_y(radians);
///
/// let identity = identity_matrix();
///
/// assert_eq!(Rx * Rx_inv, identity);
/// ```
pub fn inverse_rotate_y(rad_angle: f32) -> Matrix<f32, 4, 4> {
    rotate_y(-rad_angle)
}

/// Generate counter-clockwise R rotation matrix by the given radians around the Z axis.
/// 
/// Affine.
/// 
/// Prone to "Gimbal lock", if used with other matrix rotations.
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

/// Generate the inverse rotation of [rotate_z].
/// Rotate clockwise by the given radians around the Z axis.
///
/// Affine.
///
/// # Example:
/// ```
/// # use std::f32::consts::PI;
/// # use graphic::transform::rotate_z;
/// # use graphic::transform::inverse_rotate_z;
/// # use graphic::identity_matrix;
/// let radians = PI/2.0;
/// let Rx = rotate_z(radians);
/// let Rx_inv = inverse_rotate_z(radians);
///
/// let identity = identity_matrix();
///
/// assert_eq!(Rx * Rx_inv, identity);
/// ```
pub fn inverse_rotate_z(rad_angle: f32) -> Matrix<f32, 4, 4> {
    rotate_z(-rad_angle)
}

/// Generate S scaling matrix from the given scaling factors.
/// 
/// Affine.
/// 
/// A negative scaling factor for one or more scaling factors will
/// result in a `reflection` or `mirror` matrix. Depending on the use
/// this may have to be handled appropriately. For example
/// reflecting a triangle, may invert it's vertex order, resulting
/// in incorrect rendering. 
#[rustfmt::skip]
pub fn scale(scale_x: f32, scale_y: f32, scale_z: f32) -> Matrix<f32, 4, 4> {
    m![
        [scale_x, 0.0,     0.0,     0.0],
        [0.0,     scale_y, 0.0,     0.0],
        [0.0,     0.0,     scale_z, 0.0],
        [0.0,     0.0,     0.0,     1.0]
    ]
}

/// Generate the inverse of the [scale] matrix.
/// 
/// Affine.
/// 
/// # Panics
/// 
/// If any scalar is zero, or very close to zero.
/// 
/// # Example
/// ```
/// # use std::f32::consts::PI;
/// # use graphic::transform::scale;
/// # use graphic::transform::inverse_scale;
/// # use graphic::identity_matrix;
/// let radians = PI/2.0;
/// let S = scale(1.0, 2.0, 3.0);
/// let S_inv = inverse_scale(1.0, 2.0, 3.0);
///
/// let identity = identity_matrix();
///
/// assert_eq!(S * S_inv, identity);
/// ```
#[rustfmt::skip]
pub fn inverse_scale(scale_x: f32, scale_y: f32, scale_z: f32) -> Matrix<f32, 4, 4> {
    scale(1.0/scale_x, 1.0/scale_y, 1.0/scale_z)
}

/// Generate S scaling matrix from the given scaling [Vector].
/// 
/// Affine.
/// 
/// [Vector] based wrapper for [scale].
#[rustfmt::skip]
pub fn scale_v(s: Vector<f32, 3>) -> Matrix<f32, 4, 4> {
    scale(s[0], s[1], s[2])
}

/// Generate the inverse of the [scale_v] matrix.
/// 
/// Affine.
///
/// [Vector] based wrapper for [inverse_scale].
/// 
/// # Panics
/// 
/// If any scalar is zero, or very close to zero.
/// 
/// # Example
/// ```
/// # use std::f32::consts::PI;
/// # use graphic::transform::scale_v;
/// # use graphic::transform::inverse_scale_v;
/// # use graphic::identity_matrix;
/// # use lina::v;
/// let v = v![1.0, 2.0, 3.0];
/// let S = scale_v(v);
/// let S_inv = inverse_scale_v(v);
///
/// let identity = identity_matrix();
///
/// assert_eq!(S * S_inv, identity);
/// ```
#[rustfmt::skip]
pub fn inverse_scale_v(s: Vector<f32, 3>) -> Matrix<f32, 4, 4> {
    inverse_scale(s[0], s[1], s[2])
}
