use std::f32::consts::PI;

use lina::{m, matrix::Matrix};

/// Generate an orthographic projection matrix for the given AABB (axis aligned bounding box).
/// 
/// Affine.
/// 
/// An orthographic projection keeps parallel lines parallel and objects maintain the
/// same size, regardless of the distance to the camera.
/// It translates the an AABB into the **normalized view volume**.
/// The six arguments `left`, `right`, `bottom`, `top`, `z_near`, `z_far` describe this
/// volume. (`left`, `bottom`, `z_near`) and (`right`, `top`, `z_far`) specify the bottom
/// left and top right points of this view volume.
/// 
/// The projection is nothing more than a translation and a scaling:
/// ```text
/// P = S(s)T(t)
/// ```
/// It merely moves and scales the view volume into **normalized view volume**.
/// 
/// # Preconditions
/// 
/// All arguments are to be defined in **world space**.
/// 
/// This projection assumes that the camera is at the origo looking down at the -Z direction.
/// Thus (`left`, `bottom`, `z_near`) is the minimum corner and (`right`, `top`, `z_far`) the maximum
/// point of the bounding volume.
/// For this reason for proper results, it is expected that
/// ```text
/// left < right
/// bottom < top
/// z_far < z_near
/// z_near < 0.0
/// z_far != -infinity
/// ```
/// Keep especial attention on `z_far` being smaller than `z_near`. This is because
/// the camera is assumed to be looking down the -Z axis.
/// 
/// Breaking these preconditions is undefined behavior.
/// 
/// Checks are provided for debug builds only, otherwise the caller must ensure the provided
/// values are correct.
#[rustfmt::skip]
pub fn orthographic_proj(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    z_near: f32,
    z_far: f32,
) -> Matrix<f32, 4, 4> {
    debug_assert!(left < right);
    debug_assert!(bottom < top);
    debug_assert!(z_far < z_near);
    debug_assert!(z_near < 0.0);
    debug_assert!(z_far != f32::INFINITY);

    m![
        [2.0 / (right - left), 0.0,                  0.0,                    -(right + left)  / (right - left)],
        [0.0,                  2.0 / (top - bottom), 0.0,                    -(top + bottom) / (top - bottom)],
        [0.0,                  0.0,                  1.0 / (z_far - z_near), -z_near / (z_far - z_near)],
        [0.0,                  0.0,                  0.0,                    1.0]
    ]
}

/// Generate a perspective projection matrix for potentially asymmetric frustrum.
/// 
/// [perspective_proj_g], perspective projection generic implements the most
/// configurable implementation of the projection. All other **perspective_proj_***
/// functions are derivations of this one and if not otherwise stated all restrictions
/// described here apply to them as well.
/// 
/// The six arguments `left`, `right`, `bottom`, `top`, `z_near`, `z_far` are all the
/// parameters describing the view frustrum.
/// (`left`, `bottom`, `z_near`) and (`right`, `top`, `z_near`) specify the bottom left and
/// top right point of the nearest frustrum plane to the origo.
/// `z_far` is the distance of the farthest frutstrum plane from the origo.
/// 
/// # Preconditions
/// 
/// All arguments should be defined in **world space**.
/// 
/// ```text
/// left < 0.0
/// 0.0 < right
/// bottom < 0.0
/// 0.0 < top
/// z_near < 0.0
/// z_far < z_near
/// z_far > -infinity
/// ```
/// 
/// Keep especial attention on `z_far` being smaller than `z_near`. This is because
/// the camera is assumed to be looking down the -Z axis.
/// 
/// `z_far` can't be equal to negative infinity. In case this is necessary use
/// [perspective_proj_g_inf] instead. 
/// 
/// Breaking these preconditions is undefined behavior. 
/// 
/// Checks are provided for debug builds only, otherwise the caller must ensure the provided
/// values are correct.
#[rustfmt::skip]
pub fn perspective_proj_g(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    z_near: f32,
    z_far: f32,
) -> Matrix<f32, 4, 4> {
    debug_assert!(left < 0.0);
    debug_assert!(0.0 < right);
    debug_assert!(bottom < 0.0);
    debug_assert!(0.0 < top);
    debug_assert!(z_near < 0.0);
    debug_assert!(z_far < z_near);
    debug_assert!(z_far > f32::NEG_INFINITY);

    // The values are inverted, because the matrix expects them to be positive.
    // They could be expected to positive on the interface as well, but that would
    // only lead to unnecessary confusion as this matrix expect the camera
    // to face down the -Z axis in a right handed coordinate system.
    let z_near = -z_near;
    let z_far = -z_far;

     m![
        [(2.0 * z_near) / (right - left), 0.0, (right + left) / (right - left), 0.0],
        [0.0, (2.0 * z_near) / (top - bottom), (top + bottom) / (top - bottom), 0.0],
        [0.0, 0.0, -z_far/(z_far - z_near), -(z_far * z_near) / (z_far - z_near)],
        [0.0, 0.0, -1.0, 0.0] 
    ]
}

