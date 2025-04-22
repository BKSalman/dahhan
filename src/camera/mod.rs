use camera_uniform::CameraUniform;
use orthographic_camera::OrthographicCamera;

use crate::{
    ecs::Component,
    prelude::{Query, Read, ResMut, Transform},
    renderer::Renderer,
};

pub mod camera_uniform;
pub mod orthographic_camera;

pub enum Camera {
    Ortho(OrthographicCamera),
    // TODO: add perspective camera
}

impl Camera {
    pub fn default_2d() -> Self {
        Camera::Ortho(OrthographicCamera::new(0., 0., 0., 0.))
    }
}

impl Component for Camera {}

pub fn update_camera_uniform(
    query: Query<(Read<Camera>, Read<Transform>)>,
    renderer: ResMut<Renderer>,
) {
    // Find the camera entity
    if let Some((_, (camera, transform))) = query.iter().next() {
        match camera {
            Camera::Ortho(orthographic_camera) => {
                let view_proj = orthographic_camera.build_view_projection_matrix(transform);

                // Update the camera uniform
                let mut camera_uniform = CameraUniform::new();
                camera_uniform.update_view_proj(&view_proj);

                renderer.queue.write_buffer(
                    &renderer.camera_buffer,
                    0,
                    bytemuck::cast_slice(&[camera_uniform]),
                );
            }
        }
    }
}
