use bevy::prelude::*;
use bevy::render::RenderApp;
use bevy::sprite::SpriteSystem;

use components::*;
use events::*;
use resources::*;
use systems::*;

use crate::AppState;

mod systems;
pub mod components;
mod resources;
pub mod events;

pub struct CreepPlugin;

impl Plugin for CreepPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<KilledEvent>()
        ;

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_systems(
                    (
                        extract_health_bar.after(SpriteSystem::ExtractSprites),
                    ).in_schedule(ExtractSchedule),
                );
        };

        app.add_systems(
            (
                spawn_enemy,
                follow_waypoint,
                reach_waypoint,
                do_move_step,
                on_enemy_killed,
                despawn_dying,
            )
                .in_set(OnUpdate(AppState::Game))
        );
    }
}
