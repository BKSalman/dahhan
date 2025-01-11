use std::{
    any::{Any, TypeId},
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::anymap::AnyMap;

use super::{
    component::{Components, ComponentsInfo},
    entity::Entity,
    generational_array::GenerationalIndexAllocator,
};

pub struct World {
    entity_allocator: GenerationalIndexAllocator,
    entities: Vec<Entity>,
    components_info: ComponentsInfo,
    components: Components,
    resources: AnyMap,
}

impl World {
    pub fn new() -> Self {
        Self {
            resources: AnyMap::new(),
            components: Components::new(),
            components_info: ComponentsInfo::new(),
            entity_allocator: GenerationalIndexAllocator::new(),
            entities: Vec::new(),
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

    pub fn register_component<T: 'static>(&mut self) {
        let component_id = self.components_info.register_component::<T>();
        self.components.register_component::<T>(component_id);
    }

    // FIXME: this should NOT use a Box, that's very bad
    pub fn add_entity(&mut self, components: Vec<Box<dyn Any>>) -> Entity {
        let entity = self.entity_allocator.allocate();
        let entity = Entity::from(entity);
        self.entities.push(entity);

        for component in components {
            let component_info = self
                .components_info
                .get_by_type_id(component.type_id())
                .unwrap();
            self.components
                .insert_component(entity, component_info.id(), component);
        }

        entity
    }

    pub fn add_component<T: 'static>(&mut self, entity: Entity, component: T) {
        let component_info = self
            .components_info
            .get_by_type_id(TypeId::of::<T>())
            .unwrap();
        if let Some(component_sparse_set) = self.components.get_mut(component_info.id()) {
            component_sparse_set.insert(entity, component);
        }
    }

    pub fn remove_component<T: 'static>(&mut self, entity: Entity) {
        let component_info = self
            .components_info
            .get_by_type_id(TypeId::of::<T>())
            .unwrap();
        if let Some(component_sparse_set) = self.components.get_mut(component_info.id()) {
            component_sparse_set.remove_entity(entity);
        }
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

    // #[test]
    // fn test_entity_with_components() {

    //     let mut world = World::new();

    //     let entity_builder = world.create_entity();
    // }
}
