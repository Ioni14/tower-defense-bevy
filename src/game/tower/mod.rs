use bevy::prelude::*;

use components::*;
use events::*;
use resources::*;
use systems::*;

use crate::AppState;
use crate::game::*;
use crate::game::tilemap::can_build;

mod systems;
mod components;
mod resources;
mod events;

pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<ProjectileHitEvent>()
        ;
        app.add_systems(
            (
                throw_projectiles,
                throw_splashes,
                projectile_follow_step,
                pointer_follow_step,
                deal_projectile_damage,
            )
                .in_set(OnUpdate(AppState::Game))
        );
        app
            .add_system(build_tower_at_click
                .run_if(can_build)
                .in_set(OnUpdate(AppState::Game))
            )
        ;
    }
}
