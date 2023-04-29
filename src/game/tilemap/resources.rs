use bevy::prelude::*;

#[derive(Resource)]
pub struct CursorPos(pub Vec2);

impl Default for CursorPos {
    fn default() -> Self {
        Self(Vec2::new(0.0, 0.0))
    }
}
