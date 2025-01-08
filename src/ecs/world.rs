use std::{
    any::{Any, TypeId},
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::anymap::AnyMap;

use super::{
    archetype::{Archetype, ArchetypeId, ArchetypeRow, Archetypes},
    entity::{Entity, EntityAllocator, EntityBuilder, EntityMap, EntityMeta},
    generational_array::GenerationalIndexAllocator,
    storage::table::{TableRow, Tables},
    ComponentId,
};

pub struct World {
    entity_allocator: EntityAllocator,
    entities: EntityMap,
    archetypes: Archetypes,
    tables: Tables,
    resources: AnyMap,
}

impl World {
    pub fn new() -> Self {
        Self {
            entity_allocator: EntityAllocator::new(),
            entities: EntityMap::new(),
            archetypes: Archetypes::new(),
            resources: AnyMap::new(),
            tables: Tables::new(),
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

    fn create_entity(&mut self) -> Entity {
        let entity = self.entity_allocator.allocate();
        self.entities.insert(
            entity,
            EntityMeta {
                archetype_id: ArchetypeId::EMPTY,
                table_row: TableRow::INVALID,
                archetype_row: ArchetypeRow::INVALID,
            },
        );

        entity
    }

    pub fn has_component<T: Any>(&self, entity: Entity) -> bool {
        let entity_meta = self.entities.get(entity).unwrap();

        let component_id = TypeId::of::<T>();

        let archetype_set = self.archetypes.get_archetype_sets(component_id).unwrap();

        archetype_set.contains(&entity_meta.archetype_id)
    }

    pub fn add_component<T: Any>(&mut self, entity: Entity, component: T) {
        let Some(entity_meta) = self.entities.get_mut(entity) else {
            return;
        };

        let component_id = component.type_id();

        let new_archetype_id = insert_bundle_into_archetype(
            component_id,
            &mut self.archetypes,
            entity_meta.archetype_id,
        );

        let Some(entity_archetype) = self.archetypes.get_mut(entity_meta.archetype_id) else {
            return;
        };

        if new_archetype_id == entity_archetype.id() {
            // TODO: make sure the add is cached as an edge
            let entry = entity_archetype.edges_mut().entry(component_id).or_insert(
                super::archetype::ArchetypeEdge {
                    add: Some(new_archetype_id),
                    remove: None,
                },
            );

            entry.add = Some(new_archetype_id);
        } else {
            // TODO: move entity to new archetype
            entity_meta.archetype_id = new_archetype_id;

            let result = entity_archetype.swap_remove(entity_meta.archetype_row);

            if let Some(swapped_entity) = result.swapped_entity {
                let archetype_row = entity_meta.archetype_row;
                let swapped_location =
                        // SAFETY: If the swap was successful, swapped_entity must be valid.
                        unsafe { self.entities.get(swapped_entity).unwrap_unchecked() };
                unsafe {
                    self.entities.set(
                        swapped_entity,
                        EntityMeta {
                            archetype_id: swapped_location.archetype_id,
                            archetype_row,
                            table_row: swapped_location.table_row,
                        },
                    );
                }
            }

            // TODO: move to new table
            // TODO: allocate entity on new archetype
            // TODO: write component data to the new table
        }
    }

    pub fn get_component<T: Any>(&self, entity: Entity) -> Option<&T> {
        let entity_meta = self.entities.get(entity).unwrap();
        let component_id = TypeId::of::<T>();

        let archetype_set = self.archetypes.get_archetype_sets(component_id).unwrap();

        if !archetype_set.contains(&entity_meta.archetype_id) {
            return None;
        }

        if let Some(archetype) = self.archetypes.get(entity_meta.archetype_id) {
            return archetype
                .components
                .get_column(component_id)
                .and_then(|column| column.get::<T>(entity_meta.table_row));
        }

        None
    }
}

pub(crate) fn insert_bundle_into_archetype(
    component_id: ComponentId,
    archetypes: &mut Archetypes,
    // components: &Components,
    current_archetype_id: ArchetypeId,
) -> ArchetypeId {
    if let Some(add_edge) = archetypes[current_archetype_id]
        .edges()
        .get(&component_id)
        .and_then(|e| e.add)
    {
        return add_edge;
    }

    todo!()
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
