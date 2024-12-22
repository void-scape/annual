use crate::annual;
use bevy::prelude::*;

pub mod player;

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(player::PlayerPlugin)
            .register_required_components::<annual::Player, player::Player>()
            .register_required_components::<annual::Flower, Flower>();
    }
}

pub trait CharacterAssets {
    const POR: &str;
    const SFX: &str;
}

#[derive(Default, Component, macros::Character)]
#[require(Transform, Visibility)]
pub struct Flower;

impl CharacterAssets for Flower {
    const POR: &str = "flower.png";
    const SFX: &str = "flowey.mp3";
}

#[derive(Default, Component, macros::Character)]
#[require(Transform, Visibility)]
pub struct Izzy;

impl CharacterAssets for Izzy {
    const POR: &str = "girl.png";
    const SFX: &str = "girl.mp3";
}
