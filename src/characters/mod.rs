use bevy::asset::AssetPath;
use portrait::*;
use sfx::SfxDescriptor;

pub mod portrait;
pub mod sfx;

// pub trait CharacterAssets {
//     fn texture() -> impl Into<AssetPath<'static>>;
//     fn text_sfx() -> SfxDescriptor;
// }
//
// #[derive(macros::Character)]
// pub struct Flower;
//
// impl CharacterAssets for Flower {
//     fn texture() -> impl Into<AssetPath<'static>> {
//         "flower.png"
//     }
//
//     fn text_sfx() -> SfxDescriptor {
//         "flowey.mp3".into()
//     }
// }
//
// #[allow(unused_macros)]
// macro_rules! character_stub {
//     ($name:ident, $texture:expr, $sfx:expr) => {
//         #[derive(macros::Character, bevy::prelude::Component, Default)]
//         pub struct $name;
//
//         impl CharacterAssets for $name {
//             fn texture() -> impl Into<AssetPath<'static>> {
//                 $texture
//             }
//
//             fn text_sfx() -> SfxDescriptor {
//                 $sfx
//             }
//         }
//     };
// }
//
// character_stub!(Izzy, "girl.png", "girl.mp3".into());
