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
mod project;
mod rotate;
mod scale;
mod translate;

pub use project::*;
pub use rotate::*;
pub use scale::*;
pub use translate::*;

/// Generate a "Point At" [Matrix] for object `O`.
///  
/// Orient an object at `source` position to point in the direction of another object at
/// `target` position.
/// 
/// The `up` [Vector] is used for intermediate calculations.
/// 
/// The "Point At" [Matrix] is a rigid-body transformation.
/// 
/// # Preconditions
/// 
/// The `O` object is expected to be located at the origin, looking down the -Z axis.
/// 
/// Failure to adhere to this requirement is undefined behavior.
///
/// ## Note
/// 
/// It doesn't handle the case when the `up` vector is parallel to the vector between
/// `source` and `target`.
#[rustfmt::skip]
pub fn point_at(
    source: Vector<f32, 3>,
    target: Vector<f32, 3>,
    up: Vector<f32, 3>,
) -> Matrix<f32, 4, 4> {
    let forward = (source - target).normalized();
    let right = up.cross(forward).normalized();
    let up = forward.cross(right).normalized();

    m![
        [right[0], up[0], forward[0], source[0]],
        [right[1], up[1], forward[1], source[1]],
        [right[2], up[2], forward[2], source[2]],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// Generate a "Look At" [Matrix] for object `O`.
///  
/// The "Look At" [Matrix] is the inverse transformation of the "Point At"
/// [Matrix]. While the "Point At" is meant to orient objects in space,
/// the "Look At" is meant to be applied to every other object except the
/// one it was generated for.
/// 
/// ```
/// # use graphic::transform::point_at;
/// # use graphic::transform::look_at;
/// # use graphic::identity_matrix;
/// # use lina::v;
/// let source = v![1.0, 2.0, 3.0];
/// let target = v![4.0, 5.0, 6.0];
/// let up = v![0.0, 1.0, 0.0];
/// 
/// let point_at = point_at(source, target, up);
/// let look_at = look_at(source, target, up);
/// 
/// let identity = identity_matrix();
///  // Fails, because of floating point comparison limitations only
///  //assert_eq!(point_at * look_at, identity);
/// ```
/// 
/// Mostly used for handling cameras in a scene. In practice a camera
/// is expected to always stay at the origo, looking down the -Z axis, while
/// up being the positive Y axis. From this position the camera is never moved,
/// but the whole space is transformed in the inverse direction.
/// Instead of moving the camera one way, everything else moves the other way,
/// producing the same effect in the end.
/// 
/// The "Point At" [Matrix] is a rigid-body transformation.
/// 
/// # Preconditions
/// 
/// The `O` object is expected to be located at the origin, looking down the -Z axis.
/// 
/// Failure to adhere to this requirement is undefined behavior.
#[rustfmt::skip]
pub fn look_at(
    source: Vector<f32, 3>,
    target: Vector<f32, 3>,
    up: Vector<f32, 3>,
) -> Matrix<f32, 4, 4> {
    let forward = (source - target).normalized();
    let right = up.cross(forward).normalized();
    let up = forward.cross(right).normalized();

    m![
        [right[0],   right[1],   right[2],   -source * right],
        [up[0],      up[1],      up[2],      -source * up],
        [forward[0], forward[1], forward[2], -source * forward],
        [0.0,        0.0,        0.0,        1.0],
    ]
}
