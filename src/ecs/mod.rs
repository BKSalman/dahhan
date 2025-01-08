use std::any::TypeId;

mod archetype;
pub mod entity;
pub mod generational_array;
pub mod resources;
pub mod storage;
pub mod world;

pub type ComponentId = TypeId;
