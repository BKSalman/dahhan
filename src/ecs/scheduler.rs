use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

use crate::World;

type StoredSystem = Box<dyn System>;

pub trait SystemParam {
    // TODO: when supporting multithreading (probably not soon) add `Sync` and `Send`
    /// Used to store data which persists across invocations of a system.
    type State: 'static;

    type Item<'world, 'state>: SystemParam<State = Self::State>;

    fn init_state(world: &mut World) -> Self::State;

    fn get_param<'w, 's>(world: &'w mut World, state: &'s mut Self::State) -> Self::Item<'w, 's>;
}

pub trait System {
    fn run(&mut self, world: &mut World);
    fn initialize(&mut self, world: &mut World);
}

pub trait SystemParamFunction<Marker>: Send + Sync + 'static {
    type Out;

    type Param: SystemParam;

    fn run(&mut self, param_value: <Self::Param as SystemParam>::Item<'_, '_>) -> Self::Out;
}

pub trait IntoSystem<Out, Marker> {
    type System: System;

    fn into_system(self) -> Self::System;
}

impl SystemParam for () {
    type State = ();

    type Item<'world, 'state> = ();

    fn init_state(world: &mut World) -> Self::State {
        let _ = world;
        ()
    }

    fn get_param<'w, 's>(world: &'w mut World, state: &'s mut Self::State) -> Self::Item<'w, 's> {
        let _ = state;
        let _ = world;
        ()
    }
}

impl<T1: SystemParam, T2: SystemParam> SystemParam for (T1, T2) {
    type State = (T1::State, T2::State);

    type Item<'world, 'state> = (T1::Item<'world, 'state>, T2::Item<'world, 'state>);

    fn init_state(world: &mut World) -> Self::State {
        (T1::init_state(world), T2::init_state(world))
    }

    fn get_param<'w, 's>(world: &'w mut World, state: &'s mut Self::State) -> Self::Item<'w, 's> {
        let (state1, state2) = state;
        let world_ref = std::ptr::from_mut(world);
        // FIXME: idk what I'm doing, I probably should not use unsafe here or something
        unsafe {
            (
                T1::get_param(&mut *world_ref, state1),
                T2::get_param(&mut *world_ref, state2),
            )
        }
    }
}

impl<T1: SystemParam, T2: SystemParam, T3: SystemParam> SystemParam for (T1, T2, T3) {
    type State = (T1::State, T2::State, T3::State);

    type Item<'world, 'state> = (
        T1::Item<'world, 'state>,
        T2::Item<'world, 'state>,
        T3::Item<'world, 'state>,
    );

    fn init_state(world: &mut World) -> Self::State {
        (
            T1::init_state(world),
            T2::init_state(world),
            T3::init_state(world),
        )
    }

    fn get_param<'w, 's>(world: &'w mut World, state: &'s mut Self::State) -> Self::Item<'w, 's> {
        let (state1, state2, state3) = state;
        let world_ref = std::ptr::from_mut(world);
        // FIXME: idk what I'm doing, I probably should not use unsafe here or something
        unsafe {
            (
                T1::get_param(&mut *world_ref, state1),
                T2::get_param(&mut *world_ref, state2),
                T3::get_param(&mut *world_ref, state3),
            )
        }
    }
}

impl<T1: SystemParam, T2: SystemParam, T3: SystemParam, T4: SystemParam> SystemParam
    for (T1, T2, T3, T4)
{
    type State = (T1::State, T2::State, T3::State, T4::State);

    type Item<'world, 'state> = (
        T1::Item<'world, 'state>,
        T2::Item<'world, 'state>,
        T3::Item<'world, 'state>,
        T4::Item<'world, 'state>,
    );

    fn init_state(world: &mut World) -> Self::State {
        (
            T1::init_state(world),
            T2::init_state(world),
            T3::init_state(world),
            T4::init_state(world),
        )
    }

    fn get_param<'w, 's>(world: &'w mut World, state: &'s mut Self::State) -> Self::Item<'w, 's> {
        let (state1, state2, state3, state4) = state;
        let world_ref = std::ptr::from_mut(world);
        // FIXME: idk what I'm doing, I probably should not use unsafe here or something
        unsafe {
            (
                T1::get_param(&mut *world_ref, state1),
                T2::get_param(&mut *world_ref, state2),
                T3::get_param(&mut *world_ref, state3),
                T4::get_param(&mut *world_ref, state4),
            )
        }
    }
}

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

    pub fn add_system<O, M, S: System + 'static>(
        &mut self,
        system: impl IntoSystem<O, M, System = S>,
    ) {
        self.systems.push(Box::new(system.into_system()));
    }

    pub(crate) fn initialize(&mut self, world: &mut World) {
        for system in &mut self.systems {
            system.initialize(world);
        }
    }
}

