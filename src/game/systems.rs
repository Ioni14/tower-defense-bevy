use bevy::prelude::*;

const CAMERA_SPEED: f32 = 1000.0;

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
