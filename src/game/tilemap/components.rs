use bevy::prelude::*;
use crate::game::resources::TowerType;

#[derive(Component)]
pub struct SelectedForBuild {}

#[derive(Component)]
pub struct BuiltTile {}

#[derive(Component)]
pub struct BuildZone {
    pub rect: Rect,
}
