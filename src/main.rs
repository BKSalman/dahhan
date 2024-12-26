use dahhan::App;

pub fn main() {
    env_logger::init();

    let engine = App::new();

    engine.run().unwrap();
}
