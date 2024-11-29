// use crate::{FragmentExt, IntoFragment};
// use bevy::prelude::*;
// use rand::Rng;
//
// use super::DialogueBox;
//
// #[derive(Resource, Clone)]
// pub struct DialogueBoxSprite {
//     pub bundle: SpriteBundle,
// }
//
// pub trait InsertSprite {
//     fn sprite(
//         self,
//         dialogue_box: Entity,
//         bundle: SpriteBundle,
//     ) -> impl IntoFragment<bevy_bits::DialogueBoxToken>;
// }
//
// impl<T> InsertSprite for T
// where
//     T: IntoFragment<bevy_bits::DialogueBoxToken>,
// {
//     fn sprite(
//         self,
//         dialogue_box: Entity,
//         bundle: SpriteBundle,
//     ) -> impl IntoFragment<bevy_bits::DialogueBoxToken> {
//         let key = SpriteKey::random();
//
//         self.on_start(move |mut commands: Commands| {
//             commands.entity(dialogue_box).with_children(|b| {
//                 b.spawn((bundle.clone(), key));
//             });
//         })
//         .on_end(
//             move |mut commands: Commands,
//                   boxes: Query<&Children, With<DialogueBox>>,
//                   sprites: Query<&SpriteKey, With<Sprite>>| {
//                 if let Ok(b) = boxes.get(dialogue_box) {
//                     for child in b.iter() {
//                         if let Ok(child_key) = sprites.get(*child) {
//                             if *child_key == key {
//                                 commands.entity(*child).despawn();
//                             }
//                         }
//                     }
//                 } else {
//                     error!("failed to despawn DialogueBoxSprite after end of fragment scope");
//                 }
//             },
//         )
//     }
// }
//
// #[derive(Component, Clone, Copy, PartialEq, Eq)]
// struct SpriteKey(usize);
//
// impl SpriteKey {
//     pub fn random() -> Self {
//         Self(rand::thread_rng().gen())
//     }
// }
