use glam::{Mat4, Vec3};

use crate::prelude::Transform;

#[derive(Debug)]
pub struct OrthographicCamera {
    projection_matrix: Mat4,
}

impl OrthographicCamera {
    pub fn new(left: f32, right: f32, bottom: f32, top: f32) -> Self {
        Self {
            projection_matrix: Mat4::orthographic_rh(left, right, bottom, top, -1000., 1000.),
        }
    }

    pub fn get_projection_matrix(&self) -> Mat4 {
        self.projection_matrix
    }

    pub fn update_projection_matrix(&mut self, left: f32, right: f32, bottom: f32, top: f32) {
        self.projection_matrix = Mat4::orthographic_rh(left, right, bottom, top, -1000., 1000.);
    }

    pub fn build_view_projection_matrix(&self, transform: &Transform) -> Mat4 {
        let view = Mat4::look_at_rh(Vec3::new(0., 0., 2.), Vec3::ZERO, Vec3::Y);

        let scale = Mat4::from_scale(glam::Vec3::new(
            transform.scale.x.max(0.001),
            transform.scale.y.max(0.001),
            1.0,
        ));

        let view = view * scale;

        self.projection_matrix * view
    }
}
