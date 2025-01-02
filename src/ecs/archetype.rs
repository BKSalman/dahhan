use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
};

use super::entity::Entity;

type ComponentId = TypeId;

pub type ArchetypeId = u32;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Archetype {
    id: ArchetypeId,
    components: Vec<ComponentId>,
}

impl Archetype {
    pub fn new(id: ArchetypeId, components: Vec<ComponentId>) -> Self {
        Self { id, components }
    }
}

type ArchetypeSet = HashSet<ArchetypeId>;

pub struct Archetypes {
    archetypes: Vec<Archetype>,
    // by_components: HashMap<Vec<ComponentId>, Archetype>,
    by_component: HashMap<ComponentId, ArchetypeSet>,
}

impl Archetypes {
    pub fn new() -> Self {
        Self {
            archetypes: Vec::new(),
            by_component: HashMap::new(),
        }
    }

    pub fn get(&self, id: ArchetypeId) -> Option<&Archetype> {
        self.archetypes.iter().find(|a| a.id == id)
    }

    pub fn get_archetypes<T: Any>(&self) -> Option<&ArchetypeSet> {
        let component_id = TypeId::of::<T>();
        self.by_component.get(&component_id)
    }
}