pub struct FunctionSystemState<P: SystemParam> {
    param: P::State,
}

pub struct FunctionSystem<Input, F>
where
    F: SystemParamFunction<Input>,
{
    f: F,
    // TODO: add state to systems so we can have resources local to the system
    // (for example an `EventReader` that tracks which events were read by the system)
    state: Option<FunctionSystemState<F::Param>>,

    // we need a marker because otherwise we're not using `Input`.
    // fn() -> Input is chosen because just using Input would not be `Send` + `Sync`,
    // but the fnptr is always `Send` + `Sync`.
    //
    // Also, this way Input is covariant, but that's not super relevant since we can only deal with
    // static parameters here anyway so there's no subtyping. More info here:
    // https://doc.rust-lang.org/nomicon/subtyping.html
    marker: PhantomData<fn() -> Input>,
}

impl<Marker: 'static, F: SystemParamFunction<Marker>> System for FunctionSystem<Marker, F> {
    fn run(&mut self, world: &mut World) {
        let param = &mut self
            .state
            .as_mut()
            .expect("params were not initialized")
            .param;
        let param_state = F::Param::get_param(world, param);
        self.f.run(param_state);
    }

    fn initialize(&mut self, world: &mut World) {
        self.state = Some(FunctionSystemState {
            param: F::Param::init_state(world),
        });
    }
}

impl<Out, Func> SystemParamFunction<fn() -> Out> for Func
where
    Func: Send + Sync + 'static,
    for<'a> &'a mut Func: FnMut() -> Out + FnMut() -> Out,
    Out: 'static,
{
    type Out = Out;

    type Param = ();

    fn run(&mut self, param_value: <Self::Param as SystemParam>::Item<'_, '_>) -> Self::Out {
        let _ = param_value;

        fn call_inner<Out>(mut f: impl FnMut() -> Out) -> Out {
            f()
        }
        call_inner(self)
    }
}

impl<Out, Func, T: SystemParam> SystemParamFunction<fn(T) -> Out> for Func
where
    Func: Send + Sync + 'static,
    for<'a> &'a mut Func: FnMut(T) -> Out + FnMut(<T as SystemParam>::Item<'_, '_>) -> Out,
    Out: 'static,
{
    type Out = Out;

    type Param = T;

    fn run(&mut self, param_value: <Self::Param as SystemParam>::Item<'_, '_>) -> Self::Out {
        fn call_inner<T, Out>(mut f: impl FnMut(T) -> Out, _0: T) -> Out {
            f(_0)
        }
        call_inner(self, param_value)
    }
}

impl<Out, Func, T1: SystemParam, T2: SystemParam> SystemParamFunction<fn(T1, T2) -> Out> for Func
where
    Func: Send + Sync + 'static,
    for<'a> &'a mut Func: FnMut(T1, T2) -> Out
        + FnMut(<T1 as SystemParam>::Item<'_, '_>, <T2 as SystemParam>::Item<'_, '_>) -> Out,
    Out: 'static,
{
    type Out = Out;

    type Param = (T1, T2);

    fn run(&mut self, param_value: <Self::Param as SystemParam>::Item<'_, '_>) -> Self::Out {
        fn call_inner<Out, T1, T2>(mut f: impl FnMut(T1, T2) -> Out, _0: T1, _1: T2) -> Out {
            f(_0, _1)
        }
        let (_0, _1) = param_value;
        call_inner(self, _0, _1)
    }
}

impl<Out, Func, T1: SystemParam, T2: SystemParam, T3: SystemParam>
    SystemParamFunction<fn(T1, T2, T3) -> Out> for Func
