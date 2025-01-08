use std::any::Any;

use crate::ecs::storage::blob_vec::BlobVec;

use super::TableRow;

#[derive(Debug)]
pub struct Column {
    data: BlobVec,
}

impl Column {
    pub fn get<T: Any>(&self, row: TableRow) -> Option<&T> {
        unsafe { self.data.get(row.as_usize()) }
    }

    pub fn get_mut<T: Any>(&mut self, row: TableRow) -> Option<&mut T> {
        unsafe { self.data.get_mut(row.as_usize()) }
    }

    pub fn swap_remove<T: Any>(&mut self, row: TableRow) -> T {
        unsafe { self.data.swap_remove::<T>(row.as_usize()) }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}
