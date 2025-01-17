use std::{any::TypeId, marker::PhantomData};

use component::{ComponentSparseSet, ComponentStorage, Components, ComponentsInfo};

pub use component::Component;
use entity::Entity;

pub mod component;
pub mod entity;
pub mod generational_array;
pub mod resources;
pub mod scheduler;
pub mod storage;
pub mod world;

#[derive(Debug)]
pub struct Query<'a, T: 'static> {
    components: &'a ComponentSparseSet,
    current_index: usize,
    component_type: PhantomData<T>,
}

impl<T: 'static> Query<'_, T> {
    pub fn len(&self) -> usize {
        self.components.len()
    }
}

impl<'a, T: Component> Iterator for Query<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.current_index;
        self.current_index += 1;
        self.components.get_dense(index)
    }
}

pub trait WorldQueryable {
    type Item<'new>;
    fn query<'r>(components_info: &ComponentsInfo, components: &'r Components) -> Self::Item<'r>;
}

pub trait WorldQueryableMut {
    fn query(components: &mut Components);
}

impl<T: Component> WorldQueryable for &T {
    type Item<'new> = Query<'new, T>;

    fn query<'r>(components_info: &ComponentsInfo, components: &'r Components) -> Self::Item<'r> {
        let component_info = components_info.get_by_type_id(TypeId::of::<T>()).unwrap();

        Query {
            components: components.get(component_info.id()).unwrap(),
            current_index: 0,
            component_type: PhantomData,
        }
    }
}

impl<T1: Component, T2: Component> WorldQueryable for (&T1, &T2) {
    type Item<'new> = MixedQuery<(QueryWindow<'new, T1>, QueryWindow<'new, T2>)>;

    fn query<'r>(components_info: &ComponentsInfo, components: &'r Components) -> Self::Item<'r> {
        let component_info = components_info.get_by_type_id(TypeId::of::<T1>()).unwrap();
        let mut smallest = components.get(component_info.id()).unwrap();

        let window_1 = QueryWindow {
            component: smallest,
            phantom: PhantomData,
        };
        let component_info = components_info.get_by_type_id(TypeId::of::<T2>()).unwrap();
        let component_set = components.get(component_info.id()).unwrap();
        let window_2 = QueryWindow {
            component: component_set,
            phantom: PhantomData,
        };

        if component_set.len() < smallest.len() {
            smallest = component_set;
        }

        MixedQuery {
            storage: (window_1, window_2),
            indices: smallest.entities(),
            current_index: 0,
        }
    }
}

#[derive(Debug)]
pub struct QueryWindow<'a, T> {
    component: &'a ComponentSparseSet,
    phantom: PhantomData<&'a T>,
}

impl<'a, T: Component> ComponentStorage for QueryWindow<'a, T> {
    type Out = &'a T;

    fn get_data(&self, index: Entity) -> Option<Self::Out> {
        self.component.get(index)
    }
}

impl<'a, T1: ComponentStorage, T2: ComponentStorage> ComponentStorage for (T1, T2) {
    type Out = (T1::Out, T2::Out);

    fn get_data(&self, index: Entity) -> Option<Self::Out> {
        Some((self.0.get_data(index)?, self.1.get_data(index)?))
    }
}

#[derive(Debug)]
pub struct MixedQuery<Storage> {
    storage: Storage,
    indices: Vec<Entity>,
    current_index: usize,
}

impl<Storage: ComponentStorage> Iterator for MixedQuery<Storage> {
    type Item = <Storage as ComponentStorage>::Out;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(&index) = self.indices.get(self.current_index) else {
            return None;
        };
        self.current_index += 1;

        self.storage.get_data(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct SomeComponent(u32);

    impl component::Component for SomeComponent {}

    #[derive(Debug, PartialEq)]
    struct SomeOtherComponent(u32);

    impl component::Component for SomeOtherComponent {}

    #[test]
    fn test_query() {
        let mut world = world::World::new();

        world.register_component::<SomeComponent>();
        world.register_component::<SomeOtherComponent>();

        world.add_entity((SomeComponent(10), SomeOtherComponent(5)));

        world.add_entity(SomeComponent(5));

        let query = world.query::<(&SomeComponent, &SomeOtherComponent)>();

        eprintln!("query: {query:#?}");

        let components: Vec<(&SomeComponent, &SomeOtherComponent)> = query.collect();

        assert_eq!(
            components,
            vec![(&SomeComponent(10), &SomeOtherComponent(5))]
        );

        let query = world.query::<&SomeComponent>();

        eprintln!("query: {query:#?}");

        let components: Vec<&SomeComponent> = query.collect();

        assert_eq!(components, vec![&SomeComponent(10), &SomeComponent(5)]);
    }
}
