use std::marker::PhantomData;

use crate::World;

use super::WorldQueryable;

type StoredSystem = Box<dyn System>;

pub struct Scheduler {
    systems: Vec<StoredSystem>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    pub fn run(&mut self, world: &mut World) {
        for system in &mut self.systems {
            system.run(world);
        }
    }

    pub fn add_system<I, S: System + 'static>(&mut self, system: impl IntoSystem<I, System = S>) {
        self.systems.push(Box::new(system.into_system()));
    }
}

pub trait System {
    fn run(&mut self, world: &mut World);
}

pub struct FunctionSystem<Input, F> {
    f: F,
    // we need a marker because otherwise we're not using `Input`.
    // fn() -> Input is chosen because just using Input would not be `Send` + `Sync`,
    // but the fnptr is always `Send` + `Sync`.
    //
    // Also, this way Input is covariant, but that's not super relevant since we can only deal with
    // static parameters here anyway so there's no subtyping. More info here:
    // https://doc.rust-lang.org/nomicon/subtyping.html
    marker: PhantomData<fn() -> Input>,
}

impl<F: FnMut()> System for FunctionSystem<(), F> {
    fn run(&mut self, world: &mut World) {
        let _ = world;
        (self.f)()
    }
}

impl<F, T: WorldQueryable> System for FunctionSystem<(T,), F>
where
    for<'a, 'b> &'a mut F: FnMut(T) + FnMut(<T as WorldQueryable>::Item<'b>),
{
    fn run(&mut self, world: &mut World) {
        fn call_inner<T>(mut f: impl FnMut(T), _0: T) {
            f(_0)
        }
        let stuff = world.query::<T>();
        call_inner(&mut self.f, stuff);
    }
}

macro_rules! impl_system_tuple {
    ($($params: ident),*) => {
        impl<F: FnMut($($params),*), $($params: 'static),*> System for FunctionSystem<($($params, )*), F> {
            fn run(&mut self, world: &mut World) {
                let _ = world;
                (self.f)()
            }
        }
    };
}

pub trait IntoSystem<Input> {
    type System: System;

    fn into_system(self) -> Self::System;
}

impl<F: FnMut()> IntoSystem<()> for F {
    type System = FunctionSystem<(), Self>;

    fn into_system(self) -> Self::System {
        FunctionSystem {
            f: self,
            marker: Default::default(),
        }
    }
}

impl<F: FnMut(T), T: WorldQueryable> IntoSystem<(T,)> for F
where
    for<'a, 'b> &'a mut F: FnMut(T) + FnMut(<T as WorldQueryable>::Item<'b>),
{
    type System = FunctionSystem<(T,), Self>;

    fn into_system(self) -> Self::System {
        FunctionSystem {
            f: self,
            marker: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ecs::{component::Component, Query};

    use super::*;

    #[derive(Debug)]
    struct SomeComponent(u32);

    impl Component for SomeComponent {}

    fn something(lmao: Query<SomeComponent>) {
        for (i, component) in lmao.enumerate() {
            assert_eq!(component.0, i as u32);
        }
    }

    #[test]
    fn test_systems() {
        let mut world = World::new();
        let mut scheduler = Scheduler::new();

        world.register_component::<SomeComponent>();

        for i in 0..10 {
            world.add_entity(SomeComponent(i));
        }

        scheduler.add_system(something);

        scheduler.run(&mut world);
    }
}
