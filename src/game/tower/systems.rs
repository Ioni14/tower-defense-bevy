use bevy::input::ButtonState;
use bevy::input::mouse::MouseButtonInput;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::render::texture::DEFAULT_IMAGE_HANDLE;
use bevy::sprite::Anchor;
use bevy_ecs_tilemap::prelude::*;

use crate::game::creep::components::{Dying, Enemy, Health};
use crate::game::creep::events::KilledEvent;
use crate::game::resources::{BuildTower, TowerType};
use crate::game::tilemap::components::{BuiltTile, SelectedForBuild};

use super::components::*;
use super::events::ProjectileHitEvent;

/**
 * On click, spawn a new tower at the selected_for_build tile.
 */
pub fn build_tower_at_click(
    mut commands: Commands,
    mut clicked_event_reader: EventReader<MouseButtonInput>,
    selected_for_build_tile_q: Query<(Entity, &TilePos), With<SelectedForBuild>>,
    tilemap_q: Query<(&TilemapGridSize, &TilemapType, &GlobalTransform)>,
    asset_server: Res<AssetServer>,
    build_tower: Res<BuildTower>
) {
    for click in clicked_event_reader.iter() {
        if click.button != MouseButton::Left || click.state != ButtonState::Released {
            continue;
        }
        let Ok((tile_entity, tile_pos)) = selected_for_build_tile_q.get_single() else {
            return;
        };
        let Ok((tilemap_grid_size, tilemap_type, tilemap_transform)) = tilemap_q.get_single() else {
            return;
        };

        let tile_world_pos = tile_pos.center_in_world(&tilemap_grid_size, &tilemap_type);

        commands.entity(tile_entity).insert(BuiltTile {});

        let tower_id = commands.spawn((
            Tower {},
            Name::new("Tower")
        )).id();

        println!("build_tower_at_click: {:?}", build_tower.tower_type);

        let mut texture: Handle<Image>;
        match build_tower.tower_type {
            TowerType::Arrow => {
                texture = asset_server.load("sprites/tower.png");
                commands.entity(tower_id).insert(
                    ProjectileThrower {
                        relative_start: Vec2::new(0.0, 0.25 * 64.0),
                        cooldown: Timer::from_seconds(1.0, TimerMode::Repeating),
                        range: 450.0,
                    },
                );
            }
            TowerType::Bomb => {
                texture = asset_server.load("sprites/tower_bomb.png");
                commands.entity(tower_id).insert(
                    Splasher {
                        relative_start: Vec2::new(0.0, 0.25 * 64.0),
                        cooldown: Timer::from_seconds(3.0, TimerMode::Repeating),
                        range: 300.0,
                    },
                );
            }
        }

        commands.entity(tower_id).insert(
            SpriteBundle {
                transform: Transform::from_translation(Vec3::from((tile_world_pos, 10.0)) + tilemap_transform.translation()),
                texture,
                sprite: Sprite {
                    custom_size: Some(Vec2::new(64.0, 64.0)),
                    anchor: Anchor::Center,
                    ..default()
                },
                ..default()
            },
        );
    }
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

pub fn throw_splashes(
    mut commands: Commands,
    mut splasher_query: Query<(&mut Splasher, &Transform)>,
    asset_server: Res<AssetServer>,
    enemies_query: Query<(Entity, &Transform), With<Enemy>>,
    time: Res<Time>,
) {
    for (mut splasher, thrower_transform) in splasher_query.iter_mut() {
        splasher.cooldown.tick(time.delta());
        if !splasher.cooldown.finished() {
            continue;
        }
        let mut closest_enemy: Option<(Entity, &Transform)> = None;
        for (enemy_entity, enemy_transform) in enemies_query.iter() {
            let distance_to_enemy = (enemy_transform.translation - thrower_transform.translation).xy().length_squared();

            if distance_to_enemy > splasher.range * splasher.range {
                continue;
            }

            if closest_enemy.is_none() {
                closest_enemy = Some((enemy_entity, enemy_transform));
                continue;
            }

            let distance_to_closest_enemy = (closest_enemy.unwrap().1.translation - thrower_transform.translation).xy().length_squared();
            if distance_to_enemy < distance_to_closest_enemy
            {
                closest_enemy = Some((enemy_entity, enemy_transform));
            }
        }

        if closest_enemy.is_none() {
            continue;
        }

        commands.spawn(
            (
                Projectile {
                    damage: 40,
                },
                Pointer {
                    speed: 100.0,
                    target: closest_enemy.unwrap().1.translation.xy(),
                    pos: thrower_transform.translation.xy() + splasher.relative_start,
                    source: thrower_transform.translation.xy() + splasher.relative_start,
                },
                SpriteBundle {
                    transform: Transform::from_translation(Vec3::from((thrower_transform.translation.xy() + splasher.relative_start, 10.0))).with_scale(Vec3::splat(0.25)),
                    texture: asset_server.load("sprites/bomb.png"),
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
    target_query: Query<&Transform, (Without<Projectile>, Without<Dying>)>,
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
            if (target_transform.translation - follower_transform.translation).xy().length_squared() < 20.0 * 20.0 {
                projectile_hit_event_writer.send(ProjectileHitEvent {
                    damage: projectile.damage as f32,
                    target: follower.target,
                });
                // println!("despawn projectile because hit target {:?}", follower_entity);
                commands.entity(follower_entity).despawn_recursive();
            }
        } else {
            // target does not exist anymore (e.g. reached finish waypoint), despawn projectile
            // println!("despawn projectile because no more target {:?}", follower_entity);
            commands.entity(follower_entity).despawn_recursive();
            continue;
        }
    }
}

pub fn pointer_follow_step(
    mut commands: Commands,
    mut pointer_query: Query<(Entity, &mut Pointer, &mut Transform, &Projectile)>,
    time: Res<Time>,
) {
    for (follower_entity, mut pointer, mut follower_transform, projectile) in pointer_query.iter_mut() {
        let direction_to_target = (pointer.target - pointer.pos).normalize();

        let speed = pointer.speed;
        pointer.pos += direction_to_target * speed * time.delta_seconds();

        follower_transform.translation.x = pointer.pos.x;

        let percent_to_target = 1.0 - (pointer.target - pointer.pos).length() / (pointer.target - pointer.source).length();
        println!("percent_to_target: {}", percent_to_target);
        // println!("delta y: {}", -100.0 * (percent_to_target - 0.5) * pointer.speed * time.delta_seconds());
        // follower_transform.translation.y += -300.0 * (percent_to_target - 0.5) * time.delta_seconds();
        let coef_parabolic = 2.0 * (0.5 - (percent_to_target - 0.5).abs());
        println!("coef_parabolic: {}", coef_parabolic);
        follower_transform.translation.y = pointer.pos.y + 30.0 * coef_parabolic * coef_parabolic;

        // check if projectile is close enough to target
        if (pointer.target - pointer.pos).length_squared() < 10.0 * 10.0 {
            // projectile_hit_event_writer.send(ProjectileHitEvent {
            //     damage: projectile.damage as f32,
            //     target: follower.target,
            // });
            // println!("despawn projectile because hit target {:?}", follower_entity);
            commands.entity(follower_entity).despawn_recursive();
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

