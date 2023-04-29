use bevy::input::mouse::MouseMotion;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::render::{Extract, RenderApp};
use bevy::render::texture::DEFAULT_IMAGE_HANDLE;
use bevy::sprite::{Anchor, ExtractedSprite, ExtractedSprites, SpriteSystem};
use bevy::window::PrimaryWindow;
use bevy_ecs_tilemap::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

// use rand::prelude::*;

mod tiled;

const CAMERA_SPEED: f32 = 1000.0;

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
    app.add_plugin(TilemapPlugin);
    app.add_plugin(tiled::TiledMapPlugin);

    app.add_event::<ProjectileHitEvent>();
    app.add_event::<KilledEvent>();

    if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
        render_app
            .add_systems(
                (
                    extract_health_bar.after(SpriteSystem::ExtractSprites),
                )
                    .in_schedule(ExtractSchedule),
            );
    };

    app
        .add_startup_system(setup_camera)
        .add_startup_system(setup_map)
        .add_startup_system(spawn_tower.after(setup_map))
    ;

    app
        .add_system(spawn_enemy)
        .add_system(follow_waypoint)
        .add_system(reach_waypoint)
        .add_system(throw_projectiles)
        .add_system(projectile_follow_step)
        .add_system(do_move_step)
        .add_system(move_camera)
        .add_system(deal_projectile_damage)
        .add_system(on_enemy_killed)
    // .add_system(update_mouse_pos_display)
    ;

    app.run();
}

#[derive(Component)]
pub struct Tower {}

#[derive(Component)]
pub struct Enemy {}

#[derive(Component)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

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
    pub range: f32,
}

#[derive(Component)]
pub struct Projectile {
    damage: i32,
}

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

#[derive(Component)]
pub struct WaypointFollower {
    pub index: i32,
}

#[derive(Component)]
pub struct Healthbar {
    pub length: f32,
    pub height: f32,
}

pub struct ProjectileHitEvent {
    pub damage: f32,
    pub target: Entity,
}

pub struct KilledEvent {
    who: Entity,
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
                cooldown: Timer::from_seconds(1.0, TimerMode::Repeating),
                range: 450.0,
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
    asset_server: Res<AssetServer>,
    tile_map_query: Query<(&GlobalTransform, &TilemapTileSize), With<TileStorage>>,
    time: Res<Time>,
) {
    let Ok(mut enemy_spawner) = enemy_spawner_query.get_single_mut() else {
        return;
    };

    enemy_spawner.timer.tick(time.delta());
    if !enemy_spawner.timer.finished() {
        return;
    }

    let (tilemap_transform, tile_size) = tile_map_query.single();
    let tilemap_top_left = tilemap_transform.translation() - Vec3::new(tile_size.x / 2.0, tile_size.y / 2.0, 0.0);

    commands.spawn(
        (
            Enemy {},
            Health {
                current: 100,
                max: 100,
            },
            Healthbar {
                length: 64.0,
                height: 10.0,
            },
            Velocity {
                speed: 200.0,
                direction: Vec2::new(0.0, 0.0),
            },
            WaypointFollower {
                index: 0,
            },
            SpriteBundle {
                transform: Transform::from_translation(Vec3::from((enemy_spawner.position, 10.0)) + tilemap_top_left),
                texture: asset_server.load("sprites/enemy_1.png"),
                sprite: Sprite {
                    anchor: Anchor::Center,
                    ..default()
                },
                ..default()
            },
            Name::new("Enemy"),
        ),
    );
}

