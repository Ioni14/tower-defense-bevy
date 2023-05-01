use std::fmt::Debug;
use bevy::prelude::*;

#[derive(Copy, Clone)]
pub enum TowerType {
    Arrow,
    Bomb,
}
impl Debug for TowerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TowerType::Arrow => write!(f, "Arrow"),
            TowerType::Bomb => write!(f, "Bomb"),
        }
    }
}

#[derive(Resource)]
pub struct BuildTower {
    pub tower_type: TowerType,
}

impl Default for BuildTower {
    fn default() -> Self {
        Self {
            tower_type: TowerType::Arrow,
        }
    }
}
