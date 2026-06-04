use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, move_player)
        .run();
}

#[derive(Component)]
struct Player;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        Sprite::from_color(Color::srgb(1.0, 0.48, 0.27), Vec2::new(48.0, 48.0)),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Player,
    ));
}

fn move_player(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut players: Query<&mut Transform, With<Player>>,
) {
    let speed = 300.0;
    let dt = time.delta_secs();

    for mut transform in &mut players {
        if keys.pressed(KeyCode::KeyW) { transform.translation.y += speed * dt; }
        if keys.pressed(KeyCode::KeyS) { transform.translation.y -= speed * dt; }
        if keys.pressed(KeyCode::KeyA) { transform.translation.x -= speed * dt; }
        if keys.pressed(KeyCode::KeyD) { transform.translation.x += speed * dt; }
    }
}