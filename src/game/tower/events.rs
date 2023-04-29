use bevy::prelude::*;

pub struct ProjectileHitEvent {
    pub damage: f32,
    pub target: Entity,
}

