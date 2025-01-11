use std::{any::TypeId, collections::HashMap};

use super::{
    entity::Entity,
    storage::{
        blob_vec::BlobVec,
        sparse_set::{SparseArray, SparseIndex, SparseSet},
    },
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ComponentId(u32);

impl SparseIndex for ComponentId {
    fn sparse_index(&self) -> usize {
        self.0 as usize
    }

    fn new_sparse_index(value: usize) -> Self {
        ComponentId(value as u32)
    }
}

// sparse: []
// dense: []
//
// *insert FirstComponent(10) for entity 5*
//
// sparse: [None, None, None, None, None, Some(0)]
// dense: [FirstComponent(10)]
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ComponentSparseSet {
    sparse: SparseArray<Entity, usize>,
    entities: Vec<Entity>,
    dense: BlobVec,
}

impl ComponentSparseSet {
    pub fn new<T>() -> Self {
        Self {
            sparse: SparseArray::new(),
            dense: BlobVec::new::<T>(),
            entities: Vec::new(),
        }
    }

    pub fn insert<T>(&mut self, entity: Entity, value: T) {
        self.sparse.insert(entity, self.dense.len());
        self.entities.push(entity);
        unsafe {
            self.dense.push(value);
        }
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        if let Some(dense_index) = self.sparse.remove(entity) {
            unsafe {
                self.dense.swap_remove(dense_index);
            }
            self.entities.swap_remove(dense_index);
            let swapped_entity = self.entities[dense_index];
            eprintln!("swapped_entity: {swapped_entity:?}");
            self.sparse.insert(swapped_entity, dense_index);
        }
    }
}

pub struct Components {
    components: SparseSet<ComponentId, ComponentSparseSet>,
}

impl Components {
    pub fn new() -> Self {
        Self {
            components: SparseSet::new(),
        }
    }

    pub fn get(&self, component_id: ComponentId) -> Option<&ComponentSparseSet> {
        self.components.get(component_id)
    }

    pub fn get_mut(&mut self, component_id: ComponentId) -> Option<&mut ComponentSparseSet> {
        self.components.get_mut(component_id)
    }

    pub fn register_component<T>(&mut self, component_id: ComponentId) {
        self.components
            .insert(component_id, ComponentSparseSet::new::<T>());
    }

    pub fn insert_component<T>(&mut self, entity: Entity, component_id: ComponentId, component: T) {
        self.components
            .get_mut(component_id)
            .unwrap()
            .insert(entity, component);
    }
}

#[derive(Clone)]
pub struct ComponentInfo {
    id: ComponentId,
    // TODO: maybe add type name and other stuff
}

impl ComponentInfo {
    pub fn id(&self) -> ComponentId {
        self.id
    }
}

pub struct ComponentsInfo {
    components: Vec<ComponentInfo>,
    indices: HashMap<TypeId, ComponentId>,
}

impl ComponentsInfo {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            indices: HashMap::new(),
        }
    }

    pub fn register_component<T: 'static>(&mut self) -> ComponentId {
        let type_id = TypeId::of::<T>();
        let component_id = ComponentId((self.components.len() - 1) as u32);
        let component_info = ComponentInfo { id: component_id };
        self.components.push(component_info);
        self.indices.insert(type_id, component_id);

        component_id
    }

    pub fn get<T: 'static>(&self) -> Option<ComponentInfo> {
        self.indices
            .get(&TypeId::of::<T>())
            .map(|index| self.components[index.sparse_index()].clone())
    }

    pub fn get_by_type_id(&self, type_id: TypeId) -> Option<ComponentInfo> {
        self.indices
            .get(&type_id)
            .map(|index| self.components[index.sparse_index()].clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(unused)]
    struct SomeComponent(u32);

    #[test]
    fn test_component_sparse_set_insert() {
        let mut component = ComponentSparseSet::new::<SomeComponent>();

        component.insert(Entity::new_sparse_index(10), SomeComponent(10));

        let mut expected_dense = BlobVec::new::<SomeComponent>();
        unsafe {
            expected_dense.push::<SomeComponent>(SomeComponent(10));
        }

        let expected = ComponentSparseSet {
            sparse: SparseArray {
                values: vec![
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(0),
                ],
                phantom: std::marker::PhantomData,
            },
            entities: vec![Entity::new_sparse_index(10)],
            dense: expected_dense,
        };

        assert_eq!(component, expected);

        component.insert(Entity::new_sparse_index(1), SomeComponent(5));

        let mut expected_dense = BlobVec::new::<SomeComponent>();
        unsafe {
            expected_dense.push::<SomeComponent>(SomeComponent(10));
            expected_dense.push::<SomeComponent>(SomeComponent(5));
        }

        let expected = ComponentSparseSet {
            sparse: SparseArray {
                values: vec![
                    None,
                    Some(1),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(0),
                ],
                phantom: std::marker::PhantomData,
            },
            entities: vec![Entity::new_sparse_index(10), Entity::new_sparse_index(1)],
            dense: expected_dense,
        };

        assert_eq!(component, expected);
    }

    #[test]
    fn test_component_sparse_set_remove() {
        let mut component = ComponentSparseSet::new::<SomeComponent>();

        component.insert(Entity::new_sparse_index(10), SomeComponent(10));
        component.insert(Entity::new_sparse_index(1), SomeComponent(5));

        component.remove_entity(Entity::new_sparse_index(10));

        let mut expected_dense = BlobVec::new::<SomeComponent>();
        unsafe {
            expected_dense.push::<SomeComponent>(SomeComponent(5));
        }

        let expected = ComponentSparseSet {
            sparse: SparseArray {
                values: vec![
                    None,
                    Some(0),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ],
                phantom: std::marker::PhantomData,
            },
            entities: vec![Entity::new_sparse_index(1)],
            dense: expected_dense,
        };

        assert_eq!(component, expected);
    }
}
