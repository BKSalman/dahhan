use dahhan::{
    prelude::{Sprite, Transform},
    App,
};
use glam::{Vec2, Vec3};

pub fn main() {
    let mut app = App::new();

    app.add_entity((
        Sprite {
            texture_id: None,
            size: Vec2::splat(10.),
            color: Vec3::new(0., 1., 1.),
        },
        Transform {
            position: Vec3::new(200., 200., 0.),
            rotation: 0.,
            scale: Vec2::splat(1.),
        },
    ));

    app.add_entity((
        Sprite {
            texture_id: None,
            size: Vec2::splat(20.),
            color: Vec3::new(0., 1., 1.),
        },
        Transform {
            position: Vec3::new(500., 500., 0.),
            rotation: 0.,
            scale: Vec2::splat(1.),
        },
    ));

    app.run().unwrap();
}
