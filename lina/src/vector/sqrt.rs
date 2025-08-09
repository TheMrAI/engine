/// Square root trait
///
/// The standard library does not define square root trait,
/// making generic implementations relying on it problematic.
/// In attempt to rectify this tiny trait skeleton is presented.
pub trait Sqrt {
    type Output;

    /// The `square_root` function
    /// The name is deliberately verbose so it doesn't
    /// collide with the more common `sqrt` shorthands.
    fn square_root(self) -> Self::Output;
}
