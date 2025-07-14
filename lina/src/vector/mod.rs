//!
//!
//! ```
//! use lina::v;
//!
//! let lhs = v![1,2,3];
//! let rhs = v![1,2,3];
//! assert_eq!(lhs + rhs, [2, 4, 6]);
//! ```

mod add;
mod default;
mod macros;
mod mul;
mod sub;

pub mod vector;
pub use macros::*;
