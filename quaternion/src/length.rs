use crate::Quaternion;

macro_rules! impl_length_for_float_types {
    ($($T: ty),* $(,)*) => {$(
        impl Quaternion<$T> {
            /// Calculate the length/norm of the quaternion.
            ///
            /// For a given quaternion q:
            /// ```text
            /// q = s + ix + jy + kz
            /// ```
            /// It will calculate the length/norm `n(q)`:
            /// ```text
            /// n(q) = sqrt(s^2 + x^2 + y^2 + z^2)
            /// ```
            ///
            /// In case the second power of the length
            /// is required, it is more efficient to just call
            /// [length_squared](Quaternion::length_squared).
            pub fn length(&self) -> $T {
                self.length_squared().sqrt()
            }
        }
    )*};
}

impl_length_for_float_types!(f32, f64);

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use lina::v;

    use crate::Quaternion;

    #[test]
    fn length() {
        let q: Quaternion<f32> = Quaternion::new_parts(1.0, v![2.0, 3.0, 4.0]);
        assert_float_eq!(q.length(), 30.0f32.sqrt(), ulps <= 0);
    }
}
