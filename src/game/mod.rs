use bevy::prelude::*;

use creep::CreepPlugin;
use systems::*;
use tilemap::TilemapPlugin;
use tower::TowerPlugin;
use ui::UiPlugin;
use crate::AppState;
use crate::game::resources::BuildTower;

mod tilemap;
mod creep;
mod tower;
mod systems;
mod ui;
mod resources;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_state::<GameState>()
            .add_state::<UiState>()
        ;
        app.init_resource::<BuildTower>();
        app
            .add_plugin(TilemapPlugin)
            .add_plugin(CreepPlugin)
            .add_plugin(TowerPlugin)
            .add_plugin(UiPlugin)
        ;
        app
            .add_system(move_camera.in_set(OnUpdate(AppState::Game)))
        ;
    }
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    Playing,
    Building,
    Paused,
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum UiState {
    #[default]
    Nothing,
    ChoosingAction,
}
