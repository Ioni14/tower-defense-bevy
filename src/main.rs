use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_ecs_tilemap::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

// use rand::prelude::*;

mod tiled;

const CAMERA_SPEED: f32 = 1000.0;

fn main() {
    let mut app = App::new();

    app.insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)));
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
    app.add_plugin(TilemapPlugin);
    app.add_plugin(tiled::TiledMapPlugin);

    app
        .add_startup_system(setup_camera)
        .add_startup_system(setup_map)
        .add_startup_system(spawn_tower.after(setup_map))
    ;

    app
        .add_system(spawn_enemy)
        .add_system(throw_projectiles)
        .add_system(projectile_follow_step)
        .add_system(do_move_step)
        .add_system(move_camera)
    ;

    app.run();
}

#[derive(Component)]
pub struct Tower {}

#[derive(Component)]
pub struct Enemy {}

#[derive(Component)]
pub struct Waypoint {
    pub index: i32,
    pub position: Vec2,
}

#[derive(Component)]
pub struct EnemySpawner {
    pub timer: Timer,
    pub position: Vec2,
}

#[derive(Component)]
pub struct EnemyFinish {
    pub position: Vec2,
}

#[derive(Component)]
pub struct ProjectileThrower {
    pub relative_start: Vec2,
    pub cooldown: Timer,
}

#[derive(Component)]
pub struct Projectile {}

#[derive(Component)]
pub struct Follower {
    pub speed: f32,
    pub target: Entity,
}

#[derive(Component)]
pub struct Velocity {
    pub speed: f32,
    pub direction: Vec2,
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 999.0),
        ..default()
    });
}

pub fn spawn_tower(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(
        (
            Tower {},
            ProjectileThrower {
                relative_start: Vec2::new(0.0, 2.0 / 3.0 * 64.0),
                cooldown: Timer::from_seconds(0.1, TimerMode::Repeating),
            },
            SpriteBundle {
                transform: Transform::from_xyz(-128.0, -128.0, 10.0),
                // texture: asset_server.load("sprites/tower.png"),
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(64.0)),
                    color: Color::rgb(0.0, 1.0, 0.5),
                    anchor: Anchor::BottomCenter,
                    ..default()
                },
                ..default()
            },
            Name::new("Tower"),
        ),
    );
}

pub fn spawn_enemy(
    mut commands: Commands,
    mut enemy_spawner_query: Query<&mut EnemySpawner>,
    tile_map_query: Query<&Transform, With<TileStorage>>,
    time: Res<Time>,
) {
    // let enemy_waypoints = enemy_waypoints_query.single();
    let Ok(mut enemy_spawner) = enemy_spawner_query.get_single_mut() else {
        return;
    };

    enemy_spawner.timer.tick(time.delta());
    if !enemy_spawner.timer.finished() {
        return;
    }

    println!("Spawning enemy {:?}", enemy_spawner.position);

    commands.spawn(
        (
            Enemy {},
            Velocity {
                speed: 100.0,
                direction: Vec2::new(1.0, 0.0),
            },
            SpriteBundle {
                transform: Transform::from_translation(Vec3::from((enemy_spawner.position, 10.0)) + tile_map_query.single().translation),
                // texture: asset_server.load("sprites/enemy.png"),
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(48.0)),
                    color: Color::rgb(1.0, 0.25, 0.25),
                    anchor: Anchor::Center,
                    ..default()
                },
                ..default()
            },
            Name::new("Enemy"),
        ),
    );
}

pub fn setup_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let map_handle: Handle<tiled::TiledMap> = asset_server.load("tiled/map_1.tmx");

    commands.spawn(tiled::TiledMapBundle {
        tiled_map: map_handle,
        ..default()
    });
    //
    // let texture_handle: Handle<Image> = asset_server.load("sprites/towerDefense_tilesheet.png");
    // let map_size = TilemapSize { x: 32, y: 32 };
    //
    // let tilemap_entity = commands.spawn_empty().id();
    //
    // let mut tile_storage = TileStorage::empty(map_size);
    // for x in 0..map_size.x {
    //     for y in 0..map_size.y {
    //         let tile_pos = TilePos { x, y };
    //         let tile_entity = commands
    //             .spawn(TileBundle {
    //                 position: tile_pos,
    //                 tilemap_id: TilemapId(tilemap_entity),
    //                 texture_index: TileTextureIndex(162),
    //                 ..default()
    //             })
    //             .id();
    //         tile_storage.set(&tile_pos, tile_entity);
    //     }
    // }
    //
    // let tile_size = TilemapTileSize { x: 64.0, y: 64.0 };
    // let grid_size = tile_size.into();
    // let map_type = TilemapType::Square;
    //
    // commands.entity(tilemap_entity)
    //     .insert(TilemapBundle {
    //         grid_size,
    //         map_type,
    //         size: map_size,
    //         storage: tile_storage,
    //         texture: TilemapTexture::Single(texture_handle),
    //         tile_size,
    //         transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
    //         ..default()
    //     });
}

