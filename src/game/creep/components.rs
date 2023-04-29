use bevy::prelude::*;

#[derive(Component)]
pub struct Enemy {}

#[derive(Component)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

impl Health {
    pub fn full(max: i32) -> Self { Health { current: max, max } }
}

#[derive(Component)]
pub struct Waypoint {
    pub index: i32,
    pub position: Vec2,
}

#[derive(Component)]
pub struct Velocity {
    pub speed: f32,
    pub direction: Vec2,
}

#[derive(Component)]
pub struct EnemySpawner {
    pub timer: Timer,
    pub position: Vec2,
}

#[derive(Component)]
pub struct EnemyFinish {
    pub position: Vec2,
}

#[derive(Component)]
pub struct WaypointFollower {
    pub index: i32,
}

#[derive(Component)]
pub struct Healthbar {
    pub length: f32,
    pub height: f32,
}
