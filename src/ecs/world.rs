use std::{
    any::{Any, TypeId},
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::anymap::AnyMap;

use super::{entity::EntityBuilder, ECS};

pub struct World {
    ecs: ECS,
    resources: AnyMap,
}

impl World {
    pub fn new() -> Self {
        Self {
            ecs: ECS::new(),
            resources: AnyMap::new(),
        }
    }

    pub fn insert_resource<T: 'static>(&mut self, resource: T) {
        self.resources.insert(RwLock::new(resource));
    }

    pub fn remove_resource<T: 'static>(&mut self) -> Option<T> {
        self.resources
            .remove::<RwLock<T>>()
            .map(|r| r.into_inner().unwrap())
    }

    pub fn read_resource<T: Any + 'static>(&self) -> Result<RwLockReadGuard<T>, anyhow::Error> {
        let resource = self
            .resources
            .get::<RwLock<T>>()
            .ok_or_else(|| anyhow::anyhow!("No such resource {:?}", TypeId::of::<RwLock<T>>()))?;

        Ok(resource.read().unwrap())
    }

    pub fn write_resource<T: Any + 'static>(&self) -> Result<RwLockWriteGuard<T>, anyhow::Error> {
        let resource = self
            .resources
            .get::<RwLock<T>>()
            .ok_or_else(|| anyhow::anyhow!("No such resource {:?}", TypeId::of::<RwLock<T>>()))?;

        Ok(resource.write().unwrap())
    }

    fn create_entity(&self) -> EntityBuilder {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FPS(i32);

    #[test]
    fn test_resources() {
        let mut world = World::new();

        world.insert_resource(FPS(60));

        {
            let fps = world.read_resource::<FPS>().unwrap();

            assert_eq!(fps.0, 60);
        }

        {
            let mut fps = world.write_resource::<FPS>().unwrap();

            fps.0 = 30;
        }

        {
            let fps = world.read_resource::<FPS>().unwrap();

            assert_eq!(fps.0, 30);
        }
    }

    #[test]
    fn test_entity_with_components() {
        let mut world = World::new();

        let entity_builder = world.create_entity();
    }
}
