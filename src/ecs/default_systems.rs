use wgpu::Color;

use crate::camera::Camera;
use crate::ecs::query::{Query, Read};
use crate::ecs::rendering::{Sprite, Transform};
use crate::renderer::Renderer;
use crate::vertices::VertexColored;

use super::scheduler::ResMut;

pub(crate) fn render_sprites(
    sprites: Query<(Read<Sprite>, Read<Transform>)>,
    cameras: Query<(Read<Camera>, Read<Transform>)>,
    mut renderer: ResMut<Renderer>,
) {
    if cameras.iter().next().is_some() {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut current_index: u16 = 0;

        for (_, (sprite, transform)) in sprites.iter() {
            let width = sprite.size.x * transform.scale.x;
            let height = sprite.size.y * transform.scale.y;

            vertices.push(VertexColored {
                position: [
                    transform.position.x,
                    transform.position.y,
                    transform.position.z,
                ],
                color: sprite.color.into(),
            });

            vertices.push(VertexColored {
                position: [
                    transform.position.x,
                    transform.position.y - height,
                    transform.position.z,
                ],
                color: sprite.color.into(),
            });

            vertices.push(VertexColored {
                position: [
                    transform.position.x + width,
                    transform.position.y - height,
                    transform.position.z,
                ],
                color: sprite.color.into(),
            });

            vertices.push(VertexColored {
                position: [
                    transform.position.x + width,
                    transform.position.y,
                    transform.position.z,
                ],
                color: sprite.color.into(),
            });

            indices.push(current_index);
            indices.push(current_index + 1);
            indices.push(current_index + 2);

            indices.push(current_index);
            indices.push(current_index + 2);
            indices.push(current_index + 3);

            current_index += 4;
        }

        renderer.render_sprites(&vertices, &indices);
    }
}

pub(crate) fn draw(renderer: ResMut<Renderer>) {
    let frame = renderer
        .surface
        .get_current_texture()
        .expect("Failed to acquire next swap chain texture");
    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = renderer
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        rpass.set_pipeline(&renderer.render_pipeline);
        rpass.set_bind_group(0, &renderer.camera_bind_group, &[]);
        rpass.set_vertex_buffer(0, renderer.vertex_buffer.get_slice(..));
        rpass.set_index_buffer(
            renderer.index_buffer.get_slice(..),
            wgpu::IndexFormat::Uint16,
        );
        rpass.draw_indexed(0..renderer.num_indices, 0, 0..1);
    }

    renderer.queue.submit(Some(encoder.finish()));
    frame.present();
}
