use std::collections::HashSet;

use winit::keyboard::{KeyCode, PhysicalKey};

pub struct Input {
    pub(crate) pressed_keys: HashSet<PhysicalKey>,
    pub(crate) pressed_modifiers: HashSet<PhysicalKey>,
    pub(crate) scroll_delta: f32,
}

impl Input {
    pub(crate) fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            pressed_modifiers: HashSet::new(),
            scroll_delta: 0.,
        }
    }

    /// returns if the provided key is being pressed/held
    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&PhysicalKey::Code(key))
    }

    /// scroll delta from last frame
    pub fn scroll_delta(&self) -> f32 {
        self.scroll_delta
    }
}
