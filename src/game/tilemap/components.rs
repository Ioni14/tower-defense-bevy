use bevy::prelude::*;

#[derive(Component)]
pub struct SelectedForBuild {}

#[derive(Component)]
pub struct BuiltTile {}

#[derive(Component)]
pub struct BuildZone {
    pub rect: Rect,
}
