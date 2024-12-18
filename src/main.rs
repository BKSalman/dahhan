use dahhan::renderer::Renderer;
use std::sync::Arc;
use winit::{
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

struct App {
    window: Option<Arc<Window>>,
    window_id: Option<WindowId>,
    renderer: Option<Renderer>,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            window_id: None,
            renderer: None,
        }
    }
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
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

        let event_res = renderer.handle_egui_event(&event);

        if !event_res.consumed {
            match event {
                WindowEvent::Resized(new_size) => {
                    renderer.resize(new_size);
                    // On macos the window needs to be redrawn manually after resizing
                    window.request_redraw();
                }
                WindowEvent::RedrawRequested => renderer.draw(),

                WindowEvent::CloseRequested => event_loop.exit(),
                _ => renderer.draw(),
            };
        }

        if event_res.repaint {
            renderer.draw();
        }
    }
}

pub fn main() {
    let event_loop = EventLoop::new().unwrap();
    env_logger::init();

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
