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

// Implement for build in floating point types.

macro_rules! impl_sqrt_for_float_types {
    ($($T: ty),* $(,)*) => {$(
        impl Sqrt for $T
        {
            type Output = $T;

            fn square_root(self) -> Self::Output {
                self.sqrt()
            }
        }
    )*};
}

impl_sqrt_for_float_types!(f32, f64);
