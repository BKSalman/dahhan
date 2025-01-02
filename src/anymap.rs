use std::any::{Any, TypeId};

use rustc_hash::FxHashMap;

pub struct AnyMap {
    map: FxHashMap<TypeId, Box<dyn Any + 'static>>,
}

impl AnyMap {
    pub fn new() -> Self {
        Self {
            map: FxHashMap::default(),
        }
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|any| any.downcast_ref::<T>())
    }

    pub fn insert<T: 'static>(&mut self, value: T) {
        self.map.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        self.map
            .remove(&TypeId::of::<T>())
            .and_then(|any| any.downcast::<T>().ok())
            .map(|b: Box<T>| *b)
    }
}
