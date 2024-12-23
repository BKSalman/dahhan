use std::{
    borrow::Cow,
    num::{NonZero, NonZeroU64},
    sync::Arc,
};

use egui_wgpu::ScreenDescriptor;
use egui_winit::EventResponse;
use wgpu::{
    util::RenderEncoder, BindGroup, BufferDescriptor, Device, PipelineCompilationOptions, Queue,
    RenderPipeline, Surface, SurfaceConfiguration,
};
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

use crate::{buffers::SlicedBuffer, egui_renderer::EguiRenderer, vertices::VertexColored};

pub struct Renderer {
    surface: Surface<'static>,
    window: Arc<Window>,
    config: SurfaceConfiguration,
    device: Device,
    queue: Queue,
    render_pipeline: RenderPipeline,
    uniform_bind_group: BindGroup,
    egui_renderer: EguiRenderer,
    vertex_buffer: SlicedBuffer,
    index_buffer: SlicedBuffer,
}

impl Renderer {
    pub fn new(window: Arc<Window>) -> Self {
        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);

        let instance = wgpu::Instance::default();

        let surface = instance.create_surface(Arc::clone(&window)).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
        }))
        .expect("Failed to find an appropriate adapter");

        // Create the logical device and command queue
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

        // Load the shaders from disk
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("test-shader.wgsl"))),
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

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Uniform Buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: 1024,
            mapped_at_creation: true,
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
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

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Buffer"),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            size: INDEX_BUFFER_START_CAPACITY,
            mapped_at_creation: false,
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            size: VERTEX_BUFFER_START_CAPACITY,
            mapped_at_creation: false,
        });

        Self {
            surface,
            config,
            egui_renderer: EguiRenderer::new(&device, swapchain_format, None, 1, &window),
            device,
            render_pipeline,
            queue,
            uniform_bind_group,
            window,
            index_buffer: SlicedBuffer::new(index_buffer, INDEX_BUFFER_START_CAPACITY),
            vertex_buffer: SlicedBuffer::new(vertex_buffer, VERTEX_BUFFER_START_CAPACITY),
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.config.width = new_size.width.max(1);
        self.config.height = new_size.height.max(1);
        self.surface.configure(&self.device, &self.config);
    }

    pub fn update(&mut self) {
        // TODO
    }

    pub fn draw(&mut self, egui_ui: impl FnMut(&egui::Context), clear_color: wgpu::Color) {
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
            let mut vertex_buffer_staging = self.vertex_buffer.write_into(
                &self.queue,
                NonZero::new((std::mem::size_of::<VertexColored>() * 3) as u64).unwrap(),
            );

            let left = -1.0;
            let right = 1.0;
            let bottom = -1.0;
            let top = 1.0;
            let near = -1.0;
            let far = 1.0;

            let ortho_matrix = glam::Mat4::orthographic_rh(left, right, bottom, top, near, far);

            let mut triangle = vec![
                VertexColored {
                    position: [-0.5, -0.5, 0.],
                    color: [0.0, 0.0, 0.0],
                },
                VertexColored {
                    position: [0.5, -0.5, 0.],
                    color: [0.0, 0.0, 0.0],
                },
                VertexColored {
                    position: [0.0, 0.5, 0.],
                    color: [0.0, 0.0, 0.0],
                },
            ];

            vertex_buffer_staging.copy_from_slice(bytemuck::cast_slice(&triangle));

            rpass.set_vertex_buffer(0, self.vertex_buffer.get_slice(..));

            rpass.draw(0..3, 0..1);
        }

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };

        self.egui_renderer.draw(
            &self.device,
            &self.queue,
            &mut encoder,
            &self.window,
            &view,
            screen_descriptor,
            egui_ui,
        );

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    pub fn handle_egui_event(&mut self, event: &WindowEvent) -> EventResponse {
        self.egui_renderer.handle_input(&self.window, event)
    }
}
