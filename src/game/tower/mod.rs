use bevy::prelude::*;

use components::*;
use events::*;
use resources::*;
use systems::*;

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
        app
            .add_system(throw_projectiles)
            .add_system(projectile_follow_step)
            .add_system(deal_projectile_damage)
            .add_system(build_tower_at_click.run_if(can_build))
        ;
    }
}
