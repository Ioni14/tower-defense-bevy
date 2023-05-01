use bevy::prelude::*;

#[derive(Component)]
pub struct Tower {}

#[derive(Component)]
pub struct ProjectileThrower {
    pub relative_start: Vec2,
    pub cooldown: Timer,
    pub range: f32,
}

#[derive(Component)]
pub struct Splasher {
    pub relative_start: Vec2,
    pub cooldown: Timer,
    pub range: f32,
}

#[derive(Component)]
pub struct Projectile {
    pub damage: i32,
}

#[derive(Component)]
pub struct Follower {
    pub speed: f32,
    pub target: Entity,
}

#[derive(Component)]
pub struct Pointer {
    pub speed: f32,
    pub target: Vec2,
    pub pos: Vec2,
    pub source: Vec2,
}
