mod add;
mod add_assign;
mod adjoint;
mod default;
mod determinant;
mod index;
mod index_mut;
mod inverse;
mod macros;
mod mul;
mod mul_assign;
mod sub;
mod sub_assign;

#[allow(clippy::module_inception)]
mod matrix;

pub use macros::*;
pub use matrix::*;
