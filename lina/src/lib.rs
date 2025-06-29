//! Linear algebra library
//!
//! For a start it will only support simple vector
//! manipulations, but eventually it should be published.

mod vector;
// Re-exporting all the internal symbols, for easier
// usage.
pub use vector::vector::*;
