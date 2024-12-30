use crate::color::srgb_from_hex;
use crate::TILE_SIZE;
use bevy::prelude::*;
use bevy_light_2d::light::PointLight2d;
use bevy_light_2d::prelude::{LightOccluder2d, LightOccluder2dShape};

pub fn init_point_light_tiles(
    mut commands: Commands,
    tile_query: Query<Entity, Added<crate::annual::TilePointLight>>,
) {
    for entity in tile_query.iter() {
        commands.entity(entity).with_child((
            PointLight2d {
                color: srgb_from_hex(0xffeb57),
                intensity: 2.,
                radius: 100.,
                falloff: 100.,
                cast_shadows: true,
                ..default()
            },
            Transform::from_xyz(TILE_SIZE / 2., -TILE_SIZE / 2. - TILE_SIZE * 3., 0.),
        ));
    }
}

pub fn init_point_light_entities(
    mut commands: Commands,
    light_query: Query<Entity, Added<crate::annual::PointLight>>,
) {
    for entity in light_query.iter() {
        commands.entity(entity).with_child((
            PointLight2d {
                color: srgb_from_hex(0xf6cd26),
                intensity: 5.,
                radius: 150.,
                falloff: 50.,
                ..default()
            },
            Transform::from_xyz(TILE_SIZE / 2., -TILE_SIZE / 2., 0.),
        ));
    }
}

pub fn init_occluders(
    mut commands: Commands,
    occluder_query: Query<Entity, Added<crate::annual::Occluder>>,
) {
    for entity in occluder_query.iter() {
        commands.entity(entity).with_child((
            LightOccluder2d {
                shape: LightOccluder2dShape::Rectangle {
                    half_size: Vec2::new(TILE_SIZE / 2., TILE_SIZE / 2.),
                },
            },
            Transform::from_xyz(TILE_SIZE / 2., -TILE_SIZE / 2., 0.),
        ));
    }
}
