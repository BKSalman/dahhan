use std::{
    any::{Any, TypeId},
    collections::HashMap,
    slice::SliceIndex,
};

use crate::prelude::{Local, Res, ResMut};

use super::scheduler::SystemParam;

pub trait Event: Send + Sync + 'static {}

pub struct EventQueue {
    events: HashMap<TypeId, Vec<Box<dyn Any + Send + Sync>>>,
}

impl EventQueue {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
        }
    }

    /// Add an event to the queue
    pub fn send<E: Event>(&mut self, event: E) {
        let type_id = TypeId::of::<E>();
        let events = self.events.entry(type_id).or_insert_with(Vec::new);
        events.push(Box::new(event));
    }

    /// Get all events of a specific type
    pub fn get<E: Event>(&self) -> Vec<&E> {
        let type_id = TypeId::of::<E>();
        if let Some(events) = self.events.get(&type_id) {
            events
                .iter()
                .filter_map(|event| event.downcast_ref::<E>())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get event count for a specific type
    pub fn get_event_count<E: Event>(&self) -> usize {
        let type_id = TypeId::of::<E>();
        self.events.get(&type_id).map_or(0, |events| events.len())
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Clear events of a specific type
    pub fn clear_events<E: Event>(&mut self) {
        let type_id = TypeId::of::<E>();
        self.events.remove(&type_id);
    }
}

pub struct Events<E> {
    events: Vec<E>,
}

impl<E> Events<E> {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn get<I: SliceIndex<[E]>>(&self, index: I) -> Option<&<I as SliceIndex<[E]>>::Output> {
        self.events.get(index)
    }

    pub fn send(&mut self, event: E) {
        self.events.push(event);
    }
}

pub struct EventReader<'s, 'w, E> {
    cursor: Local<'s, usize>,
    events: Res<'w, Events<E>>,
}

impl<E: 'static> SystemParam for EventReader<'_, '_, E> {
    type State = <(Local<'static, usize>, Res<'static, Events<E>>) as SystemParam>::State;

    type Item<'world, 'state> = EventReader<'state, 'world, E>;

    fn init_state(world: &mut super::world::World) -> Self::State {
        <(Local<'static, usize>, Res<'static, Events<E>>) as SystemParam>::init_state(world)
    }

    fn get_param<'w, 's>(
        world: &'w mut super::world::World,
        state: &'s mut Self::State,
    ) -> Self::Item<'w, 's> {
        let (cursor, events) =
            <(Local<'static, usize>, Res<'static, Events<E>>) as SystemParam>::get_param(
                world, state,
            );

        EventReader { cursor, events }
    }
}

impl<E: Copy> Iterator for EventReader<'_, '_, E> {
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        self.events.get(*self.cursor).copied().and_then(|e| {
            *self.cursor += 1;
            Some(e)
        })
    }
}

impl<E> EventReader<'_, '_, E> {}

pub struct EventWriter<'w, E> {
    events: ResMut<'w, Events<E>>,
}

impl<E: 'static> SystemParam for EventWriter<'_, E> {
    type State = <ResMut<'static, Events<E>> as SystemParam>::State;

    type Item<'world, 'state> = EventWriter<'world, E>;

    fn init_state(world: &mut super::world::World) -> Self::State {
        <ResMut<'static, Events<E>> as SystemParam>::init_state(world)
    }

    fn get_param<'w, 's>(
        world: &'w mut super::world::World,
        state: &'s mut Self::State,
    ) -> Self::Item<'w, 's> {
        let events = <ResMut<'static, Events<E>> as SystemParam>::get_param(world, state);

        EventWriter { events }
    }
}

impl<'a, E> EventWriter<'a, E> {
    pub fn send(&mut self, event: E) {
        self.events.send(event);
    }
}

#[cfg(test)]
mod tests {
    use crate::State;

    use super::*;

    fn read_events(mut events: EventReader<u32>) {
        assert_eq!(Some(u32::MAX), events.next());
        assert_eq!(Some(u32::MAX / 2), events.next());
        assert_eq!(Some(0u32), events.next());
        assert_eq!(None, events.next());
    }

    #[test]
    fn test_event_reading() {
        let mut state = State::new();

        state.world.add_event::<u32>();

        state.world.send_event(u32::MAX);
        state.world.send_event(u32::MAX / 2);
        state.world.send_event(0u32);

        state.scheduler.add_system(read_events);

        state.initialize();

        state.scheduler.run(&mut state.world);
    }
}
