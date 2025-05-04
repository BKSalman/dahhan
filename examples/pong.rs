use dahhan::{camera::Camera, ecs::Component, prelude::*, App, WindowResized};
use glam::{Vec2, Vec3};

struct Background;
impl Component for Background {}

struct Ball {
    is_going_up: bool,
    is_going_right: bool,
}
impl Component for Ball {}

struct Player1 {
    score: i32,
}
impl Component for Player1 {}

struct Player2 {
    score: i32,
}
impl Component for Player2 {}

fn move_player1(
    player1: Query<(Read<Player1>, Write<Transform>)>,
    input: Res<Input>,
    time: Res<Time>,
) {
    for (_e, (_player, transform)) in player1.iter() {
        if input.is_pressed(KeyCode::KeyW) {
            transform.position.y += 10000. * time.delta_time();
        } else if input.is_pressed(KeyCode::KeyS) {
            transform.position.y -= 10000. * time.delta_time();
        }
    }
}

fn move_player2(
    player1: Query<(Read<Player2>, Write<Transform>)>,
    input: Res<Input>,
    time: Res<Time>,
) {
    for (_e, (_player, transform)) in player1.iter() {
        if input.is_pressed(KeyCode::ArrowUp) {
            transform.position.y += 10000. * time.delta_time();
        } else if input.is_pressed(KeyCode::ArrowDown) {
            transform.position.y -= 10000. * time.delta_time();
        }
    }
}

fn ball_collision(
    ball: Query<(Write<Ball>, Read<Sprite>, Read<Transform>)>,
    player1: Query<(Read<Player1>, Read<Transform>, Read<Sprite>)>,
    player2: Query<(Read<Player2>, Read<Transform>, Read<Sprite>)>,
    window: Res<Window>,
) {
    if let Some((_e, (ball, ball_sprite, ball_transform))) = ball.iter().next() {
        if let Some((_e, (_, player1_transform, player1_sprite))) = player1.iter().next() {
            if let Some((_e, (_, player2_transform, player2_sprite))) = player2.iter().next() {
                if ball_transform.position.x <= player1_transform.position.x + player1_sprite.size.x
                    && ball_transform.position.y
                        >= player1_transform.position.y - player1_sprite.size.y
                    && ball_transform.position.y <= player1_transform.position.y
                {
                    ball.is_going_right = true;
                } else if ball_transform.position.x + ball_sprite.size.x
                    >= player2_transform.position.x
                    && ball_transform.position.y
                        >= player2_transform.position.y - player2_sprite.size.y
                    && ball_transform.position.y <= player2_transform.position.y
                {
                    ball.is_going_right = false;
                } else if ball_transform.position.y - ball_sprite.size.y <= -window.height / 2. {
                    ball.is_going_up = true;
                } else if ball_transform.position.y >= window.height / 2. {
                    ball.is_going_up = false;
                }
            }
        }
    }
}

fn ball_scoring(
    ball: Query<(Write<Ball>, Read<Sprite>, Write<Transform>)>,
    player1: Query<Write<Player1>>,
    player2: Query<Write<Player2>>,
    window: Res<Window>,
) {
    if let Some((_e, (ball, ball_sprite, ball_transform))) = ball.iter().next() {
        if let Some((_, player1)) = player1.iter().next() {
            if let Some((_, player2)) = player2.iter().next() {
                if ball_transform.position.x <= -window.width / 2. {
                    player1.score += 1;
                    ball_transform.position.x = -ball_sprite.size.x / 2.;
                    ball_transform.position.y = ball_sprite.size.y / 2.;
                    ball.is_going_up = !ball.is_going_up;
                    ball.is_going_right = !ball.is_going_right;
                } else if ball_transform.position.x + ball_sprite.size.x >= window.width / 2. {
                    player2.score += 1;
                    ball_transform.position.x = -ball_sprite.size.x / 2.;
                    ball_transform.position.y = ball_sprite.size.y / 2.;
                    ball.is_going_up = !ball.is_going_up;
                    ball.is_going_right = !ball.is_going_right;
                }
            }
        }
    }
}

fn move_ball(ball: Query<(Read<Ball>, Write<Transform>)>, time: Res<Time>) {
    if let Some((_e, (ball, transform))) = ball.iter().next() {
        if ball.is_going_up {
            transform.position.y += 1000. * time.delta_time();
        } else {
            transform.position.y -= 1000. * time.delta_time();
        }

        if ball.is_going_right {
            transform.position.x += 1000. * time.delta_time();
        } else {
            transform.position.x -= 1000. * time.delta_time();
        }
    }
}

fn resize_background(
    background: Query<(Read<Background>, Write<Transform>, Write<Sprite>)>,
    mut resize_events: EventReader<WindowResized>,
) {
    if let Some((_e, (_background, transform, sprite))) = background.iter().next() {
        for new_size in resize_events.read() {
            sprite.size.x = new_size.width;
            sprite.size.y = new_size.height;
            transform.position.x = -new_size.width / 2.;
            transform.position.y = new_size.height / 2.;
        }
    }
}

fn main() {
    let mut app = App::new();

    app.add_entity((
        Camera::default_2d(),
        Transform {
            position: Vec3::new(0., 0., 0.),
            rotation: 0.,
            scale: Vec2::splat(1.),
        },
    ));

    app.register_component::<Background>();
    app.register_component::<Ball>();
    app.register_component::<Player1>();
    app.register_component::<Player2>();

    app.add_entity((
        Background,
        Sprite {
            texture_id: None,
            size: Vec2::new(50., 200.),
            color: Vec3::new(0.1, 0.1, 0.1),
        },
        Transform {
            position: Vec3::new(400., 100., 0.),
            rotation: 0.,
            scale: Vec2::splat(1.),
        },
    ));

    app.add_entity((
        Ball {
            is_going_up: true,
            is_going_right: true,
        },
        Sprite {
            texture_id: None,
            size: Vec2::splat(50.),
            color: Vec3::new(0.5, 0.5, 0.5),
        },
        Transform {
            position: Vec3::new(-25., 25., 0.),
            rotation: 0.,
            scale: Vec2::splat(1.),
        },
    ));

    app.add_entity((
        Player1 { score: 0 },
        Sprite {
            texture_id: None,
            size: Vec2::new(50., 200.),
            color: Vec3::new(1., 0.5, 0.5),
        },
        Transform {
            position: Vec3::new(-450., 100., 0.),
            rotation: 0.,
            scale: Vec2::splat(1.),
        },
    ));

    app.add_entity((
        Player2 { score: 0 },
        Sprite {
            texture_id: None,
            size: Vec2::new(50., 200.),
            color: Vec3::new(0., 0.5, 0.5),
        },
        Transform {
            position: Vec3::new(400., 100., 0.),
            rotation: 0.,
            scale: Vec2::splat(1.),
        },
    ));

    app.add_system(move_player1);
    app.add_system(move_player2);
    app.add_system(ball_collision);
    app.add_system(ball_scoring);
    app.add_system(move_ball);
    app.add_system(resize_background);

    app.run().unwrap();
}
