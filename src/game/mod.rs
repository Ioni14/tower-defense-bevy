use bevy::prelude::*;

use creep::CreepPlugin;
use systems::*;
use tilemap::TilemapPlugin;
use tower::TowerPlugin;

mod tilemap;
mod creep;
mod tower;
mod systems;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(TilemapPlugin)
            .add_plugin(CreepPlugin)
            .add_plugin(TowerPlugin)
        ;
        app
            .add_startup_system(setup_camera)
        ;
        app
            .add_system(move_camera)
        ;
    }
}
