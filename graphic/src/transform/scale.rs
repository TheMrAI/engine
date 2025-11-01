use lina::{m, matrix::Matrix, vector::Vector};

// Generate S scaling matrix from the given scaling factors.
/// 
/// Affine, orthogonal.
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
/// Affine, orthogonal.
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
/// Affine, orthogonal.
/// 
/// [Vector] based wrapper for [scale].
#[rustfmt::skip]
pub fn scale_v(s: Vector<f32, 3>) -> Matrix<f32, 4, 4> {
    scale(s[0], s[1], s[2])
}

/// Generate the inverse of the [scale_v] matrix.
/// 
/// Affine, orthogonal.
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
