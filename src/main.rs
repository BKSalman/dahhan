use dahhan::{camera::Camera, prelude::*, App};
use glam::{Vec2, Vec3};

fn get_input(squares: Query<(Read<Sprite>, Write<Transform>)>, input: Res<Input>) {
    for (_e, (_, transform)) in squares.iter() {
        let vertical_movement = (input.is_pressed(winit::keyboard::KeyCode::KeyW) as i8
            - input.is_pressed(winit::keyboard::KeyCode::KeyS) as i8)
            as f32;
        let horizontal_movement = (input.is_pressed(winit::keyboard::KeyCode::KeyD) as i8
            - input.is_pressed(winit::keyboard::KeyCode::KeyA) as i8)
            as f32;

        if horizontal_movement != 0. || vertical_movement != 0. {
            let length = f32::sqrt(
                horizontal_movement * horizontal_movement + vertical_movement * vertical_movement,
            );

            let normalized_horizontal = horizontal_movement / length;
            let normalized_vertical = vertical_movement / length;

            transform.position.x += normalized_horizontal;
            transform.position.y += normalized_vertical;
        }
    }
}

fn move_camera(cameras: Query<(Read<Camera>, Write<Transform>)>, input: Res<Input>) {
    if let Some((_e, (_, transform))) = cameras.iter().next() {
        let vertical_movement = (input.is_pressed(winit::keyboard::KeyCode::ArrowUp) as i8
            - input.is_pressed(winit::keyboard::KeyCode::ArrowDown) as i8)
            as f32;
        let horizontal_movement = (input.is_pressed(winit::keyboard::KeyCode::ArrowRight) as i8
            - input.is_pressed(winit::keyboard::KeyCode::ArrowLeft) as i8)
            as f32;

        if horizontal_movement != 0. || vertical_movement != 0. {
            let length = f32::sqrt(
                horizontal_movement * horizontal_movement + vertical_movement * vertical_movement,
            );

            let normalized_horizontal = horizontal_movement / length;
            let normalized_vertical = vertical_movement / length;

            transform.position.x += normalized_horizontal;
            transform.position.y += normalized_vertical;
        }

        transform.scale += input.scroll_delta();
    }
}

pub fn main() {
    let mut app = App::new();

    app.add_entity((
        Sprite {
            texture_id: None,
            size: Vec2::splat(10.),
            color: Vec3::new(0., 1., 1.),
        },
        Transform {
            position: Vec3::new(0., 0., 0.),
            rotation: 0.,
            scale: Vec2::splat(1.),
        },
    ));

    app.add_entity((
        Camera::default_2d(),
        Transform {
            position: Vec3::new(0., 0., 0.),
            rotation: 0.,
            scale: Vec2::splat(1.),
        },
    ));

    app.add_system(get_input);
    app.add_system(move_camera);

    app.run().unwrap();
}
