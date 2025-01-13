use std::{any::TypeId, marker::PhantomData};

use component::{ComponentSparseSet, Components, ComponentsInfo};

pub use component::Component;

pub mod component;
pub mod entity;
pub mod generational_array;
pub mod resources;
pub mod scheduler;
pub mod storage;
pub mod world;

pub struct Query<'a, T: 'static> {
    components: &'a ComponentSparseSet,
    current_index: usize,
    component_type: PhantomData<T>,
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

impl<'query, T: Component> WorldQueryable for Query<'query, T> {
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

impl<'query, T: Component> WorldQueryable for Query<'query, (T,)> {
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

// impl<T1: Component, T2: Component> WorldQueryable for (&T1, &T2) {
//     type Item<'a> = (&'a T1, &'a T2);

//     fn query<'a>(
//         components_info: &'a ComponentsInfo,
//         components: &'a Components,
//     ) -> Query<'a, Self::Item<'a>> {
//         let component_info1 = components_info.get_by_type_id(TypeId::of::<T1>()).unwrap();
//         let component_info2 = components_info.get_by_type_id(TypeId::of::<T2>()).unwrap();

//         let components1 = components
//             .get(component_info1.id())
//             .map(|component_sparse_set| component_sparse_set.iter())
//             .unwrap_or([].iter());

//         let components2 = components
//             .get(component_info2.id())
//             .map(|component_sparse_set| component_sparse_set.iter())
//             .unwrap_or([].iter());

//         Query {
//             components: Box::new(components1.zip(components2)),
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct SomeComponent(u32);

    impl component::Component for SomeComponent {}

    #[test]
    fn test_query() {
        let mut world = world::World::new();

        world.register_component::<SomeComponent>();

        world.add_entity(SomeComponent(10));

        world.add_entity(SomeComponent(5));

        let components: Vec<&SomeComponent> = world.query::<Query<SomeComponent>>().collect();

        assert_eq!(components, vec![&SomeComponent(10), &SomeComponent(5)]);
    }
}
