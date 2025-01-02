use crate::anymap::AnyMap;

use super::{
    archetype::ArchetypeId,
    generational_array::{GenerationalIndex, GenerationalIndexArray},
};

pub type Entity = GenerationalIndex;
pub type EntityMap<T> = GenerationalIndexArray<T>;

pub struct EntityMeta {
    pub(crate) archetype_id: ArchetypeId,
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
