use std::{borrow::Cow, sync::Arc};

use egui_wgpu::ScreenDescriptor;
use egui_winit::EventResponse;
use glam::Mat4;
use wgpu::{
    util::DeviceExt, BindGroup, Buffer, Device, PipelineCompilationOptions, Queue, RenderPipeline,
    Surface, SurfaceConfiguration,
};
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

use crate::{
    buffers::SlicedBuffer, camera_uniform::CameraUniform, egui_renderer::EguiRenderer,
    orthographic_camera::OrthographicCamera, vertices::VertexColored,
};

const VERTICES: &[VertexColored] = &[
    VertexColored {
        position: [-0.0868241, 0.49240386, 0.0],
        color: [0.5, 0.0, 0.5],
    },
    VertexColored {
        position: [-0.49513406, 0.06958647, 0.0],
        color: [0.5, 0.0, 0.5],
    },
    VertexColored {
        position: [-0.21918549, -0.44939706, 0.0],
        color: [0.5, 0.0, 0.5],
    },
    VertexColored {
        position: [0.35966998, -0.3473291, 0.0],
        color: [0.5, 0.0, 0.5],
    },
    VertexColored {
        position: [0.44147372, 0.2347359, 0.0],
        color: [0.5, 0.0, 0.5],
    },
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UniformBuffer {
    screen_size: [f32; 2],
    _padding: [u32; 2],
}

impl UniformBuffer {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            screen_size: [screen_width, screen_height],
            _padding: Default::default(),
        }
    }
}

pub struct Renderer {
    pub(crate) surface: Surface<'static>,
    window: Arc<Window>,
    config: SurfaceConfiguration,
    pub(crate) device: Device,
    pub(crate) queue: Queue,
    pub(crate) render_pipeline: RenderPipeline,
    // pub(crate) camera_bind_group: BindGroup,
    pub(crate) uniform_bind_group: BindGroup,
    pub(crate) uniform_buffer: Buffer,
    egui_renderer: EguiRenderer,
    pub(crate) vertex_buffer: SlicedBuffer,
    pub(crate) num_indices: u32,
    pub(crate) index_buffer: SlicedBuffer,
    // camera: OrthographicCamera,
    // camera_buffer: wgpu::Buffer,
    // camera_uniform: CameraUniform,
}

impl Renderer {
    pub(crate) fn new(window: Arc<Window>) -> Self {
        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);

        let instance = wgpu::Instance::default();

        let surface = instance.create_surface(Arc::clone(&window)).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .expect("Failed to find an appropriate adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                required_limits:
                    wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
            },
            None,
        ))
        .expect("Failed to create device");

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let uniform = UniformBuffer::new(size.width as f32, size.height as f32);

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Uniform Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        has_dynamic_offset: false,
                        min_binding_size: None,
                        ty: wgpu::BufferBindingType::Uniform,
                    },
                    count: None,
                }],
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(swapchain_capabilities.formats[0]);

        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        surface.configure(&device, &config);

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[VertexColored::desc()],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(swapchain_format.into())],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        const VERTEX_BUFFER_START_CAPACITY: wgpu::BufferAddress =
            (std::mem::size_of::<VertexColored>() * 1024) as _;
        const INDEX_BUFFER_START_CAPACITY: wgpu::BufferAddress =
            (std::mem::size_of::<u32>() * 1024 * 3) as _;

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            size: VERTEX_BUFFER_START_CAPACITY,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Buffer"),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            size: INDEX_BUFFER_START_CAPACITY,
            mapped_at_creation: false,
        });

        let vertex_buffer_size = vertex_buffer.size();
        let index_buffer_size = index_buffer.size();

        let num_indices = INDICES.len() as u32;

        Self {
            surface,
            config,
            egui_renderer: EguiRenderer::new(&device, swapchain_format, None, 1, &window),
            device,
            render_pipeline,
            queue,
            window,
            index_buffer: SlicedBuffer::new(index_buffer, index_buffer_size),
            vertex_buffer: SlicedBuffer::new(vertex_buffer, vertex_buffer_size),
            // camera,
            // camera_bind_group,
            // camera_uniform,
            // camera_buffer,
            uniform_bind_group,
            num_indices,
            uniform_buffer,
        }
    }

    pub(crate) fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.config.width = new_size.width.max(1);
        self.config.height = new_size.height.max(1);
        let mut writer = self
            .queue
            .write_buffer_with(
                &self.uniform_buffer,
                0,
                std::num::NonZeroU64::new(std::mem::size_of::<UniformBuffer>() as u64).unwrap(),
            )
            .expect("Failed to create staging buffer for vertex data");
        writer.copy_from_slice(bytemuck::cast_slice(&[UniformBuffer::new(
            new_size.width as f32,
            new_size.height as f32,
        )]));

        self.surface.configure(&self.device, &self.config);
    }

    pub(crate) fn update(&mut self) {
        // TODO
    }

    pub(crate) fn draw(&mut self, egui_ui: impl FnMut(&egui::Context), clear_color: wgpu::Color) {
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_bind_group(0, &self.uniform_bind_group, &[]);
            rpass.set_vertex_buffer(0, self.vertex_buffer.get_slice(..));
            rpass.set_index_buffer(self.index_buffer.get_slice(..), wgpu::IndexFormat::Uint16);
            rpass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        // let screen_descriptor = ScreenDescriptor {
        //     size_in_pixels: [self.config.width, self.config.height],
        //     pixels_per_point: self.window.scale_factor() as f32,
        // };

        // self.egui_renderer.draw(
        //     &self.device,
        //     &self.queue,
        //     &mut encoder,
        //     &self.window,
        //     &view,
        //     screen_descriptor,
        //     egui_ui,
        // );

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    pub(crate) fn handle_egui_event(&mut self, event: &WindowEvent) -> EventResponse {
        self.egui_renderer.handle_input(&self.window, event)
    }

    pub fn render_sprites(&mut self, vertices: &[VertexColored], indices: &[u16]) {
        if vertices.is_empty() || indices.is_empty() {
            return;
        }

        // Update the vertex buffer with new data
        self.queue.write_buffer(
            &self.vertex_buffer.buffer,
            0,
            bytemuck::cast_slice(vertices),
        );

        // Update the index buffer with new data
        self.queue
            .write_buffer(&self.index_buffer.buffer, 0, bytemuck::cast_slice(indices));

        // Update the number of indices to draw
        self.num_indices = indices.len() as u32;
    }
}
