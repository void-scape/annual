use crate::color::srgb_from_hex;
use crate::TILE_SIZE;
use bevy::prelude::*;
use bevy_light_2d::light::PointLight2d;

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
                ..default()
            },
            Transform::from_xyz(TILE_SIZE / 2., -TILE_SIZE / 2. - TILE_SIZE * 3., 0.),
        ));
    }
}
