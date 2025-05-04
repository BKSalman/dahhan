#[derive(Debug)]
pub struct Window {
    pub width: f32,
    pub height: f32,
}

impl Window {
    pub fn new() -> Self {
        Self {
            width: 0.,
            height: 0.,
        }
    }
}
