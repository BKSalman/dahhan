use std::marker::PhantomData;

use crate::{World, ecs::world::UnsafeWorldCell};

use super::{
    Component,
    entity::Entity,
    scheduler::{Access, SystemParam},
};

pub trait ComponentAccessor {
    type Output<'new>;

    unsafe fn get_component<'w>(
        world: UnsafeWorldCell<'w>,
        entity: Entity,
    ) -> Option<Self::Output<'w>>;

    fn init_access(world: &mut World, access: &mut Access);

    fn entities(world: UnsafeWorldCell<'_>) -> Vec<Entity>;
}

pub struct Query<'w, T> {
    world: UnsafeWorldCell<'w>,
    entities: Vec<Entity>,
    _marker: PhantomData<&'w T>,
}

impl<'w, T> Query<'w, T> {
    pub(crate) fn new(world: UnsafeWorldCell<'w>, entities: Vec<Entity>) -> Self {
        Self {
            world,
            entities,
            _marker: PhantomData,
        }
    }
}

impl<'w, T: ComponentAccessor> Query<'w, T> {
    pub fn iter(self) -> impl Iterator<Item = (Entity, T::Output<'w>)> + 'w {
        self.entities.into_iter().filter_map(move |entity| unsafe {
            Some((entity, T::get_component(self.world, entity)?))
        })
    }
}

impl<T: ComponentAccessor + 'static> SystemParam for Query<'_, T> {
    type State = ();
    type Item<'w, 's> = Query<'w, T>;

    fn init_state(world: &mut World) -> Self::State {
        let _ = world;
        ()
    }

    fn init_access(world: &mut World, access: &mut Access) {
        let mut query_access = Access::new();
        T::init_access(world, &mut query_access);

        if let Err(conflict) = query_access.validate() {
            panic!("{conflict}");
        }

        access.extend(query_access);
    }

    unsafe fn get_param<'w, 's>(
        world: UnsafeWorldCell<'w>,
        _state: &'s mut Self::State,
    ) -> Self::Item<'w, 's> {
        let entities = T::entities(world);
        Query::new(world, entities)
    }
}

pub struct Read<T>(PhantomData<T>);

impl<T: Component> ComponentAccessor for Read<T> {
    type Output<'new> = &'new T;

    unsafe fn get_component<'w>(
        world: UnsafeWorldCell<'w>,
        entity: Entity,
    ) -> Option<Self::Output<'w>> {
        unsafe { world.get_component(entity) }
    }

    fn init_access(world: &mut World, access: &mut Access) {
        let component_info = world
            .components_info
            .get::<T>()
            .expect("should be regisetered");

        access.add_read(component_info.id());
    }

    fn entities(world: UnsafeWorldCell<'_>) -> Vec<Entity> {
        world.entities::<T>()
    }
}

pub struct Write<T>(PhantomData<T>);

impl<T: Component> ComponentAccessor for Write<T> {
    type Output<'new> = &'new mut T;

    unsafe fn get_component<'w>(
        world: UnsafeWorldCell<'w>,
        entity: Entity,
    ) -> Option<Self::Output<'w>> {
        unsafe { world.get_component_mut(entity) }
    }

    fn init_access(world: &mut World, access: &mut Access) {
        let component_info = world
            .components_info
            .get::<T>()
            .expect("should be regisetered");

        access.add_write(component_info.id());
    }

    fn entities(world: UnsafeWorldCell<'_>) -> Vec<Entity> {
        world.entities::<T>()
    }
}

impl<A: ComponentAccessor, B: ComponentAccessor> ComponentAccessor for (A, B) {
    type Output<'new> = (A::Output<'new>, B::Output<'new>);

    unsafe fn get_component<'w>(
        world: UnsafeWorldCell<'w>,
        entity: Entity,
    ) -> Option<Self::Output<'w>> {
        unsafe {
            let a_component = A::get_component(world, entity)?;
            let b_component = B::get_component(world, entity)?;

            Some((a_component, b_component))
        }
    }

    fn init_access(world: &mut World, access: &mut Access) {
        A::init_access(world, access);
        B::init_access(world, access);
    }

    fn entities(world: UnsafeWorldCell<'_>) -> Vec<Entity> {
        let entities_a = A::entities(world);
        let entities_b = B::entities(world);

        entities_a
            .into_iter()
            .filter(|e| entities_b.contains(e))
            .collect()
    }
}

