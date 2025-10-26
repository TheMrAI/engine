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
#[rustfmt::skip]
pub fn translate(translate_x: f32, translate_y: f32, translate_z: f32) -> Matrix<f32, 4, 4> {
    m![
        [1.0, 0.0, 0.0, translate_x],
        [0.0, 1.0, 0.0, translate_y],
        [0.0, 0.0, 1.0, translate_z],
        [0.0, 0.0, 0.0, 1.0]
    ]
}

/// Generate inverse of T translation matrix given the 3 scalars.
///
/// Move a point in the inverse of T.
/// Affine.
///
/// # Example:
/// ```
/// use graphic::translate;
/// use graphic::inverse_translate;
/// use graphic::identity_matrix;
///
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
/// Move a point.
/// Affine. 
#[rustfmt::skip]
pub fn translate_v(t: &Vector<f32, 3>) -> Matrix<f32, 4, 4> {
    translate(t[0], t[1], t[2])
}

/// Generate inverse of T translation matrix given the 3 scalars.
///
/// Move a point in the inverse of T.
/// Affine.
///
/// # Example:
/// ```
/// use graphic::translate;
/// use graphic::inverse_translate;
/// use graphic::identity_matrix;
///
/// let T = translate(1.0, 2.0, 3.0);
/// let T_inv = inverse_translate(1.0, 2.0, 3.0);
///
/// let identity = identity_matrix();
///
/// assert_eq!(T * T_inv, identity);
/// ```
pub fn inverse_translate_v(t_inv: &Vector<f32, 3>) -> Matrix<f32, 4, 4> {
    inverse_translate(t_inv[0], t_inv[1], t_inv[2])
}
