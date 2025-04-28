use std::{
    any::Any,
    ops::{Deref, DerefMut},
    slice::SliceIndex,
};

use crate::prelude::{Local, Res, ResMut};

use super::{scheduler::SystemParam, world::World};

pub trait Event: Any + Send + Sync + 'static {}

pub struct EventRegistry {
    update_functions: Vec<Box<dyn Fn(&mut World)>>,
}

impl EventRegistry {
    pub fn new() -> Self {
        Self {
            update_functions: Vec::new(),
        }
    }

    pub fn register_event<E: Event>(&mut self) {
        self.update_functions.push(Box::new(|world: &mut World| {
            if let Ok(mut events) = world.write_resource::<Events<E>>() {
                events.update();
            }
        }));
    }

    pub fn update_events(&self, world: &mut World) {
        for event_update in &self.update_functions {
            event_update(world);
        }
    }
}

#[derive(Debug)]
pub(crate) struct EventSequence<E: Event> {
    pub(crate) events: Vec<E>,
    pub(crate) start_event_count: usize,
}

// Derived Default impl would incorrectly require E: Default
impl<E: Event> Default for EventSequence<E> {
    fn default() -> Self {
        Self {
            events: Default::default(),
            start_event_count: Default::default(),
        }
    }
}

impl<E: Event> Deref for EventSequence<E> {
    type Target = Vec<E>;

    fn deref(&self) -> &Self::Target {
        &self.events
    }
}

impl<E: Event> DerefMut for EventSequence<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.events
    }
}

#[derive(Debug)]
pub struct Events<E: Event> {
    events_a: EventSequence<E>,
    events_b: EventSequence<E>,
    event_count: usize,
}

impl<E: Event> Events<E> {
    pub fn new() -> Self {
        Self {
            events_a: EventSequence::default(),
            events_b: EventSequence::default(),
            event_count: 0,
        }
    }

    pub fn get<I: SliceIndex<[E]>>(&self, index: I) -> Option<&<I as SliceIndex<[E]>>::Output> {
        self.events_b.get(index)
    }

    pub fn send(&mut self, event: E) {
        self.events_b.push(event);
        self.event_count += 1;
    }

    pub fn update(&mut self) {
        core::mem::swap(&mut self.events_a, &mut self.events_b);
        self.events_b.clear();
        self.events_b.start_event_count = self.event_count;
        debug_assert_eq!(
            self.events_a.start_event_count + self.events_a.len(),
            self.events_b.start_event_count
        );
    }

    pub fn len(&self) -> usize {
        self.events_b.len() + self.events_a.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events_b.is_empty() && self.events_a.is_empty()
    }
}

/// An iterator that yields any unread events (and their IDs) from an [`EventReader`](super::EventReader) or [`EventCursor`].
#[derive(Debug)]
pub struct EventIterator<'a, E: Event> {
    cursor: &'a mut usize,
    chain: std::iter::Chain<core::slice::Iter<'a, E>, core::slice::Iter<'a, E>>,
    unread: usize,
}

impl<'a, E: Event> EventIterator<'a, E> {
    /// Creates a new iterator that yields any `events` that have not yet been seen by `reader`.
    pub fn new(cursor: &'a mut usize, events: &'a Events<E>) -> Self {
        let a_index = cursor.saturating_sub(events.events_a.start_event_count);
        let b_index = cursor.saturating_sub(events.events_b.start_event_count);
        let a = events.events_a.get(a_index..).unwrap_or_default();
        let b = events.events_b.get(b_index..).unwrap_or_default();

        let unread_count = a.len() + b.len();
        // Ensure `len` is implemented correctly
        debug_assert_eq!(
            unread_count,
            events.event_count.saturating_sub(*cursor).min(events.len())
        );

        *cursor = events.event_count - unread_count;
        // Iterate the oldest first, then the newer events
        let chain = a.iter().chain(b.iter());

        Self {
            cursor,
            chain,
            unread: unread_count,
        }
    }
}

impl<'a, E: Event> Iterator for EventIterator<'a, E> {
    type Item = &'a E;
    fn next(&mut self) -> Option<Self::Item> {
        match self.chain.next().map(|instance| instance) {
            Some(item) => {
                *self.cursor += 1;
                self.unread -= 1;
                Some(item)
            }
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.chain.size_hint()
    }

    fn count(self) -> usize {
        *self.cursor += self.unread;
        self.unread
    }

    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let event = self.chain.last()?;
        *self.cursor += self.unread;
        Some(event)
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if let Some(event) = self.chain.nth(n) {
            *self.cursor += n + 1;
            self.unread -= n + 1;
            Some(event)
        } else {
            *self.cursor += self.unread;
            self.unread = 0;
            None
        }
    }
}

pub struct EventReader<'s, 'w, E: Event> {
    cursor: Local<'s, usize>,
    events: Res<'w, Events<E>>,
}

impl<E: Event> EventReader<'_, '_, E> {
    pub fn read(&mut self) -> EventIterator<'_, E> {
        EventIterator::new(&mut self.cursor, &self.events)
    }
}

impl<E: Event> SystemParam for EventReader<'_, '_, E> {
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

pub struct EventWriter<'w, E: Event> {
    events: ResMut<'w, Events<E>>,
}

impl<E: Event> SystemParam for EventWriter<'_, E> {
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

impl<'a, E: Event> EventWriter<'a, E> {
    pub fn send(&mut self, event: E) {
        self.events.send(event);
    }
}

#[cfg(test)]
mod tests {
    use crate::State;

    use super::*;

    #[derive(Debug, Copy, Clone, PartialEq)]
    struct SomeEvent(u32);

    impl Event for SomeEvent {}

    fn read_events(mut events: EventReader<SomeEvent>) {
        let mut events = events.read();
        assert_eq!(Some(&SomeEvent(u32::MAX)), events.next());
        assert_eq!(Some(&SomeEvent(u32::MAX / 2)), events.next());
        assert_eq!(Some(&SomeEvent(0u32)), events.next());
        assert_eq!(None, events.next());
    }

    #[test]
    fn test_event_reading() {
        let mut state = State::new();

        state.world.add_event::<SomeEvent>();

        state.world.send_event(SomeEvent(u32::MAX));
        state.world.send_event(SomeEvent(u32::MAX / 2));
        state.world.send_event(SomeEvent(0u32));

        state.scheduler.add_system(read_events);

        state.initialize();

        state.scheduler.run(&mut state.world);
    }
}
