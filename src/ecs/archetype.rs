use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
};

use super::{
    entity::{Entity, EntityMeta},
    storage::table::{Table, TableRow},
    ComponentId,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ArchetypeId(u32);

impl ArchetypeId {
    /// The ID for the [`Archetype`] without any components.
    pub const EMPTY: ArchetypeId = ArchetypeId(0);

    pub const INVALID: ArchetypeId = ArchetypeId(u32::MAX);

    #[inline]
    pub const fn new(index: usize) -> Self {
        ArchetypeId(index as u32)
    }

    #[inline]
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug)]
pub struct ArchetypeEdge {
    pub(crate) add: Option<ArchetypeId>,
    pub(crate) remove: Option<ArchetypeId>,
}

/// Metadata about an [`Entity`] in a [`Archetype`].
#[derive(Debug)]
pub struct ArchetypeEntity {
    entity: Entity,
    table_row: TableRow,
}

impl ArchetypeEntity {
    /// The ID of the entity.
    #[inline]
    pub const fn id(&self) -> Entity {
        self.entity
    }

    /// The row in the [`Table`] where the entity's components are stored.
    ///
    /// [`Table`]: crate::storage::Table
    #[inline]
    pub const fn table_row(&self) -> TableRow {
        self.table_row
    }
}

/// An opaque location within a [`Archetype`].
///
/// This can be used in conjunction with [`ArchetypeId`] to find the exact location
/// of an [`Entity`] within a [`World`]. An entity's archetype and index can be
/// retrieved via [`Entities::get`].
///
/// [`World`]: crate::world::World
/// [`Entities::get`]: crate::entity::Entities
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
// SAFETY: Must be repr(transparent) due to the safety requirements on EntityLocation
#[repr(transparent)]
pub struct ArchetypeRow(u32);

impl ArchetypeRow {
    /// Index indicating an invalid archetype row.
    /// This is meant to be used as a placeholder.
    pub const INVALID: ArchetypeRow = ArchetypeRow(u32::MAX);

    /// Creates a `ArchetypeRow`.
    #[inline]
    pub const fn new(index: usize) -> Self {
        Self(index as u32)
    }

    /// Gets the index of the row.
    #[inline]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug)]
pub struct Archetype {
    id: ArchetypeId,
    pub(crate) components_ids: Vec<ComponentId>,
    pub(crate) components: Table,
    edges: HashMap<ComponentId, ArchetypeEdge>,
    entities: Vec<ArchetypeEntity>,
}

pub(crate) struct ArchetypeSwapRemoveResult {
    pub(crate) swapped_entity: Option<Entity>,
    pub(crate) table_row: TableRow,
}

impl Archetype {
    pub fn new(id: ArchetypeId, components: Table, components_ids: Vec<ComponentId>) -> Self {
        Self {
            id,
            components_ids,
            components,
            edges: HashMap::new(),
            entities: Vec::new(),
        }
    }

    pub fn id(&self) -> ArchetypeId {
        self.id
    }

    /// Returns if the component id already exist in this archetype
    pub fn contains(&self, component_id: ComponentId) -> bool {
        self.components_ids.contains(&component_id)
    }

    pub fn edges(&self) -> &HashMap<ComponentId, ArchetypeEdge> {
        &self.edges
    }

    pub fn edges_mut(&mut self) -> &mut HashMap<ComponentId, ArchetypeEdge> {
        &mut self.edges
    }

    /// Allocates an entity to the archetype.
    ///
    /// # Safety
    /// valid component values must be immediately written to the relevant storages
    /// `table_row` must be valid
    #[inline]
    pub(crate) unsafe fn allocate(&mut self, entity: Entity, table_row: TableRow) -> EntityMeta {
        let archetype_row = ArchetypeRow::new(self.entities.len());
        self.entities.push(ArchetypeEntity { entity, table_row });

        EntityMeta {
            archetype_id: self.id,
            archetype_row,
            table_row,
        }
    }

    pub(crate) fn swap_remove(&mut self, row: ArchetypeRow) -> ArchetypeSwapRemoveResult {
        let is_last = row.index() == self.entities.len() - 1;
        let entity = self.entities.swap_remove(row.index());

        ArchetypeSwapRemoveResult {
            swapped_entity: if is_last {
                None
            } else {
                Some(self.entities[row.index()].entity)
            },
            table_row: entity.table_row,
        }
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
        let mut archetypes = Vec::new();

        archetypes.push(Archetype {
            id: ArchetypeId::new(archetypes.len()),
            components_ids: Vec::new(),
            components: Table::default(),
            edges: HashMap::new(),
            entities: Vec::new(),
        });

        Self {
            archetypes,
            by_component: HashMap::new(),
        }
    }

    pub fn get(&self, id: ArchetypeId) -> Option<&Archetype> {
        self.archetypes.iter().find(|a| a.id == id)
    }

    pub fn get_mut(&mut self, id: ArchetypeId) -> Option<&mut Archetype> {
        self.archetypes.iter_mut().find(|a| a.id == id)
    }

    pub fn get_archetype_sets(&self, component_id: ComponentId) -> Option<&ArchetypeSet> {
        self.by_component.get(&component_id)
    }

    pub fn get_archetype_sets_mut(
        &mut self,
        component_id: ComponentId,
    ) -> Option<&mut ArchetypeSet> {
        self.by_component.get_mut(&component_id)
    }

    pub fn archetypes(&self) -> Vec<&Archetype> {
        self.archetypes.iter().collect()
    }
}

impl std::ops::Index<ArchetypeId> for Archetypes {
    type Output = Archetype;

    #[inline]
    fn index(&self, index: ArchetypeId) -> &Self::Output {
        &self.archetypes[index.index()]
    }
}

impl std::ops::IndexMut<ArchetypeId> for Archetypes {
    #[inline]
    fn index_mut(&mut self, index: ArchetypeId) -> &mut Self::Output {
        &mut self.archetypes[index.index()]
    }
}
