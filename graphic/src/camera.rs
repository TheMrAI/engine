//! Camera
//!
//! Right handed, Y-up coordinate system
//!

use lina::{matrix::Matrix, v, vector::Vector};
use quaternion::Quaternion;

use crate::transform::look_at;

/// Simple Camera with basic movement support.
///
/// It supports a basic classic FPS like movement,
/// by going forward, backwards, up/down and turning left/right.
pub struct Camera {
    eye: Vector<f32, 3>,
    pitch: f32,
    roll: f32,
    yaw: f32,
}

impl Camera {
    fn recalculate_orientation(&self) -> Quaternion<f32> {
        let pitch = Quaternion::<f32>::new_unit(self.pitch, v![1.0, 0.0, 0.0]);
        // Camera is looking down at the -Z direction.
        let roll = Quaternion::<f32>::new_unit(self.roll, v![0.0, 0.0, -1.0]);
        let yaw = Quaternion::<f32>::new_unit(self.yaw, v![0.0, 1.0, 0.0]);

        roll * yaw * pitch
    }

    pub fn move_on_look_at_vector(&mut self, units: f32) {
        let q = self.recalculate_orientation();

        let look_dir = Quaternion::from_vector(v![0.0, 0.0, -1.0])
            .conjugate_by(q)
            .vector();

        self.eye += look_dir * units;
    }

    pub fn move_on_right_vector(&mut self, units: f32) {
        let q = self.recalculate_orientation();

        let look_dir = Quaternion::from_vector(v![0.0, 0.0, -1.0])
            .conjugate_by(q)
            .vector();
        let up_dir = Quaternion::from_vector(v![0.0, 1.0, 0.0])
            .conjugate_by(q)
            .vector();

        let right = look_dir.cross(up_dir).norm();
        self.eye += right * units;
    }

    pub fn move_on_up_vector(&mut self, units: f32) {
        let q = self.recalculate_orientation();

        let up_dir = Quaternion::from_vector(v![0.0, 1.0, 0.0])
            .conjugate_by(q)
            .vector();
        self.eye += up_dir * units;
    }

    pub fn roll(&mut self, radians: f32) {
        self.roll += radians;
    }

    pub fn pitch(&mut self, radians: f32) {
        self.pitch += radians;
    }

    pub fn yaw(&mut self, radians: f32) {
        self.yaw += radians;
    }

    pub fn as_transform_matrix(&self) -> Matrix<f32, 4, 4> {
        let q = self.recalculate_orientation();

        let look_dir = Quaternion::from_vector(v![0.0, 0.0, -1.0])
            .conjugate_by(q)
            .vector();
        let up_dir = Quaternion::from_vector(v![0.0, 1.0, 0.0])
            .conjugate_by(q)
            .vector();

        let target = self.eye + look_dir;
        // Unwrap is perfectly safe as we are in a 4x4 matrix
        look_at(self.eye, target, up_dir)
    }
}

/// The default implementation is temporary
/// until we provide proper construction pattern to it.
impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: v![0.0, 0.0, 5.0],
            pitch: 0.0,
            roll: 0.0,
            yaw: 0.0,
        }
    }
}
