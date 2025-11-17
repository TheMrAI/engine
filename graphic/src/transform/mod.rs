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
//!
//! ## Projection matrices
//!
//! Projection matrices transform **world space** into the **normalized view volume**.
//!
//! The transformations bellow assume the dimensions of the **normalized view volume** to be
//! ```text
//! -1.0 <= x <= 1.0
//! -1.0 <= y <= 1.0
//!  0.0 <= z <= 1.0
//! ```
//! where the coordinates use a left-handed system.
//! This is what `DirectX` and `WebGPU` uses.
//! `OpenGL` and `Vulkan` are similar, but they are not supported here.
//!
//! The topic is not trivial and may be confusing to the uninitiated.
//! Good resources describing the underlying math can be found at:
//! [OpenGL Overview](https://www.songho.ca/opengl/gl_overview.html), while it is about
//! `OpenGL` the basic math is the exact same. In combination with some in-depth descriptions
//! from [Real-Time Rendering](https://www.realtimerendering.com/) book one may understand
//! what is necessary.

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
/// Given a rotation matrix `R` and a translation matrix `T` the "Point At" `Pa` matrix can
/// be defined as:
/// ```text
/// Pa = T * R
/// ```
/// 
/// The "Point At" [Matrix] is a rigid-body transformation.
/// 
/// # Preconditions
/// 
/// The `O` object is expected to be located at the origin, in world space, with its
/// desired final position being at `source`.
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
    let left = up.cross(forward).normalized();
    let up = forward.cross(left).normalized();

    m![
        [left[0], up[0], forward[0], source[0]],
        [left[1], up[1], forward[1], source[1]],
        [left[2], up[2], forward[2], source[2]],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// Generate a "Look At" [Matrix] for object `O`.
///  
/// Given a rotation matrix `R` and a translation matrix `T` the "Look At" `La` matrix can
/// be defined as:
/// ```text
/// La = (T * R)^-1 = R^-1 * T^-1
/// ```
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
/// # use float_eq::assert_float_eq;
/// let source = v![1.0, 2.0, 3.0];
/// let target = v![4.0, 5.0, 6.0];
/// let up = v![0.0, 1.0, 0.0];
/// 
/// let point_at = point_at(source, target, up);
/// let look_at = look_at(source, target, up);
/// 
/// let identity = identity_matrix();
/// let point_look = point_at * look_at;
/// 
/// point_look.as_slices().iter().flatten().zip(identity.as_slices().iter().flatten()).for_each(|(l, r)| assert_float_eq!(*l, *r, abs <= 3.0 * f32::EPSILON));
/// ```
/// 
/// Mostly used for handling cameras in a scene. In practice a camera
/// is expected to always stay at the origo, looking down the -Z axis, while
/// up being the positive Y axis. From this position the camera is never moved,
/// but the whole space is transformed in the inverse direction.
/// Instead of moving the camera one way, everything else moves the other way,
/// producing the same effect in the end.
/// 
/// The "Look At" [Matrix] is a rigid-body transformation.
/// 
/// # Preconditions
/// 
/// The `O` object is expected to be located at the `source`, in world space, with its
/// desired final position being at the origin.
#[rustfmt::skip]
pub fn look_at(
    source: Vector<f32, 3>,
    target: Vector<f32, 3>,
    up: Vector<f32, 3>,
) -> Matrix<f32, 4, 4> {
    let forward = (source - target).normalized();
    let left = up.cross(forward).normalized();
    let up = forward.cross(left).normalized();

    m![
        [left[0],    left[1],    left[2],    -source * left],
        [up[0],      up[1],      up[2],      -source * up],
        [forward[0], forward[1], forward[2], -source * forward],
        [0.0,        0.0,        0.0,        1.0],
    ]
}
