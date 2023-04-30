use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use components::*;
use resources::*;
use systems::*;
use crate::game::{GameState, UiState};

mod systems;
pub mod components;
mod resources;
mod tiled;

pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<CursorPos>();
        ;
        app
            .add_plugin(bevy_ecs_tilemap::TilemapPlugin)
            .add_plugin(tiled::TiledMapPlugin)
        ;
        app
            .add_startup_system(setup_map)
        ;

        app
            .add_system(update_cursor_pos)
            .add_system(unselect_build_zone.in_schedule(OnExit(GameState::Building)))
            .add_system(select_build_zone.run_if(can_build))
        ;
    }
}

pub fn can_build(
    game_state: Res<State<GameState>>,
    ui_state: Res<State<UiState>>,
) -> bool {
    return game_state.0 == GameState::Building && ui_state.0 == UiState::Nothing;
}
