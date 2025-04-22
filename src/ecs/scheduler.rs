use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

use crate::World;

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

pub trait SystemParam {
    type Item<'w>;

    fn fetch(world: &mut World) -> Self::Item<'_>;
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

impl<F, T: SystemParam> System for FunctionSystem<(T,), F>
where
    for<'a, 'b> &'a mut F: FnMut(T) + FnMut(<T as SystemParam>::Item<'b>),
{
    fn run(&mut self, world: &mut World) {
        fn call_inner<T>(mut f: impl FnMut(T), _0: T) {
            f(_0)
        }
        let stuff = T::fetch(world);
        call_inner(&mut self.f, stuff);
    }
}

impl<F, T1: SystemParam, T2: SystemParam> System for FunctionSystem<(T1, T2), F>
where
    // for any two arbitrary lifetimes, a mutable reference to F with lifetime 'a
    // implements FnMut taking parameters of lifetime 'b
    for<'a, 'b> &'a mut F:
        FnMut(T1, T2) + FnMut(<T1 as SystemParam>::Item<'b>, <T2 as SystemParam>::Item<'b>),
{
    fn run(&mut self, world: &mut World) {
        fn call_inner<T1, T2>(mut f: impl FnMut(T1, T2), _0: T1, _1: T2) {
            f(_0, _1)
        }

        // SAFETY: We're creating two mutable references to world, but we ensure
        // they're used in a non-overlapping way. Each parameter fetch accesses
        // different parts of the world, and we don't reuse the pointers after
        // the function call.
        unsafe {
            let world_ptr = world as *mut World;
            let param1 = T1::fetch(&mut *world_ptr);
            let param2 = T2::fetch(&mut *world_ptr);
            call_inner(&mut self.f, param1, param2);
        }
    }
}

impl<F, T1: SystemParam, T2: SystemParam, T3: SystemParam> System
    for FunctionSystem<(T1, T2, T3), F>
where
    // for any two arbitrary lifetimes, a mutable reference to F with lifetime 'a
    // implements FnMut taking parameters of lifetime 'b
    for<'a, 'b> &'a mut F: FnMut(T1, T2, T3)
        + FnMut(
            <T1 as SystemParam>::Item<'b>,
            <T2 as SystemParam>::Item<'b>,
            <T3 as SystemParam>::Item<'b>,
        ),
{
    fn run(&mut self, world: &mut World) {
        fn call_inner<T1, T2, T3>(mut f: impl FnMut(T1, T2, T3), _0: T1, _1: T2, _2: T3) {
            f(_0, _1, _2)
        }

        // SAFETY: We're creating two mutable references to world, but we ensure
        // they're used in a non-overlapping way. Each parameter fetch accesses
        // different parts of the world, and we don't reuse the pointers after
        // the function call.
        unsafe {
            let world_ptr = world as *mut World;
            let param1 = T1::fetch(&mut *world_ptr);
            let param2 = T2::fetch(&mut *world_ptr);
            let param3 = T3::fetch(&mut *world_ptr);
            call_inner(&mut self.f, param1, param2, param3);
        }
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

impl<F: FnMut(T), T: SystemParam> IntoSystem<(T,)> for F
where
    for<'a, 'b> &'a mut F: FnMut(T) + FnMut(<T as SystemParam>::Item<'b>),
{
    type System = FunctionSystem<(T,), Self>;

    fn into_system(self) -> Self::System {
        FunctionSystem {
            f: self,
            marker: Default::default(),
        }
    }
}

impl<F: FnMut(T1, T2), T1: SystemParam, T2: SystemParam> IntoSystem<(T1, T2)> for F
where
    for<'a, 'b> &'a mut F:
        FnMut(T1, T2) + FnMut(<T1 as SystemParam>::Item<'b>, <T2 as SystemParam>::Item<'b>),
{
    type System = FunctionSystem<(T1, T2), Self>;

    fn into_system(self) -> Self::System {
        FunctionSystem {
            f: self,
            marker: Default::default(),
        }
    }
}

impl<F: FnMut(T1, T2, T3), T1: SystemParam, T2: SystemParam, T3: SystemParam>
    IntoSystem<(T1, T2, T3)> for F
where
    for<'a, 'b> &'a mut F: FnMut(T1, T2, T3)
        + FnMut(
            <T1 as SystemParam>::Item<'b>,
            <T2 as SystemParam>::Item<'b>,
            <T3 as SystemParam>::Item<'b>,
        ),
{
    type System = FunctionSystem<(T1, T2, T3), Self>;

    fn into_system(self) -> Self::System {
        FunctionSystem {
            f: self,
            marker: Default::default(),
        }
    }
}

pub struct Res<'a, T>(RwLockReadGuard<'a, T>);

impl<'a, T: 'static> SystemParam for Res<'a, T> {
    type Item<'w> = Res<'w, T>;

    fn fetch(world: &mut World) -> Self::Item<'_> {
        Res(world.read_resource::<T>().expect("Resource not found"))
    }
}

impl<'a, T> Deref for Res<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct ResMut<'a, T>(RwLockWriteGuard<'a, T>);

impl<'a, T> Deref for ResMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> DerefMut for ResMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T: 'static> SystemParam for ResMut<'a, T> {
    type Item<'w> = ResMut<'w, T>;

    fn fetch(world: &mut World) -> Self::Item<'_> {
        ResMut(world.write_resource::<T>().expect("Resource not found"))
    }
}

#[cfg(test)]
mod tests {
    use crate::ecs::{
        component::Component,
        query::{Query, Read},
    };

    use super::*;

    #[derive(Debug)]
    struct SomeComponent(u32);

    impl Component for SomeComponent {}

    fn something(lmao: Query<Read<SomeComponent>>) {
        assert!(lmao.iter().count() == 10);
    }

    fn something_else(lmao: Query<Read<SomeComponent>>) {
        for (i, (_e, component)) in lmao.iter().enumerate() {
            assert_eq!(component.0, i as u32);
        }
    }

    fn panic() {
        panic!("hello");
    }

    #[test]
    #[should_panic]
    fn test_systems_work() {
        let mut world = World::new();
        let mut scheduler = Scheduler::new();

        scheduler.add_system(panic);

        scheduler.run(&mut world);
    }

    #[test]
    fn test_systems_query_component() {
        let mut world = World::new();
        let mut scheduler = Scheduler::new();

        world.register_component::<SomeComponent>();

        for i in 0..10 {
            world.add_entity(SomeComponent(i));
        }

        scheduler.add_system(something);

        scheduler.add_system(something_else);

        scheduler.run(&mut world);
    }
}
