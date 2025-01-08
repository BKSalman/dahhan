#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GenerationalIndex {
    index: usize,
    generation: u64,
}

impl GenerationalIndex {
    pub fn index(&self) -> usize {
        self.index
    }
}

struct AllocatorEntry {
    is_live: bool,
    generation: u64,
}

pub struct GenerationalIndexAllocator {
    entries: Vec<AllocatorEntry>,
    free: Vec<usize>,
}

impl GenerationalIndexAllocator {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            free: Vec::new(),
        }
    }

    pub fn allocate(&mut self) -> GenerationalIndex {
        if let Some(index) = self.free.pop() {
            let id_entry = &mut self.entries[index];
            assert!(!id_entry.is_live);
            id_entry.is_live = true;
            GenerationalIndex {
                index,
                generation: id_entry.generation,
            }
        } else {
            self.entries.push(AllocatorEntry {
                is_live: true,
                generation: 0,
            });
            GenerationalIndex {
                index: self.entries.len() - 1,
                generation: 0,
            }
        }
    }

    // Returns true if the index was allocated before and is now deallocated
    pub fn deallocate(&mut self, index: GenerationalIndex) -> bool {
        if index.index >= self.entries.len() {
            return false;
        }

        let id_entry = &mut self.entries[index.index];
        if !id_entry.is_live {
            return false;
        }

        id_entry.is_live = false;
        id_entry.generation = id_entry
            .generation
            .checked_add(1)
            .expect("GenerationalIndex generation overflow");
        self.free.push(index.index);

        true
    }

    pub fn is_live(&self, index: GenerationalIndex) -> bool {
        if index.index < self.entries.len() {
            let id_entry = &self.entries[index.index];
            id_entry.is_live && id_entry.generation == index.generation
        } else {
            false
        }
    }
}

struct ArrayEntry<T> {
    value: T,
    generation: u64,
}

// An associative array from GenerationalIndex to some Value T.
pub struct GenerationalIndexArray<T>(Vec<Option<ArrayEntry<T>>>);

impl<T> GenerationalIndexArray<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Overwrites any entry with the matching index, returns both the GenerationalIndex and T that
    /// were replaced, which may be a GenerationalIndex from a past generation.
    pub fn insert(
        &mut self,
        gen_index: GenerationalIndex,
        value: T,
    ) -> Option<(GenerationalIndex, T)> {
        if gen_index.index >= self.0.len() {
            for _ in self.0.len()..gen_index.index + 1 {
                self.0.push(None);
            }
        }

        let entry = &mut self.0[gen_index.index];

        let old = entry.take().map(|e| {
            (
                GenerationalIndex {
                    index: gen_index.index,
                    generation: e.generation,
                },
                e.value,
            )
        });
        *entry = Some(ArrayEntry {
            value,
            generation: gen_index.generation,
        });
        old
    }

    pub fn remove(&mut self, gen_index: GenerationalIndex) -> Option<T> {
        if gen_index.index < self.0.len() {
            let entry = &mut self.0[gen_index.index];

            if let Some(e) = entry.take() {
                if e.generation == gen_index.generation {
                    return Some(e.value);
                } else {
                    *entry = Some(e);
                }
            }
        }
        None
    }

    pub fn contains_key(&self, gen_index: GenerationalIndex) -> bool {
        self.get(gen_index).is_some()
    }

    pub fn get(&self, gen_index: GenerationalIndex) -> Option<&T> {
        if gen_index.index < self.0.len() {
            self.0[gen_index.index].as_ref().and_then(|e| {
                if e.generation == gen_index.generation {
                    Some(&e.value)
                } else {
                    None
                }
            })
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, gen_index: GenerationalIndex) -> Option<&mut T> {
        if gen_index.index < self.0.len() {
            self.0[gen_index.index].as_mut().and_then(|e| {
                if e.generation == gen_index.generation {
                    Some(&mut e.value)
                } else {
                    None
                }
            })
        } else {
            None
        }
    }

    pub unsafe fn get_unchecked_mut(&mut self, gen_index: GenerationalIndex) -> &mut T {
        self.0[gen_index.index]
            .as_mut()
            .map(|e| &mut e.value)
            .unwrap_unchecked()
    }

    pub fn retain<F: FnMut(GenerationalIndex, &mut T) -> bool>(&mut self, mut f: F) {
        for i in 0..self.0.len() {
            let entry = &mut self.0[i];

            let keep = if let Some(entry) = entry.as_mut() {
                f(
                    GenerationalIndex {
                        index: i,
                        generation: entry.generation,
                    },
                    &mut entry.value,
                )
            } else {
                false
            };

            if !keep {
                *entry = None;
            }
        }
    }

    pub fn filter_map<F: FnMut(GenerationalIndex, T) -> Option<T>>(&mut self, mut f: F) {
        for i in 0..self.0.len() {
            let entry = &mut self.0[i];

            if let Some(e) = entry.take() {
                let gen_index = GenerationalIndex {
                    index: i,
                    generation: e.generation,
                };

                if let Some(value) = f(gen_index, e.value) {
                    *entry = Some(ArrayEntry {
                        value,
                        generation: gen_index.generation,
                    })
                }
            }
        }
    }

    // pub fn iter<'a>(&'a self) -> GenerationalIndexArrayIter<'a, T> {
    //     GenerationalIndexArrayIter(self.0.iter().enumerate())
    // }

    // pub fn iter_mut<'a>(&'a mut self) -> GenerationalIndexArrayIterMut<'a, T> {
    //     GenerationalIndexArrayIterMut(self.0.iter_mut().enumerate())
    // }
}