where
    Func: Send + Sync + 'static,
    for<'a> &'a mut Func: FnMut(T1, T2, T3) -> Out
        + FnMut(
            <T1 as SystemParam>::Item<'_, '_>,
            <T2 as SystemParam>::Item<'_, '_>,
            <T3 as SystemParam>::Item<'_, '_>,
        ) -> Out,
    Out: 'static,
{
    type Out = Out;

    type Param = (T1, T2, T3);

    fn run(&mut self, param_value: <Self::Param as SystemParam>::Item<'_, '_>) -> Self::Out {
        fn call_inner<Out, T1, T2, T3>(
            mut f: impl FnMut(T1, T2, T3) -> Out,
            _0: T1,
            _1: T2,
            _2: T3,
        ) -> Out {
            f(_0, _1, _2)
        }
        let (_0, _1, _2) = param_value;
        call_inner(self, _0, _1, _2)
    }
}

impl<Out, Func, T1: SystemParam, T2: SystemParam, T3: SystemParam, T4: SystemParam>
    SystemParamFunction<fn(T1, T2, T3, T4) -> Out> for Func
where
    Func: Send + Sync + 'static,
    for<'a> &'a mut Func: FnMut(T1, T2, T3, T4) -> Out
        + FnMut(
            <T1 as SystemParam>::Item<'_, '_>,
            <T2 as SystemParam>::Item<'_, '_>,
            <T3 as SystemParam>::Item<'_, '_>,
            <T4 as SystemParam>::Item<'_, '_>,
        ) -> Out,
    Out: 'static,
{
    type Out = Out;

    type Param = (T1, T2, T3, T4);

    fn run(&mut self, param_value: <Self::Param as SystemParam>::Item<'_, '_>) -> Self::Out {
        fn call_inner<Out, T1, T2, T3, T4>(
            mut f: impl FnMut(T1, T2, T3, T4) -> Out,
            _0: T1,
            _1: T2,
            _2: T3,
            _3: T4,
        ) -> Out {
            f(_0, _1, _2, _3)
        }
        let (_0, _1, _2, _3) = param_value;
        call_inner(self, _0, _1, _2, _3)
    }
}

impl<Marker: 'static, F: SystemParamFunction<Marker>> IntoSystem<F::Out, Marker> for F {
    type System = FunctionSystem<Marker, Self>;

    fn into_system(self) -> Self::System {
        FunctionSystem {
            f: self,
            state: None,
            marker: Default::default(),
        }
    }
}

pub struct Res<'a, T>(RwLockReadGuard<'a, T>);

impl<'a, T: 'static> SystemParam for Res<'a, T> {
    type State = ();

    type Item<'w, 's> = Res<'w, T>;

    fn init_state(world: &mut World) -> Self::State {
        let _ = world;
        ()
    }

    fn get_param<'w, 's>(world: &'w mut World, state: &'s mut Self::State) -> Self::Item<'w, 's> {
        let _ = state;
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
    type State = ();

    type Item<'w, 's> = ResMut<'w, T>;

    fn init_state(world: &mut World) -> Self::State {
        let _ = world;
        ()
    }

    fn get_param<'w, 's>(world: &'w mut World, state: &'s mut Self::State) -> Self::Item<'w, 's> {
        let _ = state;
        ResMut(world.write_resource::<T>().expect("Resource not found"))
    }
}

pub struct Local<'s, T: Default + 'static>(pub(crate) &'s mut T);

impl<'s, T: Default + 'static> Deref for Local<'s, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'s, T: Default + 'static> DerefMut for Local<'s, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl<'a, T: Default + 'static> SystemParam for Local<'a, T> {
    type State = T;

    type Item<'world, 'state> = Local<'state, T>;

    fn init_state(world: &mut World) -> Self::State {
        // TODO: add a `FromWorld` trait to allow for state to use world for initialization
        let _ = world;
        T::default()
    }

    fn get_param<'w, 's>(world: &'w mut World, state: &'s mut Self::State) -> Self::Item<'w, 's> {
        let _ = world;
        Local(state)
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

        scheduler.initialize(&mut world);

        scheduler.run(&mut world);
    }
}
