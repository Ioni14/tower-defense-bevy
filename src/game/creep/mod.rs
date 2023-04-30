mod systems;
pub mod components;
mod resources;
pub mod events;

use bevy::prelude::*;
use bevy::render::RenderApp;
use bevy::sprite::SpriteSystem;
use events::*;
use resources::*;
use components::*;
use systems::*;

pub struct CreepPlugin;

impl Plugin for CreepPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<KilledEvent>();
        ;

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_systems(
                    (
                        extract_health_bar.after(SpriteSystem::ExtractSprites),
                    ).in_schedule(ExtractSchedule),
                );
        };

        app
            .add_system(spawn_enemy)
            .add_system(follow_waypoint)
            .add_system(reach_waypoint)
            .add_system(do_move_step)
            .add_system(on_enemy_killed)
            .add_system(despawn_dying)
        ;
    }
}
