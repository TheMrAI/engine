//! Camera
//!
//! Right handed, Y-up coordinate system
//!

use lina::{matrix::Matrix, v, vector::Vector};

use crate::{cross, transform::look_at, transform::rotate_y};

/// Simple Camera with basic movement support.
///
/// It supports a basic classic FPS like movement,
/// by going forward, backwards, up/down and turning left/right.
/// Not very smart, or good, but it works for now.
/// More complicated movements like, proper mouse support are
/// problematic with the current matrix rotation system.
pub struct Camera {
    eye: Vector<f32, 4>,
    look_dir: Vector<f32, 4>,
    up_dir: Vector<f32, 4>,
}

impl Camera {
    pub fn move_on_look_at_vector(&mut self, units: f32) {
        self.eye += self.look_dir * units;
    }

    pub fn move_on_right_vector(&mut self, units: f32) {
        let right = cross(self.look_dir, self.up_dir).norm();
        self.eye += right * units;
    }

    pub fn move_on_up_vector(&mut self, units: f32) {
        self.eye += self.up_dir * units;
    }

    pub fn yaw(&mut self, radians: f32) {
        self.look_dir = rotate_y(radians) * self.look_dir;
    }

    pub fn as_transform_matrix(&self) -> Matrix<f32, 4, 4> {
        let target = self.eye + self.look_dir;
        // Unwrap is perfectly safe as we are in a 4x4 matrix
        look_at(
            self.eye.xyz().unwrap(),
            target.xyz().unwrap(),
            self.up_dir.xyz().unwrap(),
        )
    }
}

/// The default implementation is temporary
/// until we provide proper construction pattern to it.
impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: v![0.0, 0.0, 5.0, 1.0],
            look_dir: v![0.0, 0.0, -1.0, 0.0],
            up_dir: v![0.0, 1.0, 0.0, 0.0],
        }
    }
}
