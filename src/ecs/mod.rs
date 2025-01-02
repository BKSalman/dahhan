use std::any::{Any, TypeId};

use archetype::Archetypes;
use entity::{Entity, EntityMap, EntityMeta};
use generational_array::GenerationalIndexAllocator;

mod archetype;
pub mod entity;
pub mod generational_array;
pub mod resources;
pub mod world;

pub struct ECS {
    entity_allocator: GenerationalIndexAllocator,
    entities: EntityMap<EntityMeta>,
    archetypes: Archetypes,
}

impl ECS {
    pub fn new() -> Self {
        Self {
            entity_allocator: GenerationalIndexAllocator::new(),
            entities: EntityMap::new(),
            archetypes: Archetypes::new(),
        }
    }

    // pub fn create_entity(&mut self) -> EntityBuilder {
    //     EntityBuilder::new()
    // }

    pub fn has_component<T: Any>(&self, entity: Entity) -> bool {
        let entity_meta = self.entities.get(entity).unwrap();

        let archetype_set = self.archetypes.get_archetypes::<T>().unwrap();

        archetype_set.contains(&entity_meta.archetype_id)
    }
}
