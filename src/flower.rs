use bevy::prelude::*;
use crate::ldtk::entities::LdtkFlower;

pub struct FlowerPlugin;

impl Plugin for FlowerPlugin {
    fn build(&self, app: &mut App) {
        app.register_required_components::<LdtkFlower, Flower>().add_systems(Update, log);
        // .register_ldtk_entity::<FlowerBundle>(Entities::Flower.identifier())
        // .add_systems(
        //     Update,
        //     talk.run_if(crate::player::on_player_interact)
        //         .run_if(loaded()),
        // );
    }
}

#[derive(Component, Default)]
pub struct Flower;

fn log(flower: Query<Entity, Added<Flower>>) {
    for flower in flower.iter() {
        info!("Spawned flower!");
    }
}

// fn talk(flower: Query<&Transform, With<Flower>>, player: Query<&Transform, With<Player>>) {
//     if let Ok(flower) = flower.get_single() {
//         if let Ok(player) = player.get_single() {
//             if flower.translation.distance_squared(player.translation) < 800.0 {
//                 println!("hi");
//             }
//         }
//     }
// }
