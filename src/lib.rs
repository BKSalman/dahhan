use camera::{update_camera_uniform, Camera};
use ecs::{
    component::TupleAddComponent,
    default_systems::{draw, render_sprites},
    entity::Entity,
    rendering::{Sprite, Transform},
    scheduler::{IntoSystem, Scheduler, System},
    world::World,
    Component,
};
use input::Input;
use prelude::{Query, Write};
use renderer::Renderer;
use std::{sync::Arc, time::Instant};
use winit::{
    event::{StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

mod anymap;
mod buffers;
pub mod camera;
pub mod ecs;
mod egui_renderer;
pub mod input;
pub mod renderer;
mod vertices;

pub mod prelude {
    pub use crate::{
        ecs::{
            query::{Query, Read, Write},
            rendering::{Sprite, Transform},
            scheduler::{Res, ResMut, Scheduler},
        },
        input::Input,
    };
}

#[rustfmt::skip]
pub(crate) const OPENGL_TO_WGPU_MATRIX: glam::Mat4 = glam::Mat4::from_cols_array(&[
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
]);

pub struct App {
    state: State,
    event_loop: EventLoop<()>,
}

impl App {
    pub fn new() -> Self {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        let state = State::new();

        Self { event_loop, state }
    }

    pub fn run(mut self) -> Result<(), winit::error::EventLoopError> {
        self.event_loop.run_app(&mut self.state)
    }

    pub fn insert_resource<T: 'static>(&mut self, resource: T) {
        self.state.world.insert_resource(resource);
    }

    pub fn register_component<T: Component>(&mut self) {
        self.state.world.register_component::<T>();
    }

    pub fn add_entity<T: TupleAddComponent>(&mut self, components: T) -> Entity {
        self.state.world.add_entity(components)
    }

    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) {
        self.state.world.add_component(entity, component);
    }

    pub fn remove_component<T: Component>(&mut self, entity: Entity) {
        self.state.world.remove_component::<T>(entity);
    }

    pub fn add_system<I, S: System + 'static>(&mut self, system: impl IntoSystem<I, System = S>) {
        self.state.scheduler.add_system(system);
    }
}

struct State {
    window: Option<Arc<Window>>,
    window_id: Option<WindowId>,
    last_frame_time: Instant,
    world: World,
    scheduler: Scheduler,
}

impl State {
    fn new() -> Self {
        let mut world = World::new();

        world.register_component::<Transform>();
        world.register_component::<Sprite>();
        world.register_component::<Camera>();

        world.insert_resource(Input::new());

        Self {
            window: None,
            window_id: None,
            last_frame_time: Instant::now(),
            world,
            scheduler: Scheduler::new(),
        }
    }

    pub(crate) fn init_rendering(&mut self, renderer: Renderer) {
        self.world.insert_resource(renderer);

        self.scheduler.add_system(render_sprites);
        self.scheduler.add_system(draw);
        self.scheduler.add_system(update_camera_uniform);
    }
}

impl winit::application::ApplicationHandler for State {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        let window_attributes = Window::default_attributes()
            .with_title("Fantastic window number one!")
            .with_inner_size(winit::dpi::LogicalSize::new(128.0, 128.0));
        let window = event_loop.create_window(window_attributes).unwrap();
        let window = Arc::new(window);

        let renderer = Renderer::new(Arc::clone(&window));
        self.init_rendering(renderer);

        // self.renderer = Some();
        self.window_id = Some(window.id());
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_ref() else {
            return;
        };

        window.request_redraw();

        // let event_res = {
        //     let mut renderer = self.world.write_resource::<Renderer>().unwrap();
        //     renderer.handle_egui_event(&event)
        // };

        // if !event_res.consumed {

        match event {
            WindowEvent::Resized(new_size) => {
                {
                    let mut renderer = self.world.write_resource::<Renderer>().unwrap();
                    renderer.resize(new_size);
                }

                let camera = self.world.query::<Query<Write<Camera>>>();
                if let Some((_, camera)) = camera.iter().next() {
                    match camera {
                        Camera::Ortho(orthographic_camera) => {
                            orthographic_camera.update_projection_matrix(
                                -(new_size.width as f32 / 2.),
                                new_size.width as f32 / 2.,
                                -(new_size.height as f32 / 2.),
                                new_size.height as f32 / 2.,
                            );
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                self.scheduler.run(&mut self.world);
                // let mut renderer = self.world.write_resource::<Renderer>().unwrap();
                // renderer.draw(|ctx| {}, wgpu::Color::BLACK);
                {
                    let mut input = self.world.write_resource::<Input>().unwrap();
                    input.scroll_delta = 0.;
                }
                self.last_frame_time = Instant::now();
            }

            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic,
            } => {
                if !is_synthetic {
                    let mut input = self.world.write_resource::<Input>().unwrap();
                    match event.physical_key {
                        key @ PhysicalKey::Code(KeyCode::AltLeft)
                        | key @ PhysicalKey::Code(KeyCode::AltRight)
                        | key @ PhysicalKey::Code(KeyCode::ControlLeft)
                        | key @ PhysicalKey::Code(KeyCode::ControlRight)
                        | key @ PhysicalKey::Code(KeyCode::ShiftLeft)
                        | key @ PhysicalKey::Code(KeyCode::ShiftRight)
                        | key @ PhysicalKey::Code(KeyCode::Meta)
                        | key @ PhysicalKey::Code(KeyCode::SuperLeft)
                        | key @ PhysicalKey::Code(KeyCode::SuperRight)
                        | key @ PhysicalKey::Code(KeyCode::Hyper) => {
                            if event.state.is_pressed() {
                                input.pressed_modifiers.insert(key);
                            } else {
                                input.pressed_modifiers.remove(&key);
                            }
                        }
                        key => {
                            if event.state.is_pressed() {
                                input.pressed_keys.insert(key);
                            } else {
                                input.pressed_keys.remove(&key);
                            }
                        }
                    }
                }
            }
            WindowEvent::MouseWheel {
                device_id: _,
                delta,
                phase: _,
            } => {
                let mut input = self.world.write_resource::<Input>().unwrap();
                input.scroll_delta = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, lines) => lines,
                    winit::event::MouseScrollDelta::PixelDelta(physical_position) => {
                        physical_position.y as f32
                    }
                };
            }
            _ => {}
        };
        // }
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        match cause {
            StartCause::Poll => {
                self.last_frame_time = Instant::now();
            }
            _ => {}
        }
    }
}
