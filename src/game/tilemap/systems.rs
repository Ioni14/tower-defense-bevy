use bevy::math::Vec4Swizzles;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use super::tiled::*;
use super::components::*;
use super::resources::*;

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

pub fn setup_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let map_handle: Handle<TiledMap> = asset_server.load("tiled/map_1.tmx");

    commands.spawn(TiledMapBundle {
        tiled_map: map_handle,
        ..default()
    });
}

fn remove_selected_for_build(
    mut commands: &mut Commands,
    mut tile_color_q: &mut Query<&mut TileColor>,
    selected_for_build_q: &Query<Entity, With<SelectedForBuild>>,
) {
    for selected_for_build_entity in selected_for_build_q.iter() {
        if let Ok(mut color) = tile_color_q.get_mut(selected_for_build_entity) {
            *color = TileColor(Color::WHITE);
        }
        commands.entity(selected_for_build_entity).remove::<SelectedForBuild>();
    }
}

pub fn unselect_build_zone(
    mut commands: Commands,
    mut tile_color_q: Query<&mut TileColor>,
    selected_for_build_q: Query<Entity, With<SelectedForBuild>>,
) {
    remove_selected_for_build(&mut commands, &mut tile_color_q, &selected_for_build_q);
}

/**
 * When we move the mouse over the tilemap, the targeted tile should be highlighted.
 * see https://github.com/StarArawn/bevy_ecs_tilemap/blob/main/examples/mouse_to_tile.rs#L315
 */
pub fn select_build_zone(
    mut commands: Commands,
    mut tile_color_q: Query<&mut TileColor>,
    cursor_pos: Res<CursorPos>,
    selected_for_build_q: Query<Entity, With<SelectedForBuild>>,
    build_zones_q: Query<&BuildZone>,
    built_tiles_q: Query<&BuiltTile>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &TileStorage,
        &GlobalTransform,
        &TilemapTileSize,
    )>,
) {
    // TODO : only if cursor pos has changed

    remove_selected_for_build(&mut commands, &mut tile_color_q, &selected_for_build_q);

    for (map_size, grid_size, map_type, tile_storage, map_transform, tile_size) in tilemap_q.iter() {
        // Grab the cursor position from the `Res<CursorPos>`
        let cursor_pos: Vec2 = cursor_pos.0;
        // We need to make sure that the cursor's world position is correct relative to the map
        // due to any map transformation.
        let cursor_in_map_pos: Vec2 = {
            // Extend the cursor_pos vec3 by 0.0 and 1.0
            let cursor_pos = Vec4::from((cursor_pos, 0.0, 1.0));
            let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
            cursor_in_map_pos.xy()
        };

        // Once we have a world position we can transform it into a possible tile position.
        let Some(tile_pos) = TilePos::from_world_pos(&cursor_in_map_pos, &map_size, &grid_size, &map_type) else {
            continue;
        };

        // check if tile is in build zone
        let tile_center_pos_world = tile_pos.center_in_world(&grid_size, &map_type) + Vec2::new(tile_size.x / 2.0, tile_size.y / 2.0);
        let mut in_build_zone = false;
        for build_zone in build_zones_q.iter() {
            let mut rect = build_zone.rect;
            if rect.contains(tile_center_pos_world) {
                in_build_zone = true;
                break;
            }
        }
        if !in_build_zone {
            continue;
        }

        let Some(tile_entity) = tile_storage.get(&tile_pos) else {
            continue;
        };

        if built_tiles_q.get(tile_entity).is_ok() {
            // a building is already on this tile
            continue;
        }

        commands.entity(tile_entity).insert(SelectedForBuild {
            // previousColor: Some(color),
        });
        if let Ok(mut color) = tile_color_q.get_mut(tile_entity) {
            *color = TileColor(Color::rgba(0.0, 1.0, 0.5, 0.5));
        }
    }
}

pub fn update_cursor_pos(
    mut cursor_event_reader: EventReader<CursorMoved>,
    mut cursor_pos: ResMut<CursorPos>,
    camera_q: Query<(&GlobalTransform, &Camera)>,
) {
    for cursor_moved in cursor_event_reader.iter() {
        // To get the mouse's world position, we have to transform its window position by
        // any transforms on the camera. This is done by projecting the cursor position into
        // camera space (world space).
        for (cam_t, cam) in camera_q.iter() {
            if let Some(pos) = cam.viewport_to_world_2d(cam_t, cursor_moved.position) {
                *cursor_pos = CursorPos(pos);
            }
        }
    }
}
