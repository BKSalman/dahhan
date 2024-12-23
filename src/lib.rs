use renderer::Renderer;
use std::{sync::Arc, time::Instant};
use winit::{
    event::{StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

mod buffers;
mod egui_renderer;
pub mod renderer;
mod vertices;

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
}

struct State {
    window: Option<Arc<Window>>,
    window_id: Option<WindowId>,
    renderer: Option<Renderer>,
    last_frame_time: Instant,
}

impl State {
    fn new() -> Self {
        Self {
            window: None,
            window_id: None,
            renderer: None,
            last_frame_time: Instant::now(),
        }
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

        self.renderer = Some(Renderer::new(Arc::clone(&window)));
        self.window_id = Some(window.id());
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let (Some(window), Some(renderer)) = (self.window.as_ref(), self.renderer.as_mut()) else {
            return;
        };

        window.request_redraw();

        let event_res = renderer.handle_egui_event(&event);

        let ui = |ctx: &egui::Context| {
            egui::Window::new("lmao").show(ctx, |ui| {
                ui.label(format!(
                    "{}ms",
                    Instant::now()
                        .duration_since(self.last_frame_time)
                        .as_millis()
                ))
            });
        };

        if !event_res.consumed {
            match event {
                WindowEvent::Resized(new_size) => {
                    renderer.resize(new_size);
                }
                WindowEvent::RedrawRequested => {
                    renderer.update();
                    renderer.draw(ui, wgpu::Color::BLACK);
                    self.last_frame_time = Instant::now();
                }

                WindowEvent::CloseRequested => event_loop.exit(),
                _ => {}
            };
        }
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
