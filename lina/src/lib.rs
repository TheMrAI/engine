//! # Linear algebra library
//!
//! For a start it will only support simple vector
//! manipulations, but eventually it should be published.
//!
//! ## Improvement idea
//!
//! Consider implementing the operators for types which implement the Copy trait as well.
//! Does that need to be handled as a special case or not?

pub mod vector;
// Re-exporting all the internal symbols, for easier
// usage.
pub use vector::vector::*;

#[cfg(test)]
mod tests {
    use crate::v;

    #[test]
    fn scope_check() {
        let v = v![1, 2, 3];
    }
}
