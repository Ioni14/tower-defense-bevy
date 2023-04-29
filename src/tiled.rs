use std::io::BufReader;

use anyhow::Result;
use bevy::{
    asset::{AssetLoader, AssetPath, LoadedAsset},
    log,
    prelude::*,
    reflect::TypeUuid,
    time::*,
    utils::HashMap,
};
use bevy_ecs_tilemap::prelude::*;
use tiled::ObjectShape;
use tiled::PropertyValue::IntValue;

// use tiled::PropertyValue;
use crate::{BuildZone, EnemyFinish, EnemySpawner, Waypoint};

#[derive(Default)]
pub struct TiledMapPlugin;

impl Plugin for TiledMapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_asset::<TiledMap>()
            .add_asset_loader(TiledLoader)
            .add_system(process_loaded_maps);
    }
}

#[derive(TypeUuid)]
#[uuid = "e51081d0-6168-4881-a1c6-4249b2000d7f"]
pub struct TiledMap {
    pub map: tiled::Map,

    pub tilemap_textures: HashMap<usize, TilemapTexture>,

    // The offset into the tileset_images for each tile id within each tileset.
    #[cfg(not(feature = "atlas"))]
    pub tile_image_offsets: HashMap<(usize, tiled::TileId), u32>,
}

// Stores a list of tiled layers.
#[derive(Component, Default)]
pub struct TiledLayersStorage {
    pub storage: HashMap<u32, Entity>,
}

#[derive(Default, Bundle)]
pub struct TiledMapBundle {
    pub tiled_map: Handle<TiledMap>,
    pub storage: TiledLayersStorage,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

pub struct TiledLoader;

impl AssetLoader for TiledLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            // The load context path is the TMX file itself. If the file is at the root of the
            // assets/ directory structure then the tmx_dir will be empty, which is fine.
            let tmx_dir = load_context
                .path()
                .parent()
                .expect("The asset load context was empty.")
                .parent()
                .expect("The asset load context was empty.");

            let mut loader = tiled::Loader::new();
            let map = loader
                .load_tmx_map_from(BufReader::new(bytes), load_context.path())
                .map_err(|e| anyhow::anyhow!("Could not load TMX map: {e}"))?;

            let mut dependencies = Vec::new();
            let mut tilemap_textures = HashMap::default();
            #[cfg(not(feature = "atlas"))]
                let mut tile_image_offsets = HashMap::default();

            for (tileset_index, tileset) in map.tilesets().iter().enumerate() {
                let tilemap_texture = match &tileset.image {
                    None => {
                        #[cfg(feature = "atlas")]
                        {
                            log::info!("Skipping image collection tileset '{}' which is incompatible with atlas feature", tileset.name);
                            continue;
                        }

                        #[cfg(not(feature = "atlas"))]
                        {
                            let mut tile_images: Vec<Handle<Image>> = Vec::new();
                            for (tile_id, tile) in tileset.tiles() {
                                if let Some(img) = &tile.image {
                                    let tile_path = tmx_dir.join(&img.source);
                                    let asset_path = AssetPath::new(tile_path, None);
                                    log::info!("Loading tile image from {asset_path:?} as image ({tileset_index}, {tile_id})");
                                    let texture: Handle<Image> =
                                        load_context.get_handle(asset_path.clone());
                                    tile_image_offsets
                                        .insert((tileset_index, tile_id), tile_images.len() as u32);
                                    tile_images.push(texture.clone());
                                    dependencies.push(asset_path);
                                }
                            }

                            TilemapTexture::Vector(tile_images)
                        }
                    }
                    Some(img) => {
                        let tile_path = tmx_dir.join(&img.source);
                        let asset_path = AssetPath::new(tile_path, None);
                        let texture: Handle<Image> = load_context.get_handle(asset_path.clone());
                        dependencies.push(asset_path);

                        TilemapTexture::Single(texture.clone())
                    }
                };

                tilemap_textures.insert(tileset_index, tilemap_texture);
            }

            let asset_map = TiledMap {
                map,
                tilemap_textures,
                #[cfg(not(feature = "atlas"))]
                tile_image_offsets,
            };

            log::info!("Loaded map: {}", load_context.path().display());

            let loaded_asset = LoadedAsset::new(asset_map);
            load_context.set_default_asset(loaded_asset.with_dependencies(dependencies));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["tmx"];
        EXTENSIONS
    }
}

