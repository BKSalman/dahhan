use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dahhan::{
    ecs::{Component, Query},
    World,
};

struct Something(u32);

impl Component for Something {}

fn iterate_component() {
    let mut world = World::new();
    world.register_component::<Something>();

    for i in 0..1000 {
        world.add_entity(Something(i));
    }

    let components: Vec<&Something> = world.query::<&Something>().collect();
}

fn iterate_vec() {
    let mut vec = vec![];

    for i in 0..1000 {
        vec.push(Something(i));
    }

    let components: Vec<&Something> = vec.iter().collect();
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("query iterate", move |b| {
        b.iter(move || black_box(iterate_component()));
    });

    c.bench_function("vec iterate", move |b| {
        b.iter(move || black_box(iterate_vec()));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