impl<A: ComponentAccessor, B: ComponentAccessor, C: ComponentAccessor> ComponentAccessor
    for (A, B, C)
{
    type Output<'new> = (A::Output<'new>, B::Output<'new>, C::Output<'new>);

    unsafe fn get_component<'w>(
        world: UnsafeWorldCell<'w>,
        entity: Entity,
    ) -> Option<Self::Output<'w>> {
        unsafe {
            let a_component = A::get_component(world, entity)?;
            let b_component = B::get_component(world, entity)?;
            let c_component = C::get_component(world, entity)?;

            Some((a_component, b_component, c_component))
        }
    }

    fn init_access(world: &mut World, access: &mut Access) {
        A::init_access(world, access);
        B::init_access(world, access);
        C::init_access(world, access);
    }

    fn entities(world: UnsafeWorldCell<'_>) -> Vec<Entity> {
        let entities_a = A::entities(world);
        let entities_b = B::entities(world);
        let entities_c = C::entities(world);

        entities_a
            .into_iter()
            .filter(|e| entities_b.contains(e) && entities_c.contains(e))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct SomeComponent(u32);

    impl Component for SomeComponent {}

    #[derive(Debug, PartialEq)]
    struct SomeOtherComponent(u32);

    impl Component for SomeOtherComponent {}

    #[test]
    fn test_single_read_query() {
        let mut world = World::new();

        world.register_component::<SomeComponent>();

        let entity = world.add_entity(());

        world.add_component(entity, SomeComponent(10));

        let query = world.query::<Read<SomeComponent>>();

        assert_eq!(Some((entity, &SomeComponent(10))), query.iter().next());
    }

    #[test]
    fn test_single_write_query() {
        let mut world = World::new();

        world.register_component::<SomeComponent>();

        let entity = world.add_entity(());

        world.add_component(entity, SomeComponent(10));

        let query = world.query::<Write<SomeComponent>>();

        assert_eq!(Some((entity, &mut SomeComponent(10))), query.iter().next());
    }

    #[test]
    fn test_two_read_query() {
        let mut world = World::new();

        world.register_component::<SomeComponent>();
        world.register_component::<SomeOtherComponent>();

        let entity = world.add_entity(());

        world.add_component(entity, SomeComponent(10));

        world.add_component(entity, SomeOtherComponent(10));

        let query = world.query::<(Read<SomeComponent>, Read<SomeOtherComponent>)>();

        assert_eq!(
            Some((entity, (&SomeComponent(10), &SomeOtherComponent(10)))),
            query.iter().next()
        );
    }

    #[test]
    fn test_single_read_single_write_query() {
        let mut world = World::new();

        world.register_component::<SomeComponent>();
        world.register_component::<SomeOtherComponent>();

        let entity = world.add_entity(());

        world.add_component(entity, SomeComponent(10));

        world.add_component(entity, SomeOtherComponent(10));

        let query = world.query::<(Read<SomeComponent>, Write<SomeOtherComponent>)>();

        assert_eq!(
            Some((entity, (&SomeComponent(10), &mut SomeOtherComponent(10)))),
            query.iter().next()
        );
    }

    #[test]
    fn test_concurrent_write_different_components_soundness() {
        let mut world = World::new();

        world.register_component::<SomeComponent>();
        world.register_component::<SomeOtherComponent>();

        let entity = world.add_entity(());

        world.add_component(entity, SomeComponent(10));
        world.add_component(entity, SomeOtherComponent(20));

        let cell = world.as_unsafe_world_cell();

        unsafe {
            let a = cell.get_component_mut::<SomeComponent>(entity).unwrap();
            let b = cell
                .get_component_mut::<SomeOtherComponent>(entity)
                .unwrap();

            a.0 += 1;
            b.0 += 1;

            assert_eq!(a.0, 11);
            assert_eq!(b.0, 21);
        }
    }

    #[test]
    fn test_concurrent_read_and_write_different_components_soundness() {
        let mut world = World::new();

        world.register_component::<SomeComponent>();
        world.register_component::<SomeOtherComponent>();

        let entity = world.add_entity(());

        world.add_component(entity, SomeComponent(10));
        world.add_component(entity, SomeOtherComponent(20));

        let cell = world.as_unsafe_world_cell();

        unsafe {
            let a = cell.get_component::<SomeComponent>(entity).unwrap();
            let b = cell
                .get_component_mut::<SomeOtherComponent>(entity)
                .unwrap();

            b.0 += 1;

            assert_eq!(a.0, 10);
            assert_eq!(b.0, 21);
        }
    }

    #[test]
    fn test_write_query_multiple_entities_soundness() {
        let mut world = World::new();

        world.register_component::<SomeComponent>();

        let e1 = world.add_entity(());
        let e2 = world.add_entity(());

        world.add_component(e1, SomeComponent(1));
        world.add_component(e2, SomeComponent(2));

        let query = world.query::<Write<SomeComponent>>();
        let mut results: Vec<(Entity, &mut SomeComponent)> = query.iter().collect();

        results[0].1.0 += 100;
        results[1].1.0 += 100;

        assert_eq!(results[0].1.0, 101);
        assert_eq!(results[1].1.0, 102);
    }

    #[test]
    #[should_panic]
    fn test_access_validation_same_component_multiple_writes() {
        let mut world = World::new();

        world.register_component::<SomeComponent>();

        let e1 = world.add_entity(());

        world.add_component(e1, SomeComponent(1));

        let _ = world.query::<(Write<SomeComponent>, Write<SomeComponent>)>();
    }

    #[test]
    #[should_panic]
    fn test_access_validation_same_component_write_read() {
        let mut world = World::new();

        world.register_component::<SomeComponent>();

        let e1 = world.add_entity(());

        world.add_component(e1, SomeComponent(1));

        let _ = world.query::<(Write<SomeComponent>, Read<SomeComponent>)>();
    }
}
