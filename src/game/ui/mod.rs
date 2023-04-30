use bevy::prelude::*;

use systems::*;

mod systems;
mod components;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_action_bar);
        app.add_system(interact_with_build_action);
    }
}
