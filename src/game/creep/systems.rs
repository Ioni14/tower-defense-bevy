use std::cmp::max;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::render::Extract;
use bevy::render::texture::DEFAULT_IMAGE_HANDLE;
use bevy::sprite::{Anchor, ExtractedSprite, ExtractedSprites};
use bevy_ecs_tilemap::prelude::*;
use super::components::*;
use super::events::*;

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
            Health::full(500),
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
                println!("despawn creep because no more waypoint {:?}", follower_entity);
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
                println!("despawn creep because finish reached {:?}", follower_entity);
                commands.entity(follower_entity).despawn_recursive();
            }

            continue;
        };

        if ((waypoint.position + tilemap_top_left.xy()) - transform.translation.xy()).length_squared() < 1.0 {
            follower.index += 1;
        }
    }
}

pub fn on_enemy_killed(
    mut commands: Commands,
    mut event_reader: EventReader<KilledEvent>,
    enemy_query: Query<&Enemy, Without<Dying>>,
) {
    for event in event_reader.iter() {
        if let Some(mut who_entity) = commands.get_entity(event.who) {
            // println!("despawn creep because killed {:?}", event.who);
            // who_entity.despawn_recursive();
            println!("set dying creep because killed {:?}", event.who);
            who_entity.insert(Dying);
        }
        // enemy_query.get(event.who).ok().map(|enemy| {
        //     // TODO : add points ?
        //     commands.entity(event.who).despawn_recursive();
        // });
    }
}

pub fn despawn_dying(
    mut commands: Commands,
    dying_query: Query<Entity, With<Dying>>,
) {
    for dying in dying_query.iter() {
        println!("despawn creep because killed {:?}", dying);
        commands.entity(dying).despawn_recursive();
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
        let health_percent = 0.0f32.max(health.current as f32 / health.max as f32);
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
