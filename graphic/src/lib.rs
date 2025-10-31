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
pub fn identity_matrix() -> Matrix<f32, 4, 4> {
    m![
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0]
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
