use std::{
    any::{Any, TypeId},
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::anymap::AnyMap;

use super::{
    component::{Component, Components, ComponentsInfo, TupleAddComponent},
    entity::Entity,
    generational_array::GenerationalIndexAllocator,
    scheduler::Scheduler,
    Query, WorldQueryable,
};

pub struct World {
    entity_allocator: GenerationalIndexAllocator,
    entities: Vec<Entity>,
    components_info: ComponentsInfo,
    components: Components,
    resources: AnyMap,
    scheduler: Scheduler,
}

impl World {
    pub fn new() -> Self {
        Self {
            resources: AnyMap::new(),
            components: Components::new(),
            components_info: ComponentsInfo::new(),
            entity_allocator: GenerationalIndexAllocator::new(),
            entities: Vec::new(),
            scheduler: Scheduler::new(),
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

    pub fn register_component<T: Component>(&mut self) {
        let component_id = self.components_info.register_component::<T>();
        self.components.register_component::<T>(component_id);
    }

    pub fn add_entity<T: TupleAddComponent>(&mut self, components: T) -> Entity {
        let entity = self.entity_allocator.allocate();
        let entity = Entity::from(entity);
        self.entities.push(entity);

        components.add_component(&self.components_info, &mut self.components, entity);

        entity
    }

    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) {
        let component_info = self
            .components_info
            .get_by_type_id(TypeId::of::<T>())
            .unwrap();
        if let Some(component_sparse_set) = self.components.get_mut(component_info.id()) {
            component_sparse_set.insert(entity, component);
        }
    }

    pub fn remove_component<T: Component>(&mut self, entity: Entity) {
        let component_info = self
            .components_info
            .get_by_type_id(TypeId::of::<T>())
            .unwrap();
        if let Some(component_sparse_set) = self.components.get_mut(component_info.id()) {
            component_sparse_set.remove_entity(entity);
        }
    }

    pub fn iter_component<'a, T: Component>(&'a self) -> std::slice::Iter<'a, T> {
        let component_info = self
            .components_info
            .get_by_type_id(TypeId::of::<T>())
            .unwrap();
        self.components
            .get(component_info.id())
            .map(|component_sparse_set| component_sparse_set.iter())
            .unwrap_or([].iter())
    }

    pub fn query<'a, T: WorldQueryable>(&'a self) -> <T as WorldQueryable>::Item<'a> {
        T::query(&self.components_info, &self.components)
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
