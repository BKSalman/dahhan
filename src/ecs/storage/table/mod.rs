// this file has elements from `bevy_ecs`

use std::collections::HashMap;

use column::Column;

use crate::ecs::ComponentId;

mod column;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableRow(u32);

impl TableRow {
    pub(crate) const INVALID: TableRow = TableRow(u32::MAX);

    /// Creates a `TableRow`.
    #[inline]
    pub const fn from_u32(index: u32) -> Self {
        Self(index)
    }

    /// Creates a `TableRow` from a [`usize`] index.
    ///
    /// # Panics
    ///
    /// Will panic in debug mode if the provided value does not fit within a [`u32`].
    #[inline]
    pub const fn from_usize(index: usize) -> Self {
        debug_assert!(index as u32 as usize == index);
        Self(index as u32)
    }

    /// Gets the index of the row as a [`usize`].
    #[inline]
    pub const fn as_usize(self) -> usize {
        // usize is at least u32 in Bevy
        self.0 as usize
    }

    /// Gets the index of the row as a [`usize`].
    #[inline]
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

/// A unique ID for a [`Table`]
///
/// Each [`Archetype`] points to a table via [`Archetype::table_id`].
pub struct TableId(u32);

impl TableId {
    pub(crate) const INVALID: TableId = TableId(u32::MAX);

    /// Creates a new [`TableId`].
    ///
    /// `index` *must* be retrieved from calling [`TableId::as_u32`] on a `TableId` you got
    /// from a table of a given [`World`] or the created ID may be invalid.
    ///
    /// [`World`]: crate::world::World
    #[inline]
    pub const fn from_u32(index: u32) -> Self {
        Self(index)
    }

    /// Creates a new [`TableId`].
    ///
    /// `index` *must* be retrieved from calling [`TableId::as_usize`] on a `TableId` you got
    /// from a table of a given [`World`] or the created ID may be invalid.
    ///
    /// [`World`]: crate::world::World
    ///
    /// # Panics
    ///
    /// Will panic if the provided value does not fit within a [`u32`].
    #[inline]
    pub const fn from_usize(index: usize) -> Self {
        debug_assert!(index as u32 as usize == index);
        Self(index as u32)
    }

    /// Gets the underlying table index from the ID.
    #[inline]
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    /// Gets the underlying table index from the ID.
    #[inline]
    pub const fn as_usize(self) -> usize {
        // usize is at least u32 in Bevy
        self.0 as usize
    }

    /// The [`TableId`] of the [`Table`] without any components.
    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }
}

#[derive(Default, Debug)]
pub struct Table {
    columns: HashMap<ComponentId, Column>,
}

impl Table {
    pub fn get_column(&self, component_id: ComponentId) -> Option<&Column> {
        self.columns.get(&component_id)
    }
    pub fn get_column_mut(&mut self, component_id: ComponentId) -> Option<&mut Column> {
        self.columns.get_mut(&component_id)
    }
}

pub struct Tables {
    tables: Vec<Table>,
}

impl Tables {
    pub fn new() -> Self {
        Self { tables: Vec::new() }
    }
}