pub fn follow_waypoint(
    mut commands: Commands,
    mut follower_query: Query<(Entity, &WaypointFollower, &mut Velocity, &Transform)>,
    finish_query: Query<&EnemyFinish>,
    waypoints_query: Query<&Waypoint>,
    tile_map_query: Query<(&GlobalTransform, &TilemapTileSize), With<TileStorage>>,
) {
    let Ok((tilemap_transform, tile_size)) = tile_map_query.get_single() else {
        return;
    };
    let tilemap_top_left = tilemap_transform.translation() - Vec3::new(tile_size.x / 2.0, tile_size.y / 2.0, 0.0);

    for (follower_entity, follower, mut velocity, transform) in follower_query.iter_mut() {
        let Some(waypoint) = waypoints_query.iter().find(|waypoint| waypoint.index == follower.index) else {
            if let Ok(finish) = finish_query.get_single() {
                velocity.direction = ((finish.position + tilemap_top_left.xy()) - transform.translation.xy()).normalize();
            } else {
                // no finish : despawn now
                commands.entity(follower_entity).despawn_recursive();
            }

            continue;
        };

        velocity.direction = ((waypoint.position + tilemap_top_left.xy()) - transform.translation.xy()).normalize();
    }
}

pub fn reach_waypoint(
    mut commands: Commands,
    mut follower_query: Query<(Entity, &mut WaypointFollower, &Transform)>,
    finish_query: Query<&EnemyFinish>,
    waypoints_query: Query<&Waypoint>,
    tile_map_query: Query<(&GlobalTransform, &TilemapTileSize), With<TileStorage>>,
) {
    let Ok((tilemap_transform, tile_size)) = tile_map_query.get_single() else {
        return;
    };
    let tilemap_top_left = tilemap_transform.translation() - Vec3::new(tile_size.x / 2.0, tile_size.y / 2.0, 0.0);

    for (follower_entity, mut follower, transform) in follower_query.iter_mut() {

        // TODO : optimiser en mettant dans le composant directement la position du prochain waypoint

        let Some(waypoint) = waypoints_query.iter().find(|waypoint| waypoint.index == follower.index) else {
            // finish ?
            let Ok(finish) = finish_query.get_single() else {
                continue;
            };

            if ((finish.position + tilemap_top_left.xy()) - transform.translation.xy()).length_squared() < 1.0 {
                // finish reached : TODO : publish event, update score...
                commands.entity(follower_entity).despawn_recursive();
            }

            continue;
        };

        if ((waypoint.position + tilemap_top_left.xy()) - transform.translation.xy()).length_squared() < 1.0 {
            follower.index += 1;
        }
    }
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
            let distance_to_enemy = (enemy_transform.translation - thrower_transform.translation).length_squared();

            if distance_to_enemy > projectile_thrower.range * projectile_thrower.range {
                continue;
            }

            if closest_enemy.is_none() {
                closest_enemy = Some((enemy_entity, enemy_transform));
                continue;
            }

            let distance_to_closest_enemy = (closest_enemy.unwrap().1.translation - thrower_transform.translation).length_squared();
            if distance_to_enemy < distance_to_closest_enemy
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
                Projectile {
                    damage: 40,
                },
                Follower {
                    speed: 800.0,
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
    mut projectile_query: Query<(Entity, &Follower, &mut Transform, &Projectile)>,
    mut projectile_hit_event_writer: EventWriter<ProjectileHitEvent>,
    target_query: Query<&Transform, Without<Projectile>>,
    time: Res<Time>,
) {
    for (follower_entity, follower, mut follower_transform, projectile) in projectile_query.iter_mut() {
        if let Ok(target_transform) = target_query.get(follower.target) {
            let direction_to_target = (target_transform.translation - follower_transform.translation).xy().normalize();
            follower_transform.translation += Vec3::from((direction_to_target, 0.0)) * follower.speed * time.delta_seconds();
            follower_transform.rotation = Quat::from_rotation_arc_2d(
                Vec2::new(1.0, 0.0),
                direction_to_target,
            );

            // check if projectile is close enough to target
            if (target_transform.translation - follower_transform.translation).length_squared() < 20.0 * 20.0 {
                projectile_hit_event_writer.send(ProjectileHitEvent {
                    damage: projectile.damage as f32,
                    target: follower.target,
                });
                commands.entity(follower_entity).despawn_recursive();
            }
        } else {
            // target does not exist anymore (e.g. reached finish waypoint), despawn projectile
            commands.entity(follower_entity).despawn_recursive();
            continue;
        }
    }
}

pub fn deal_projectile_damage(
    mut projectile_hit_event_reader: EventReader<ProjectileHitEvent>,
    mut health_query: Query<&mut Health>,
    mut event_writer: EventWriter<KilledEvent>,
) {
    for event in projectile_hit_event_reader.iter() {
        let Ok(mut target_health) = health_query.get_mut(event.target) else {
            // does not exist anymore
            continue;
        };
        target_health.current -= (event.damage) as i32;
        if target_health.current <= 0 {
            event_writer.send(KilledEvent {
                who: event.target,
            });
        }
    }
}

pub fn on_enemy_killed(
    mut commands: Commands,
    mut event_reader: EventReader<KilledEvent>,
    enemy_query: Query<&Enemy>,
) {
    for event in event_reader.iter() {
        enemy_query.get(event.who).ok().map(|enemy| {
            // TODO : add points ?
            commands.entity(event.who).despawn_recursive();
        });
    }
}

pub fn do_move_step(
    mut move_query: Query<(&Velocity, &mut Transform)>,
    time: Res<Time>,
) {
    for (velocity, mut transform) in move_query.iter_mut() {
        transform.translation += Vec3::from((velocity.direction, 0.0)) * velocity.speed * time.delta_seconds();
        transform.rotation = Quat::from_rotation_arc_2d(
            Vec2::new(1.0, 0.0),
            velocity.direction,
        );
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

pub fn update_mouse_pos_display(
    window: Query<&Window>,
    mut cursor_evr: EventReader<CursorMoved>,
    camera_query: Query<&Transform, With<Camera>>,
) {
    let w_x = window.get_single().unwrap().width();
    let w_h = window.get_single().unwrap().height();
    for ev in cursor_evr.iter() {
        println!(
            "New cursor position: X: {}, Y: {}, in Window ID: {:?}",
            ev.position.x - w_x / 2.0 + camera_query.single().translation.x, ev.position.y - w_h / 2.0 + camera_query.single().translation.y, ev.window.index()
        );
    }
}

pub fn extract_health_bar(
    mut extracted_sprites: ResMut<ExtractedSprites>,
    healthbar_query: Extract<
        Query<(
            Entity,
            &Healthbar,
            &Health,
            &ComputedVisibility,
            &GlobalTransform,
        )>,
    >,
) {
    for (healthbar_entity, healthbar, health, healthbar_visibility, entity_transform) in healthbar_query.iter() {
        if !healthbar_visibility.is_visible() {
            continue;
        }

        let mut background_translation = entity_transform.translation();
        background_translation.x -= healthbar.length / 2.0;
        background_translation.y += 32.0;
        background_translation.z = 50.0;

        // background
        extracted_sprites.sprites.push(ExtractedSprite {
            entity: healthbar_entity,
            transform: GlobalTransform::from(Transform::from_translation(background_translation)),
            custom_size: Some(Vec2::new(healthbar.length, healthbar.height)),
            color: Color::rgb(0.2, 0.2, 0.2),
            anchor: Anchor::CenterLeft.as_vec(),
            flip_x: false,
            flip_y: false,
            rect: None,
            image_handle_id: DEFAULT_IMAGE_HANDLE.into(),
        });

        // current life
        let padding = 2.0;
        let health_percent = health.current as f32 / health.max as f32;
        let width = healthbar.length * health_percent;
        let mut healthbar_translation = background_translation.clone();
        healthbar_translation.x += padding; // "left border"
        healthbar_translation.z += 1.0; // in front of background

        extracted_sprites.sprites.push(ExtractedSprite {
            entity: healthbar_entity,
            transform: GlobalTransform::from(Transform::from_translation(healthbar_translation)),
            custom_size: Some(Vec2::new(width - padding * 2.0, healthbar.height - padding * 2.0)),
            color: Color::rgb(0.0, 1.0, 0.25),
            anchor: Anchor::CenterLeft.as_vec(),
            flip_x: false,
            flip_y: false,
            rect: None,
            image_handle_id: DEFAULT_IMAGE_HANDLE.into(),
        });
    }
}
