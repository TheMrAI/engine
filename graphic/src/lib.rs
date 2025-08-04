//! Provide all common functionality necessary for basic
//! rendering operations. Building basic transformation
//! matrices and transforming/mapping memory to GPU format.
//!
//!

use lina::{m, matrix::Matrix};

#[rustfmt::skip]
pub fn translate(translate_x: f32, translate_y: f32, translate_z: f32) -> Matrix<f32, 4, 4> {
    m![
        [1.0, 0.0, 0.0, translate_x],
        [0.0, 1.0, 0.0, translate_y],
        [0.0, 0.0, 1.0, translate_z],
        [0.0, 0.0, 0.0, 1.0]
    ]
}

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

// pub fn look_at(
//     eye: Vector<f32, 4>,
//     target: Vector<f32, 4>,
//     up: Vector<f32, 4>,
// ) -> Matrix<f32, 4, 4> {
//     let z_axis = normalize(eye - target);
//     let x_axis = normalize(&cross(up, &z_axis));
//     let y_axis = normalize(&cross(&z_axis, &x_axis));

//     m![
//         [x_axis[0], y_axis[0], z_axis[0], eye[0]],
//         [x_axis[1], y_axis[1], z_axis[1], eye[1]],
//         [x_axis[2], y_axis[2], z_axis[2], eye[2]],
//         [0.0, 0.0, 0.0, 1.0],
//     ]
// }
