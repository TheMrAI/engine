mod add;
mod add_assign;
mod default;
mod index;
mod index_mut;
mod macros;
mod mul;
mod mul_assign;
mod sub;
mod sub_assign;
mod adjoint;

#[allow(clippy::module_inception)]
mod matrix;

pub use macros::*;
pub use matrix::*;
