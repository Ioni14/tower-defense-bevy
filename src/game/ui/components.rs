use bevy::prelude::*;

use crate::game::resources::TowerType;

#[derive(Component)]
pub struct BuildTowerAction {
    pub tower_type: TowerType,
}
