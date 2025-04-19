use crate::ecs::Component;
use glam::{Vec2, Vec3};

#[derive(Debug, Clone)]
pub struct Sprite {
    pub texture_id: Option<wgpu::Texture>,
    pub size: Vec2,
    pub color: Vec3,
}

impl Component for Sprite {}

#[derive(Debug, Clone)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Component for Transform {}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
}
