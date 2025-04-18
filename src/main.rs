use dahhan::{
    ecs::{
        query::{Query, Read},
        Component,
    },
    App,
};

#[derive(Debug)]
struct Health(u32);

impl Component for Health {}

fn print_healths(healths: Query<Read<Health>>) {
    for (e, health) in healths.iter() {
        println!("{e:?}: {health:?}");
    }
}

pub fn main() {
    let mut app = App::new();

    app.register_component::<Health>();

    let entity = app.add_entity(Health(10));
    println!("Added {entity:?}");

    app.add_system(print_healths);

    app.run().unwrap();
}
