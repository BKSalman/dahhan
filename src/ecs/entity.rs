use super::{generational_array::GenerationalIndex, storage::sparse_set::SparseIndex};

// pub struct EntityAllocator(GenerationalIndexAllocator);

// impl EntityAllocator {
//     pub fn new() -> Self {
//         Self(GenerationalIndexAllocator::new())
//     }

//     pub fn allocate(&mut self) -> Entity {
//         Entity(self.0.allocate())
//     }
// }

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Entity(GenerationalIndex);

impl Entity {
    pub fn index(&self) -> usize {
        self.0.index()
    }
}

impl From<GenerationalIndex> for Entity {
    fn from(value: GenerationalIndex) -> Self {
        Self(value)
    }
}

impl SparseIndex for Entity {
    fn sparse_index(&self) -> usize {
        self.index()
    }

    fn new_sparse_index(value: usize) -> Self {
        Self(GenerationalIndex::from_raw(value))
    }
}
