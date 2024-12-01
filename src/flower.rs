use crate::{asset_loading::loaded, player::Player};
use bevy::prelude::*;
use bevy_ecs_ldtk::{app::LdtkEntityAppExt, LdtkEntity};

pub struct FlowerPlugin;

impl Plugin for FlowerPlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_entity::<FlowerBundle>("Flower")
            .add_systems(
                Update,
                talk.run_if(crate::player::on_player_interact)
                    .run_if(loaded()),
            );
    }
}

#[derive(Default, Bundle, LdtkEntity)]
pub struct FlowerBundle {
    flower: Flower,
    #[sprite_bundle]
    sprite: SpriteBundle,
}

#[derive(Component, Default)]
pub struct Flower;

fn talk(flower: Query<&Transform, With<Flower>>, player: Query<&Transform, With<Player>>) {
    if let Ok(flower) = flower.get_single() {
        if let Ok(player) = player.get_single() {
            if flower.translation.distance_squared(player.translation) < 800.0 {
                println!("hi");
            }
        }
    }
}
