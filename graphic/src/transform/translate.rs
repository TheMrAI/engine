use lina::{m, matrix::Matrix, vector::Vector};

/// Generate a T translation matrix given 3 scalars.
/// 
/// Move a point.
/// Affine, orthogonal.
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
/// Affine, orthogonal.
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
