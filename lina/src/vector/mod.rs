//!
//!
//! ```
//! use lina::v;
//!
//! let lhs = v![1,2,3];
//! let rhs = v![1,2,3];
//! assert_eq!(lhs + rhs, [2, 4, 6]);
//! ```

mod accessor;
mod add;
mod add_assign;
mod default;
mod div;
mod div_assign;
mod index;
mod index_mut;
mod macros;
mod mul;
mod mul_assign;
mod sqrt;
mod sub;
mod sub_assign;

// Re-export to allow users their own implementations.
pub use sqrt::Sqrt;

// In this case module inception is allowed, because [vector] symbols
// will be re-exported. The goal is to keep the modules structure separate from the
// module elements.
#[allow(clippy::module_inception)]
mod vector;

pub use macros::*;
pub use vector::*;
