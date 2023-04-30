use bevy::prelude::*;

use systems::*;
use crate::AppState;

mod systems;
mod components;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_action_bar.in_schedule(OnEnter(AppState::Game)));
        app.add_system(interact_with_build_action.in_set(OnUpdate(AppState::Game)));
    }
}
