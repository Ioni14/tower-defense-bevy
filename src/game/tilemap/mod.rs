use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use components::*;
use resources::*;
use systems::*;

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
            .add_system(select_build_zone)
        ;
    }
}
