use bevy::prelude::*;

const CAMERA_SPEED: f32 = 1000.0;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 999.0),
        ..default()
    });
}

pub fn move_camera(
    mut camera_query: Query<&mut Transform, With<Camera>>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let mut direction = Vec3::new(0.0, 0.0, 0.0);
    if keyboard.pressed(KeyCode::Z) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::Q) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::S) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::D) {
        direction.x += 1.0;
    }
    camera_query.single_mut().translation += direction * CAMERA_SPEED * time.delta_seconds();
}
