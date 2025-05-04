use std::{any::TypeId, marker::PhantomData};

use crate::World;

use super::{entity::Entity, scheduler::SystemParam, Component};

pub trait ComponentAccessor {
    type Output<'new>;

    fn get_component(world: &mut World, entity: Entity) -> Option<Self::Output<'_>>;
    fn entities(world: &mut World) -> Vec<Entity>;
}

pub struct Query<'a, T> {
    world: *mut World,
    entities: Vec<Entity>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> Query<'a, T> {
    pub(crate) fn new(world: &'a mut World, entities: Vec<Entity>) -> Self {
        Self {
            world,
            entities,
            _marker: PhantomData,
        }
    }
}

impl<'a, T: ComponentAccessor> Query<'a, T> {
    pub fn iter(self) -> impl Iterator<Item = (Entity, T::Output<'a>)> + 'a {
        self.entities.into_iter().filter_map(move |entity| unsafe {
            Some((entity, T::get_component(&mut *self.world, entity)?))
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

    fn get_param<'w, 's>(world: &'w mut World, _state: &'s mut Self::State) -> Self::Item<'w, 's> {
        let entities = T::entities(world);
        Query::new(world, entities)
    }
}

impl<T: ComponentAccessor + 'static> ComponentAccessor for Query<'_, T> {
    type Output<'new> = T::Output<'new>;

    fn get_component(world: &mut World, entity: Entity) -> Option<Self::Output<'_>> {
        T::get_component(world, entity)
    }

    fn entities(world: &mut World) -> Vec<Entity> {
        T::entities(world)
    }
}

pub struct Read<T>(PhantomData<T>);

impl<T: Component> ComponentAccessor for Read<T> {
    type Output<'new> = &'new T;

    fn get_component(world: &mut World, entity: Entity) -> Option<Self::Output<'_>> {
        let component_info = world
            .components_info
            .get_by_type_id(TypeId::of::<T>())
            .unwrap();

        world
            .components
            .get(component_info.id())
            .and_then(|c| c.get(entity))
    }

    fn entities(world: &mut World) -> Vec<Entity> {
        let component_info = world
            .components_info
            .get_by_type_id(TypeId::of::<T>())
            .unwrap();

        world
            .components
            .get(component_info.id())
            .map(|c| c.entities())
            .unwrap_or_default()
    }
}

pub struct Write<T>(PhantomData<T>);

impl<T: Component> ComponentAccessor for Write<T> {
    type Output<'new> = &'new mut T;

    fn get_component(world: &mut World, entity: Entity) -> Option<Self::Output<'_>> {
        let component_info = world
            .components_info
            .get_by_type_id(TypeId::of::<T>())
            .unwrap();

        world
            .components
            .get_mut(component_info.id())
            .and_then(|c| c.get_mut(entity))
    }

    fn entities(world: &mut World) -> Vec<Entity> {
        let component_info = world
            .components_info
            .get_by_type_id(TypeId::of::<T>())
            .unwrap();

        world
            .components
            .get(component_info.id())
            .map(|c| c.entities())
            .unwrap_or_default()
    }
}

impl<A: ComponentAccessor, B: ComponentAccessor> ComponentAccessor for (A, B) {
    type Output<'new> = (A::Output<'new>, B::Output<'new>);

    fn get_component(world: &mut World, entity: Entity) -> Option<Self::Output<'_>> {
        unsafe {
            let world_ptr = world as *mut World;

            let a_component = A::get_component(&mut *world_ptr, entity)?;
            let b_component = B::get_component(&mut *world_ptr, entity)?;

            Some((a_component, b_component))
        }
    }

    fn entities(world: &mut World) -> Vec<Entity> {
        unsafe {
            let world_ptr = world as *mut World;
            let entities_a = A::entities(&mut *world_ptr);
            let entities_b = B::entities(&mut *world_ptr);

            entities_a
                .into_iter()
                .filter(|e| entities_b.contains(e))
                .collect()
        }
    }
}

impl<A: ComponentAccessor, B: ComponentAccessor, C: ComponentAccessor> ComponentAccessor
    for (A, B, C)
{
    type Output<'new> = (A::Output<'new>, B::Output<'new>, C::Output<'new>);

    fn get_component(world: &mut World, entity: Entity) -> Option<Self::Output<'_>> {
        unsafe {
            let world_ptr = world as *mut World;

            let a_component = A::get_component(&mut *world_ptr, entity)?;
            let b_component = B::get_component(&mut *world_ptr, entity)?;
            let c_component = C::get_component(&mut *world_ptr, entity)?;

            Some((a_component, b_component, c_component))
        }
    }

    fn entities(world: &mut World) -> Vec<Entity> {
        unsafe {
            let world_ptr = world as *mut World;
            let entities_a = A::entities(&mut *world_ptr);
            let entities_b = B::entities(&mut *world_ptr);
            let entities_c = C::entities(&mut *world_ptr);

            entities_a
                .into_iter()
                .filter(|e| entities_b.contains(e) && entities_c.contains(e))
                .collect()
        }
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
}
