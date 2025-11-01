use lina::{m, matrix::Matrix};

/// Generate an orthographic projection matrix for the given AABB (axis aligned bounding box).
/// 
/// Affine.
/// 
/// An orthographic projection keeps parallel lines parallel and objects maintain the
/// same size, regardless of the distance to the camera.
/// 
/// The projection is nothing more than a translation and a scaling:
/// ```text
/// P = S(s)T(t)
/// ```
/// It merely moves and scales the view volume into **normalized view volume**.
/// 
/// This matrix assumes the dimensions of the **normalized view volume** to be
/// ```text
/// -1.0 <= x <= 1.0
/// -1.0 <= y <= 1.0
///  0.0 <= z <= 1.0
/// ```
/// where the coordinates use a left-handed system.
/// This is what `DirectX` and `WebGPU` are using.
/// 
/// # Preconditions
/// 
/// This projection assumes that the camera is at the origo looking down at the -Z direction.
/// Thus (`left`, `bottom`, `z_near`) is the minimum corner and (`right`, `top`, `z_far`) the maximum
/// point of the bounding volume.
/// For this reason for proper results, it is expected that
/// ```text
/// left < right
/// bottom < top
/// z_far < z_near
/// ```
/// Keep especial attention on `z_far` being smaller than `z_near`. This is because
/// the camera is assumed to be looking down the -Z direction.
/// 
/// Breaking these preconditions leads to undefined behavior. 
#[rustfmt::skip]
pub fn orthographic_proj(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    z_near: f32,
    z_far: f32,
) -> Matrix<f32, 4, 4> {
    m![
        [2.0 / (right - left), 0.0,                  0.0,                    -(right + left)  / (right - left)],
        [0.0,                  2.0 / (top - bottom), 0.0,                    -(top + bottom) / (top - bottom)],
        [0.0,                  0.0,                  1.0 / (z_far - z_near), -z_near / (z_far - z_near)],
        [0.0,                  0.0,                  0.0,                    1.0]
    ]
}

// ok this works as expected!!!
// now we need to translate and scale the z axis
#[rustfmt::skip]
pub fn perspective_proj_g(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    z_near: f32,
    z_far: f32,
) -> Matrix<f32, 4, 4> {
    // The values are inverted, because the matrix expects them to be positive.
    // They could be expected to positive on the interface as well, but that would
    // only lead to unnecessary confusion as this matrix expect the camera
    // to face down the -Z axis in a right handed coordinate system.
    let z_near = -z_near;
    let z_far = -z_far;

     m![
        [(2.0 * z_near) / (right - left), 0.0, (right + left) / (right - left),    0.0],
        [0.0, (2.0 * z_near) / (top - bottom), (top + bottom) / (top - bottom), 0.0],
        [0.0, 0.0, -z_far/(z_far - z_near), -(z_far * z_near) / (z_far - z_near)],
        [0.0, 0.0, -1.0, 0.0] 
    ]
}

#[rustfmt::skip]
pub fn perspective_proj_g_inf(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    z_near: f32,
) -> Matrix<f32, 4, 4> {
    // The values are inverted, because the matrix expects them to be positive.
    // They could be expected to positive on the interface as well, but that would
    // only lead to unnecessary confusion as this matrix expect the camera
    // to face down the -Z axis in a right handed coordinate system.
    let z_near = -z_near;

     m![
        [(2.0 * z_near) / (right - left), 0.0, (right + left) / (right - left),    0.0],
        [0.0, (2.0 * z_near) / (top - bottom), (top + bottom) / (top - bottom), 0.0],
        [0.0, 0.0, -1.0, -z_near],
        [0.0, 0.0, -1.0, 0.0] 
    ]
}

/// For symmetric frustra it is assumed that:
/// - r = -l
/// - top = -bottom
/// 
/// Which leads to the simplified transformation matrix derived from: [perspective_proj_g].
#[rustfmt::skip]
pub fn perspective_proj_sym(
    right: f32,
    top: f32,
    z_near: f32,
    z_far: f32,
) -> Matrix<f32, 4, 4> {
    // The values are inverted, because the matrix expects them to be positive.
    // They could be expected to positive on the interface as well, but that would
    // only lead to unnecessary confusion as this matrix expect the camera
    // to face down the -Z axis in a right handed coordinate system.
    let z_near = -z_near;
    let z_far = -z_far;

     m![
        [z_near / right,    0.0,          0.0,                      0.0],
        [0.0,               z_near / top, 0.0,                      0.0],
        [0.0,               0.0,          -z_far/(z_far - z_near),  -(z_far * z_near) / (z_far - z_near)],
        [0.0,               0.0,          -1.0,                     0.0] 
    ]
}

#[rustfmt::skip]
pub fn perspective_proj_sym_inf(
    right: f32,
    top: f32,
    z_near: f32,
) -> Matrix<f32, 4, 4> {
    // The values are inverted, because the matrix expects them to be positive.
    // They could be expected to positive on the interface as well, but that would
    // only lead to unnecessary confusion as this matrix expect the camera
    // to face down the -Z axis in a right handed coordinate system.
    let z_near = -z_near;

     m![
        [z_near / right,    0.0,           0.0,   0.0],
        [0.0,               z_near / top,  0.0,   0.0],
        [0.0,               0.0,          -1.0,   -z_near],
        [0.0,               0.0,          -1.0,    0.0] 
    ]
}

#[rustfmt::skip]
pub fn perspective_proj_sym_h_fov(
    fov_x: f32,
    aspect_ratio: f32,
    z_near: f32,
    z_far: f32,
) -> Matrix<f32, 4, 4> {
    
    let tangent = (fov_x / 2.0).tan();
    // The z_near is negated because it comes in as a negative value
    // but we do not with to invert the right value.
    let right = -z_near * tangent;
    let top = right / aspect_ratio;

    perspective_proj_sym(right, top, z_near, z_far)
}

#[rustfmt::skip]
pub fn perspective_proj_sym_v_fov(
    fov_y: f32,
    aspect_ratio: f32,
    z_near: f32,
    z_far: f32,
) -> Matrix<f32, 4, 4> {
    
    let tangent = (fov_y / 2.0).tan();
    // The z_near is negated because it comes in as a negative value
    // but we do not with to invert the right value.
    let top = -z_near * tangent;
    let right = top * aspect_ratio;

    perspective_proj_sym(right, top, z_near, z_far)
}
