use crate::{
    asset_loading::AssetState,
    collision::{Collider, RectCollider, StaticBody},
};
use bevy::{prelude::*, sprite::Wireframe2dPlugin};
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_tilemap::map::TilemapTileSize;

#[derive(AssetCollection, Resource)]
pub struct LdtkAssets {
    #[asset(path = "ldtk/annual.ldtk")]
    pub annual: Handle<LdtkProject>,
}

pub struct LdtkPlugin;

impl Plugin for LdtkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_ecs_ldtk::LdtkPlugin)
            // main level should be `level_0`
            .insert_resource(LevelSelection::index(0))
            .add_plugins(Wireframe2dPlugin)
            .insert_resource(LdtkSettings {
                // level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                //     load_level_neighbors: true,
                // },
                ..default()
            })
            .add_systems(OnEnter(AssetState::Loaded), startup)
            .add_systems(PreUpdate, build_tile_set_colliders);
    }
}

fn startup(mut commands: Commands, assets: Res<LdtkAssets>) {
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: assets.annual.clone(),
        transform: Transform::from_xyz(0., 0., -100.),
        ..Default::default()
    });
}

// TODO: collider collapsing vertically
fn build_tile_set_colliders(
    mut commands: Commands,
    levels: Query<(&LevelIid, &Children), Added<Children>>,
    layers: Query<(&LayerMetadata, &TilemapTileSize, &Children)>,
    tiles: Query<(&Transform, &TileEnumTags)>,
) {
    if levels.is_empty() {
        return;
    }

    let mut num_colliders = 0;

    // ~14k without combining
    // ~600 with horizontal combining

    let mut cached_collider_positions = Vec::with_capacity(1024);
    let tile_size = 8.;

    for (id, children) in levels.iter() {
        println!("{id}");
        for child in children.iter() {
            if let Ok((layer_meta, layer_tile_size, children)) = layers.get(*child) {
                println!("processing layer: {}", &layer_meta.identifier);
                let offset = tile_size / 2.;
                for tile in children.iter() {
                    if let Ok((transform, tile_tags)) = tiles.get(*tile) {
                        if tile_tags.tags.iter().any(|t| t == "Solid")
                            && layer_tile_size.x == tile_size
                            && layer_tile_size.y == tile_size
                        {
                            cached_collider_positions.push(Vec2::new(
                                transform.translation.x + offset,
                                transform.translation.y + offset,
                            ));

                            // let collider =
                            //     Collider::new(transform.translation, tile_size.x, tile_size.y);
                            // commands.spawn(collider);
                        } else {
                            warn!("unknown tagged ldtk tile: {tile_tags:?}");
                        }
                    }
                }
            }
        }
    }

    if cached_collider_positions.is_empty() {
        return;
    }

    // for v in cached_collider_positions.into_iter() {
    //     commands.spawn((
    //         SpatialBundle::from_transform(Transform::from_xyz(v.x, v.y, 0.)),
    //         Collider::from_rect(RectCollider {
    //             tl: Vec2::ZERO,
    //             size: Vec2::new(tile_size, tile_size),
    //         }),
    //         StaticBody,
    //     ));
    //     num_colliders += 1;
    // }

    for (pos, collider) in
        build_colliders_from_vec2(cached_collider_positions, tile_size).into_iter()
    {
        commands.spawn((
            SpatialBundle::from_transform(Transform::from_xyz(pos.x, pos.y, 0.)),
            collider,
            StaticBody,
        ));
        num_colliders += 1;
    }

    println!("num_colliders: {num_colliders}");
}

pub fn build_colliders_from_vec2(
    mut positions: Vec<Vec2>,
    tile_size: f32,
) -> Vec<(Vec2, Collider)> {
    positions.sort_by(|a, b| {
        let y_cmp = a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal);
        if y_cmp == std::cmp::Ordering::Equal {
            a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal)
        } else {
            y_cmp
        }
    });

    let mut rows = Vec::with_capacity(positions.len() / 2);
    let mut current_y = None;
    let mut current_xs = Vec::with_capacity(positions.len() / 2);
    for v in positions.into_iter() {
        match current_y {
            None => {
                current_y = Some(v.y);
                current_xs.push(v.x);
            }
            Some(y) => {
                if v.y == y {
                    current_xs.push(v.x);
                } else {
                    rows.push((y, current_xs.clone()));
                    current_xs.clear();

                    current_y = Some(v.y);
                    current_xs.push(v.x);
                }
            }
        }
    }

    match current_y {
        Some(y) => {
            rows.push((y, current_xs));
        }
        None => unreachable!(),
    }

    #[derive(Debug, Clone, Copy)]
    struct Plate {
        y: f32,
        x_start: f32,
        x_end: f32,
    }

    let mut row_plates = Vec::with_capacity(rows.len());
    for (y, row) in rows.into_iter() {
        let mut current_x = None;
        let mut x_start = None;
        let mut plates = Vec::with_capacity(row.len() / 4);

        for x in row.iter() {
            match (current_x, x_start) {
                (None, None) => {
                    current_x = Some(*x);
                    x_start = Some(*x);
                }
                (Some(cx), Some(xs)) => {
                    if *x > cx + tile_size {
                        plates.push(Plate {
                            x_end: cx + tile_size,
                            x_start: xs,
                            y,
                        });
                        x_start = Some(*x);
                    }

                    current_x = Some(*x);
                }
                _ => unreachable!(),
            }
        }

        match (current_x, x_start) {
            (Some(cx), Some(xs)) => {
                plates.push(Plate {
                    x_end: cx + tile_size,
                    x_start: xs,
                    y,
                });
            }
            _ => unreachable!(),
        }

        row_plates.push(plates);
    }

    let mut output = Vec::new();
    for plates in row_plates.into_iter() {
        for plate in plates.into_iter() {
            output.push((
                Vec2::new(plate.x_start, plate.y),
                Collider::from_rect(RectCollider {
                    tl: Vec2::ZERO,
                    size: Vec2::new(plate.x_end - plate.x_start, tile_size),
                }),
            ));
        }
    }

    output
}
