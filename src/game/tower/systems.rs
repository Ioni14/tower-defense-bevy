use bevy::input::ButtonState;
use bevy::input::mouse::MouseButtonInput;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_ecs_tilemap::prelude::*;
use crate::game::creep::components::{Enemy, Health};
use crate::game::creep::events::KilledEvent;
use crate::game::tilemap::components::{BuiltTile, SelectedForBuild};
use super::events::ProjectileHitEvent;
use super::components::*;

/**
 * On click, spawn a new tower at the selected_for_build tile.
 */
pub fn build_tower_at_click(
    mut commands: Commands,
    mut clicked_event_reader: EventReader<MouseButtonInput>,
    selected_for_build_tile_q: Query<(Entity, &TilePos), With<SelectedForBuild>>,
    tilemap_q: Query<(&TilemapGridSize, &TilemapType, &GlobalTransform)>,
    asset_server: Res<AssetServer>,
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
        commands.spawn(
            (
                Tower {},
                ProjectileThrower {
                    relative_start: Vec2::new(0.0, 0.25 * 64.0),
                    cooldown: Timer::from_seconds(1.0, TimerMode::Repeating),
                    range: 450.0,
                },
                SpriteBundle {
                    transform: Transform::from_translation(Vec3::from((tile_world_pos, 10.0)) + tilemap_transform.translation()),
                    texture: asset_server.load("sprites/tower.png"),
                    sprite: Sprite {
                        anchor: Anchor::Center,
                        ..default()
                    },
                    ..default()
                },
                Name::new("Tower"),
            ),
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

