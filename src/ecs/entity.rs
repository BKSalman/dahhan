use crate::anymap::AnyMap;

use super::{
    archetype::{Archetype, ArchetypeId, ArchetypeRow},
    generational_array::{GenerationalIndex, GenerationalIndexAllocator, GenerationalIndexArray},
    storage::table::TableRow,
};

pub struct EntityAllocator(GenerationalIndexAllocator);

impl EntityAllocator {
    pub fn new() -> Self {
        Self(GenerationalIndexAllocator::new())
    }

    pub fn allocate(&mut self) -> Entity {
        Entity(self.0.allocate())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Entity(GenerationalIndex);

impl Entity {
    pub fn index(&self) -> usize {
        self.0.index()
    }
}

pub struct EntityMap(GenerationalIndexArray<EntityMeta>);

impl EntityMap {
    pub fn new() -> Self {
        Self(GenerationalIndexArray::new())
    }

    pub fn insert(&mut self, entity: Entity, value: EntityMeta) {
        self.0.insert(entity.0, value);
    }

    pub fn get(&self, entity: Entity) -> Option<&EntityMeta> {
        self.0.get(entity.0)
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut EntityMeta> {
        self.0.get_mut(entity.0)
    }

    pub(crate) unsafe fn set(&mut self, entity: Entity, new_meta: EntityMeta) {
        let meta = self.0.get_unchecked_mut(entity.0);
        *meta = new_meta;
    }
}

pub struct EntityMeta {
    pub(crate) archetype_id: ArchetypeId,
    pub(crate) table_row: TableRow,
    pub(crate) archetype_row: ArchetypeRow,
}

pub struct EntityBuilder {
    components: AnyMap,
}

impl EntityBuilder {
    pub fn new() -> Self {
        Self {
            components: AnyMap::new(),
        }
    }

    // pub fn with_component(&mut self, compo) -> &mut Self {
    // }
}
