use glam::Mat4;

use crate::OPENGL_TO_WGPU_MATRIX;

#[derive(Debug)]
pub struct OrthographicCamera {
    projection_matrix: Mat4,
}

impl OrthographicCamera {
    pub fn new(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        Self {
            projection_matrix: Mat4::orthographic_rh(left, right, bottom, top, near, far),
        }
    }

    pub fn build_view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(
            (0.0, 1.0, 2.0).into(),
            (0.0, 0.0, 0.0).into(),
            glam::Vec3::Y,
        );

        OPENGL_TO_WGPU_MATRIX * self.projection_matrix * view
    }
}
