use std::hash::Hash;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct SparseSet<I, V> {
    sparse: SparseArray<I, usize>,
    dense: Vec<V>,
}

impl<I: SparseIndex, V> SparseSet<I, V> {
    pub fn new() -> Self {
        Self {
            sparse: SparseArray::new(),
            dense: Vec::new(),
        }
    }
}

impl<I: SparseIndex, V> SparseSet<I, V> {
    pub fn get(&self, index: I) -> Option<&V> {
        self.sparse.get(index).map(|si| &self.dense[*si])
    }

    pub fn get_mut(&mut self, index: I) -> Option<&mut V> {
        self.sparse.get(index).map(|si| &mut self.dense[*si])
    }

    pub fn insert(&mut self, index: I, value: V) {
        if let Some(dense_index) = self.sparse.get(index.clone()) {
            // # Safety: if dense index exists, value always exists
            unsafe { *self.dense.get_unchecked_mut(dense_index.sparse_index()) = value };
        } else {
            self.sparse.insert(index, self.dense.len());
            self.dense.push(value);
        }
    }
}

pub trait SparseIndex: Clone + PartialEq + Eq + Hash {
    fn sparse_index(&self) -> usize;
    fn new_sparse_index(value: usize) -> Self;
}

impl SparseIndex for u8 {
    fn sparse_index(&self) -> usize {
        *self as usize
    }

    fn new_sparse_index(value: usize) -> Self {
        value as Self
    }
}

impl SparseIndex for u16 {
    fn sparse_index(&self) -> usize {
        *self as usize
    }

    fn new_sparse_index(value: usize) -> Self {
        value as Self
    }
}

impl SparseIndex for u32 {
    fn sparse_index(&self) -> usize {
        *self as usize
    }

    fn new_sparse_index(value: usize) -> Self {
        value as Self
    }
}

impl SparseIndex for u64 {
    fn sparse_index(&self) -> usize {
        *self as usize
    }

    fn new_sparse_index(value: usize) -> Self {
        value as Self
    }
}

impl SparseIndex for usize {
    fn sparse_index(&self) -> usize {
        *self
    }

    fn new_sparse_index(value: usize) -> Self {
        value
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct SparseArray<I, V = I> {
    pub(crate) values: Vec<Option<V>>,
    pub(crate) phantom: PhantomData<I>,
}

impl<I, V> SparseArray<I, V> {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            phantom: PhantomData,
        }
    }
}

impl<I: SparseIndex, V> SparseArray<I, V> {
    pub fn get(&self, index: I) -> Option<&V> {
        self.values
            .get(index.sparse_index())
            .and_then(Option::as_ref)
    }

    pub fn get_mut(&mut self, index: I) -> Option<&mut V> {
        self.values
            .get_mut(index.sparse_index())
            .and_then(Option::as_mut)
    }

    pub fn insert(&mut self, index: I, value: V) {
        let index = index.sparse_index();
        if index >= self.values.len() {
            self.values.resize_with(index + 1, || None);
        }
        self.values[index] = Some(value);
    }

    pub fn remove(&mut self, index: I) -> Option<V> {
        self.values
            .get_mut(index.sparse_index())
            .and_then(Option::take)
    }
}