/// Generate a perspective projection matrix for potentially asymmetric frustrum
/// and **-infinite z_far** distance.
/// 
/// All requirements from [perspective_proj_g] stand, with the exception
/// that `z_far` is assumed to be -infinite.
#[rustfmt::skip]
pub fn perspective_proj_g_inf(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    z_near: f32,
) -> Matrix<f32, 4, 4> {
    debug_assert!(left < 0.0);
    debug_assert!(0.0 < right);
    debug_assert!(bottom < 0.0);
    debug_assert!(0.0 < top);
    debug_assert!(z_near < 0.0);
    // The values are inverted, because the matrix expects them to be positive.
    // They could be expected to positive on the interface as well, but that would
    // only lead to unnecessary confusion as this matrix expect the camera
    // to face down the -Z axis in a right handed coordinate system.
    let z_near = -z_near;

     m![
        [(2.0 * z_near) / (right - left), 0.0, (right + left) / (right - left), 0.0],
        [0.0, (2.0 * z_near) / (top - bottom), (top + bottom) / (top - bottom), 0.0],
        [0.0, 0.0, -1.0, -z_near],
        [0.0, 0.0, -1.0, 0.0] 
    ]
}

/// Generate a perspective projection matrix with a symmetric frustrum.
/// 
/// All requirements from [perspective_proj_g] stand, with the additions
/// ```text
/// right = -left
/// top = -bottom
/// ```
#[rustfmt::skip]
pub fn perspective_proj_sym(
    right: f32,
    top: f32,
    z_near: f32,
    z_far: f32,
) -> Matrix<f32, 4, 4> {
    debug_assert!(0.0 < right);
    debug_assert!(0.0 < top);
    debug_assert!(z_near < 0.0);
    debug_assert!(z_far < z_near);
    debug_assert!(z_far > f32::NEG_INFINITY);
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

/// Generate a perspective projection matrix with a symmetric frustrum and
/// **-infinite z_far** distance.
/// 
/// All requirements from [perspective_proj_g] stand, with the additions
/// ```text
/// right = -left
/// top = -bottom
/// ```
/// and `z_far` being -infinity.
#[rustfmt::skip]
pub fn perspective_proj_sym_inf(
    right: f32,
    top: f32,
    z_near: f32,
) -> Matrix<f32, 4, 4> {
    debug_assert!(0.0 < right);
    debug_assert!(0.0 < top);
    debug_assert!(z_near < 0.0);
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

/// Generate a perspective projection matrix with a symmetric frustrum using
/// horizontal FOV and aspect ratio.
/// 
/// In most use cases it is rather difficult to define the appropriate
/// values for [perspective_proj_g].
/// It is much friendlier to use a **horizontal field of view** (FOV) and the
/// **aspect ratio** for defining the matrix.
/// 
/// # Preconditions
/// 
/// All arguments should be defined in **world space**.
/// `fov_x` is expected to be in **radians**.
/// 
/// ```text
/// 0.0 < fov_x
/// fov_x < PI
/// 0.0 < aspect_ratio
/// z_near < 0.0
/// z_far < z_near
/// z_far > -infinity
/// ```
/// 
/// Keep especial attention on `z_far` being smaller than `z_near`. This is because
/// the camera is assumed to be looking down the -Z axis. 
/// 
/// Breaking these preconditions is undefined behavior. 
/// 
/// Checks are provided for debug builds only, otherwise the caller must ensure the provided
/// values are correct.
#[rustfmt::skip]
pub fn perspective_proj_sym_h_fov(
    fov_x: f32,
    aspect_ratio: f32,
    z_near: f32,
    z_far: f32,
) -> Matrix<f32, 4, 4> {
    debug_assert!(0.0 < fov_x);
    debug_assert!(fov_x < PI);
    debug_assert!(0.0 < aspect_ratio);
    debug_assert!(z_near < 0.0);
    debug_assert!(z_far < z_near);
    debug_assert!(z_far > f32::NEG_INFINITY);

    let tangent = (fov_x / 2.0).tan();
    // The z_near is negated because it comes in as a negative value
    // but we do not wish to invert the `right` value.
    let right = -z_near * tangent;
    let top = right / aspect_ratio;

    perspective_proj_sym(right, top, z_near, z_far)
}

/// Generate a perspective projection matrix with a symmetric frustrum using
/// vertical FOV and aspect ratio.
/// 
/// All requirements from [perspective_proj_sym_h_fov] stand, with the only
/// difference being that now the **vertical field of view** (`fov_y`) has
/// to be provided in radians.
#[rustfmt::skip]
pub fn perspective_proj_sym_v_fov(
    fov_y: f32,
    aspect_ratio: f32,
    z_near: f32,
    z_far: f32,
) -> Matrix<f32, 4, 4> {
    debug_assert!(0.0 < fov_y);
    debug_assert!(fov_y < PI);
    debug_assert!(0.0 < aspect_ratio);
    debug_assert!(z_near < 0.0);
    debug_assert!(z_far < z_near);
    debug_assert!(z_far > f32::NEG_INFINITY);

    let tangent = (fov_y / 2.0).tan();
    // The z_near is negated because it comes in as a negative value
    // but we do not wish to invert the `right` value.
    let top = -z_near * tangent;
    let right = top * aspect_ratio;

    perspective_proj_sym(right, top, z_near, z_far)
}