pub fn throw_projectiles(
    mut commands: Commands,
    mut thrower_query: Query<(&mut ProjectileThrower, &Transform)>,
    asset_server: Res<AssetServer>,
    enemies_query: Query<(Entity, &Transform), With<Enemy>>,
    time: Res<Time>,
) {
    for (mut projectile_thrower, thrower_transform) in thrower_query.iter_mut() {
        projectile_thrower.cooldown.tick(time.delta());
        if !projectile_thrower.cooldown.finished() {
            continue;
        }
        let mut closest_enemy: Option<(Entity, &Transform)> = None;
        for (enemy_entity, enemy_transform) in enemies_query.iter() {
            if closest_enemy.is_none() {
                closest_enemy = Some((enemy_entity, enemy_transform));
                continue;
            }
            if (enemy_transform.translation - thrower_transform.translation).length_squared()
                < (closest_enemy.unwrap().1.translation - thrower_transform.translation)
                .length_squared()
            {
                closest_enemy = Some((enemy_entity, enemy_transform));
            }
        }

        if closest_enemy.is_none() {
            continue;
        }

        let direction = (closest_enemy.unwrap().1.translation - thrower_transform.translation).xy().normalize();

        commands.spawn(
            (
                Projectile {},
                Follower {
                    speed: 400.0,
                    target: closest_enemy.unwrap().0,
                },
                SpriteBundle {
                    transform: Transform::from_translation(thrower_transform.translation + Vec3::from((projectile_thrower.relative_start, 0.0))).with_rotation(
                        Quat::from_rotation_arc_2d(
                            Vec2::new(1.0, 0.0),
                            direction,
                        )
                    ).with_scale(Vec3::splat(0.25)),
                    texture: asset_server.load("sprites/arrow.png"),
                    sprite: Sprite {
                        // custom_size: Some(Vec2::splat(32.0)),
                        // color: Color::rgb(0.5, 0.0, 1.0),
                        anchor: Anchor::CenterRight,
                        ..default()
                    },
                    ..Default::default()
                },
                Name::new("Projectile"),
            ),
        );
    }
}

pub fn projectile_follow_step(
    mut commands: Commands,
    mut projectile_query: Query<(Entity, &Follower, &mut Transform), With<Projectile>>,
    target_query: Query<&Transform, Without<Projectile>>,
    time: Res<Time>,
) {
    for (follower_entity, follower, mut follower_transform) in projectile_query.iter_mut() {
        if let Ok(target_transform) = target_query.get(follower.target) {
            let direction_to_target = (target_transform.translation - follower_transform.translation).xy().normalize();
            follower_transform.translation += Vec3::from((direction_to_target, 0.0)) * follower.speed * time.delta_seconds();
            follower_transform.rotation = Quat::from_rotation_arc_2d(
                Vec2::new(1.0, 0.0),
                direction_to_target,
            );

            // check if projectile is close enough to target
            if (target_transform.translation - follower_transform.translation).length_squared() < 20.0 * 20.0 {
                // hit target (event?)
                commands.entity(follower_entity).despawn_recursive();
            }
        } else {
            // target does not exist anymore, despawn projectile
            commands.entity(follower_entity).despawn_recursive();
            continue;
        }
    }
}

pub fn do_move_step(
    mut move_query: Query<(&Velocity, &mut Transform)>,
    time: Res<Time>,
) {
    for (velocity, mut transform) in move_query.iter_mut() {
        transform.translation += Vec3::from((velocity.direction, 0.0)) * velocity.speed * time.delta_seconds();
    }
}

pub fn move_camera(
    mut camera_query: Query<&mut Transform, With<Camera>>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let mut direction = Vec3::new(0.0, 0.0, 0.0);
    if keyboard.pressed(KeyCode::Z) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::Q) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::S) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::D) {
        direction.x += 1.0;
    }
    camera_query.single_mut().translation += direction * CAMERA_SPEED * time.delta_seconds();
}
