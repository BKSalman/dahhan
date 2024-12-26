use dahhan::{Engine, Game};

struct MyGame {}

impl Game for MyGame {
    fn egui_render(&mut self, context: &egui::Context) {
        egui::Window::new("lmao").show(context, |ui| ui.label("lmao"));
    }
}

pub fn main() {
    env_logger::init();

    let engine = Engine::new(Box::new(MyGame {}));

    engine.run().unwrap();
}