pub fn process_loaded_maps(
    mut commands: Commands,
    // mut map_events: EventReader<AssetEvent<TiledMap>>,
    maps: Res<Assets<TiledMap>>,
    tile_storage_query: Query<(Entity, &TileStorage)>,
    mut map_query: Query<(&Handle<TiledMap>, &mut TiledLayersStorage)>,
    new_maps: Query<&Handle<TiledMap>, Added<Handle<TiledMap>>>,
) {
    let mut changed_maps = Vec::<Handle<TiledMap>>::default();
    // for event in map_events.iter() {
    //     match event {
    //         AssetEvent::Created { handle } => {
    //             log::info!("Map added!");
    //             changed_maps.push(handle.clone());
    //         }
    //         AssetEvent::Modified { handle } => {
    //             log::info!("Map changed!");
    //             changed_maps.push(handle.clone());
    //         }
    //         AssetEvent::Removed { handle } => {
    //             log::info!("Map removed!");
    //             // if mesh was modified and removed in the same update, ignore the modification
    //             // events are ordered so future modification events are ok
    //             changed_maps.retain(|changed_handle| changed_handle == handle);
    //         }
    //     }
    // }

    // If we have new map entities add them to the changed_maps list.
    for new_map_handle in new_maps.iter() {
        changed_maps.push(new_map_handle.clone_weak());
    }

    for changed_map in changed_maps.iter() {
        for (map_handle, mut layer_storage) in map_query.iter_mut() {
            // only deal with currently changed map
            if map_handle != changed_map {
                continue;
            }
            if let Some(tiled_map) = maps.get(map_handle) {
                for layer_entity in layer_storage.storage.values() {
                    if let Ok((_, layer_tile_storage)) = tile_storage_query.get(*layer_entity) {
                        for tile in layer_tile_storage.iter().flatten() {
                            commands.entity(*tile).despawn_recursive()
                        }
                    }
                    // commands.entity(*layer_entity).despawn_recursive();
                }

                // The TilemapBundle requires that all tile images come exclusively from a single
                // tiled texture or from a Vec of independent per-tile images. Furthermore, all of
                // the per-tile images must be the same size. Since Tiled allows tiles of mixed
                // tilesets on each layer and allows differently-sized tile images in each tileset,
                // this means we need to load each combination of tileset and layer separately.
                for (tileset_index, tileset) in tiled_map.map.tilesets().iter().enumerate() {
                    let Some(tilemap_texture) = tiled_map
                        .tilemap_textures
                        .get(&tileset_index) else {
                        log::warn!("Skipped creating layer with missing tilemap textures.");
                        continue;
                    };

                    let tile_size = TilemapTileSize {
                        x: tileset.tile_width as f32,
                        y: tileset.tile_height as f32,
                    };

                    let tile_spacing = TilemapSpacing {
                        x: tileset.spacing as f32,
                        y: tileset.spacing as f32,
                    };

                    // Once materials have been created/added we need to then create the layers.
                    for (layer_index, layer) in tiled_map.map.layers().enumerate() {
                        let offset_x = layer.offset_x;
                        let offset_y = layer.offset_y;

                        if let tiled::LayerType::ObjectLayer(object_layer) = layer.layer_type() {
                            for object_data in object_layer.object_data() {
                                let mapped_x = object_data.x + offset_x;
                                let mapped_y = tiled_map.map.height as f32 * tile_size.y - (object_data.y + offset_y);

                                match object_data.user_type.as_str() {
                                    "Waypoint" => {
                                        let IntValue(index) = object_data.properties["waypoint"] else {
                                            log::warn!("Skipped entity waypoint because no waypoint property found.");
                                            continue;
                                        };
                                        commands.spawn(Waypoint {
                                            index,
                                            position: Vec2::new(mapped_x, mapped_y),
                                        }).insert(Name::new(object_data.name.clone()));
                                    }
                                    "EnemyFinish" => {
                                        commands.spawn(EnemyFinish {
                                            position: Vec2::new(mapped_x, mapped_y),
                                        }).insert(Name::new(object_data.name.clone()));
                                    }
                                    "EnemySpawner" => {
                                        commands.spawn(EnemySpawner {
                                            position: Vec2::new(mapped_x, mapped_y),
                                            timer: Timer::from_seconds(2.0, TimerMode::Repeating),
                                        }).insert(Name::new(object_data.name.clone()));
                                    }
                                    "BuildZone" => {
                                        let (shape_width, shape_height) = match object_data.shape {
                                            ObjectShape::Rect { width, height } => (width, height),
                                            _ => (0.0, 0.0)
                                        };
                                        commands.spawn(BuildZone {
                                            rect: Rect::new(mapped_x, mapped_y, mapped_x + shape_width, mapped_y - shape_height),
                                        }).insert(Name::new(object_data.name.clone()));
                                    }
                                    _ => {}
                                }
                            }

                            continue;
                        }

                        let tiled::LayerType::TileLayer(tile_layer) = layer.layer_type() else {
                            log::info!(
                                "Skipping layer {} because only tile layers are supported.",
                                layer.id()
                            );
                            continue;
                        };

                        let tiled::TileLayer::Finite(layer_data) = tile_layer else {
                            log::info!(
                                "Skipping layer {} because only finite layers are supported.",
                                layer.id()
                            );
                            continue;
                        };

                        let map_size = TilemapSize {
                            x: tiled_map.map.width,
                            y: tiled_map.map.height,
                        };

                        let grid_size = TilemapGridSize {
                            x: tiled_map.map.tile_width as f32,
                            y: tiled_map.map.tile_height as f32,
                        };

                        let map_type = match tiled_map.map.orientation {
                            tiled::Orientation::Hexagonal => {
                                TilemapType::Hexagon(HexCoordSystem::Row)
                            }
                            tiled::Orientation::Isometric => {
                                TilemapType::Isometric(IsoCoordSystem::Diamond)
                            }
                            tiled::Orientation::Staggered => {
                                TilemapType::Isometric(IsoCoordSystem::Staggered)
                            }
                            tiled::Orientation::Orthogonal => TilemapType::Square,
                        };

                        let mut tile_storage = TileStorage::empty(map_size);
                        let layer_entity = commands.spawn_empty().id();

                        for x in 0..map_size.x {
                            for y in 0..map_size.y {
                                // Transform TMX coords into bevy coords.
                                let mapped_y = tiled_map.map.height - 1 - y;

                                let mapped_x = x as i32;
                                let mapped_y = mapped_y as i32;

                                let layer_tile = match layer_data.get_tile(mapped_x, mapped_y) {
                                    Some(t) => t,
                                    None => {
                                        continue;
                                    }
                                };
                                if tileset_index != layer_tile.tileset_index() {
                                    continue;
                                }
                                let layer_tile_data =
                                    match layer_data.get_tile_data(mapped_x, mapped_y) {
                                        Some(d) => d,
                                        None => {
                                            continue;
                                        }
                                    };

                                // TODO : if we want properties of tiles
                                // let tile = match layer_tile.get_tile() {
                                //     Some(t) => t,
                                //     None => {
                                //         continue;
                                //     }
                                // };
                                // let tile_properties = &tile.properties;
                                // if ! tile_properties.is_empty() {
                                //     println!("tile properties of {:?}: {:?}", layer_tile.id(), tile_properties);
                                // }

                                let texture_index = match tilemap_texture {
                                    TilemapTexture::Single(_) => layer_tile.id(),
                                    #[cfg(not(feature = "atlas"))]
                                    TilemapTexture::Vector(_) =>
                                        *tiled_map.tile_image_offsets.get(&(tileset_index, layer_tile.id()))
                                            .expect("The offset into to image vector should have been saved during the initial load."),
                                    #[cfg(not(feature = "atlas"))]
                                    _ => unreachable!()
                                };

                                let tile_pos = TilePos { x, y };
                                let tile_entity_builder = commands
                                    .spawn(TileBundle {
                                        position: tile_pos,
                                        tilemap_id: TilemapId(layer_entity),
                                        texture_index: TileTextureIndex(texture_index),
                                        flip: TileFlip {
                                            x: layer_tile_data.flip_h,
                                            y: layer_tile_data.flip_v,
                                            d: layer_tile_data.flip_d,
                                        },
                                        color: TileColor(Color::WHITE),
                                        ..Default::default()
                                    });

                                let tile_entity = tile_entity_builder.id();
                                tile_storage.set(&tile_pos, tile_entity);
                            }
                        }

                        println!("insert TilemapBundle {:?}", layer_entity.index());
                        commands.entity(layer_entity).insert(TilemapBundle {
                            grid_size,
                            size: map_size,
                            storage: tile_storage,
                            texture: tilemap_texture.clone(),
                            tile_size,
                            spacing: tile_spacing,
                            transform: get_tilemap_center_transform(
                                &map_size,
                                &grid_size,
                                &map_type,
                                layer_index as f32,
                            ) * Transform::from_xyz(offset_x, -offset_y, 0.0),
                            map_type,
                            ..Default::default()
                        });

                        layer_storage
                            .storage
                            .insert(layer_index as u32, layer_entity);
                    }
                }
            }
        }
    }
}
