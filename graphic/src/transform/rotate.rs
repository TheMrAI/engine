use lina::{m, matrix::Matrix};

/// Generate counter-clockwise R rotation matrix by the given radians around the X axis.
/// 
/// Affine, orthogonal.
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
/// Affine, orthogonal.
///
/// # Example:
/// ```
/// # use std::f32::consts::PI;
/// # use graphic::transform::rotate_x;
/// # use graphic::transform::inv_rotate_x;
/// # use graphic::identity_matrix;
/// let radians = PI/2.0;
/// let Rx = rotate_x(radians);
/// let Rx_inv = inv_rotate_x(radians);
///
/// let identity = identity_matrix();
///
/// assert_eq!(Rx * Rx_inv, identity);
/// ```
pub fn inv_rotate_x(rad_angle: f32) -> Matrix<f32, 4, 4> {
    rotate_x(-rad_angle)
}

/// Generate counter-clockwise R rotation matrix by the given radians around the Y axis.
/// 
/// Affine, orthogonal.
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
/// Affine, orthogonal.
///
/// # Example:
/// ```
/// # use std::f32::consts::PI;
/// # use graphic::transform::rotate_y;
/// # use graphic::transform::inv_rotate_y;
/// # use graphic::identity_matrix;
/// let radians = PI/2.0;
/// let Rx = rotate_y(radians);
/// let Rx_inv = inv_rotate_y(radians);
///
/// let identity = identity_matrix();
///
/// assert_eq!(Rx * Rx_inv, identity);
/// ```
pub fn inv_rotate_y(rad_angle: f32) -> Matrix<f32, 4, 4> {
    rotate_y(-rad_angle)
}

/// Generate counter-clockwise R rotation matrix by the given radians around the Z axis.
/// 
/// Affine, orthogonal.
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
/// Affine, orthogonal.
///
/// # Example:
/// ```
/// # use std::f32::consts::PI;
/// # use graphic::transform::rotate_z;
/// # use graphic::transform::inv_rotate_z;
/// # use graphic::identity_matrix;
/// let radians = PI/2.0;
/// let Rx = rotate_z(radians);
/// let Rx_inv = inv_rotate_z(radians);
///
/// let identity = identity_matrix();
///
/// assert_eq!(Rx * Rx_inv, identity);
/// ```
pub fn inv_rotate_z(rad_angle: f32) -> Matrix<f32, 4, 4> {
    rotate_z(-rad_angle)
}
