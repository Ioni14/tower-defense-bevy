use std::thread::current;

use bevy::input::ButtonState;
use bevy::input::mouse::{MouseButtonInput, MouseMotion};
use bevy::math::{Vec3Swizzles, Vec4Swizzles};
use bevy::prelude::*;
use bevy::render::{Extract, RenderApp};
use bevy::render::texture::DEFAULT_IMAGE_HANDLE;
use bevy::sprite::{Anchor, ExtractedSprite, ExtractedSprites, SpriteSystem};
use bevy::window::PrimaryWindow;
use bevy_ecs_tilemap::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use crate::game::GamePlugin;

mod game;

fn main() {
    let mut app = App::new();

    app.insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)));
    app.insert_resource(Msaa::Off);
    app.add_plugins(DefaultPlugins
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "Ioni Tower Defense".into(),
                resolution: (1600.0, 900.0).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        })
        .set(ImagePlugin::default_nearest())
    );
    app.add_plugin(WorldInspectorPlugin::new());
    app.add_plugin(GamePlugin);

    app.run();
}
