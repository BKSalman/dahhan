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

pub trait ComponentStorage {
    type Out;

    fn get_data(&self, index: Entity) -> Option<Self::Out>;
}

// sparse: []
// dense: []
//
// *insert FirstComponent(10) for entity 5*
//
// sparse: [None, None, None, None, None, Some(0)]
// dense: [FirstComponent(10)]
/// A set that holds a single type of component for multiple entities
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

    pub fn get<T>(&self, entity: Entity) -> Option<&T> {
        let dense_index = self.sparse.get(entity)?;
        // eprintln!("dense index: {dense_index}");
        unsafe { self.dense.get(*dense_index) }
    }

    pub fn get_mut<T>(&mut self, entity: Entity) -> Option<&mut T> {
        let dense_index = self.sparse.get(entity)?;
        // eprintln!("dense index: {dense_index}");
        unsafe { self.dense.get_mut(*dense_index) }
    }

    pub fn get_dense<T>(&self, dense_index: usize) -> Option<&T> {
        unsafe { self.dense.get(dense_index) }
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        if let Some(dense_index) = self.sparse.remove(entity) {
            unsafe {
                self.dense.swap_remove(dense_index);
            }
            self.entities.swap_remove(dense_index);
            let swapped_entity = self.entities[dense_index];
            self.sparse.insert(swapped_entity, dense_index);
        }
    }

    pub fn iter<T>(&self) -> std::slice::Iter<'_, T> {
        unsafe { self.dense.iter() }
    }

    pub fn iter_mut<T>(&mut self) -> std::slice::IterMut<'_, T> {
        unsafe { self.dense.iter_mut() }
    }

    /// Returns the how many entities have this component
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.len() == 0
    }

    pub fn entities(&self) -> Vec<Entity> {
        self.entities.to_vec()
    }
}

#[cfg_attr(test, derive(Debug))]
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

    pub fn has_component(&self, component_id: ComponentId, entity: Entity) -> bool {
        self.components
            .get(component_id)
            .is_some_and(|c| c.entities.contains(&entity))
    }

    pub fn entities(&self, component_id: ComponentId) -> Vec<Entity> {
        self.components
            .get(component_id)
            .map(|c| c.entities.clone())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct ComponentInfo {
    id: ComponentId,
    // TODO: maybe add type name and other stuff
}

impl ComponentInfo {
    pub fn id(&self) -> ComponentId {
        self.id
    }
}

#[derive(Debug)]
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
        let component_id = ComponentId((self.components.len()) as u32);
        let component_info = ComponentInfo { id: component_id };
        self.components.push(component_info);
        self.indices.insert(type_id, component_id);

        component_id
    }

    pub fn get<T: 'static>(&self) -> Option<ComponentInfo> {
        self.get_by_type_id(TypeId::of::<T>())
    }

    pub fn get_by_type_id(&self, type_id: TypeId) -> Option<ComponentInfo> {
        self.indices
            .get(&type_id)
            .map(|index| self.components[index.sparse_index()].clone())
    }
}

pub trait Component: 'static {}

pub trait TupleAddComponent {
    fn add_component(
        self,
        components_info: &ComponentsInfo,
        components: &mut Components,
        entity: Entity,
    );
}

impl TupleAddComponent for () {
    fn add_component(
        self,
        components_info: &ComponentsInfo,
        components: &mut Components,
        entity: Entity,
    ) {
        let _ = components_info;
        let _ = components;
        let _ = entity;
    }
}

impl<T: Component> TupleAddComponent for T {
    fn add_component(
        self,
        components_info: &ComponentsInfo,
        components: &mut Components,
        entity: Entity,
    ) {
        let component_info = components_info.get::<T>().unwrap();
        components.insert_component(entity, component_info.id(), self);
    }
}

impl<T1: Component> TupleAddComponent for (T1,) {
    fn add_component(
        self,
        components_info: &ComponentsInfo,
        components: &mut Components,
        entity: Entity,
    ) {
        let component_info = components_info.get::<T1>().unwrap();
        components.insert_component(entity, component_info.id(), self);
    }
}

impl<T1: Component, T2: Component> TupleAddComponent for (T1, T2) {
    fn add_component(
        self,
        components_info: &ComponentsInfo,
        components: &mut Components,
        entity: Entity,
    ) {
        self.0.add_component(components_info, components, entity);
        self.1.add_component(components_info, components, entity);
    }
}

impl<T1: Component, T2: Component, T3: Component> TupleAddComponent for (T1, T2, T3) {
    fn add_component(
        self,
        components_info: &ComponentsInfo,
        components: &mut Components,
        entity: Entity,
    ) {
        self.0.add_component(components_info, components, entity);
        self.1.add_component(components_info, components, entity);
        self.2.add_component(components_info, components, entity);
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
