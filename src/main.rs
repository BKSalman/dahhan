use dahhan::App;

pub fn main() {
    env_logger::init();

    let app = App::new();

    app.run().unwrap();
}
