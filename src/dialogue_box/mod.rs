#![allow(unused)]
use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
    utils::HashMap,
};
use std::path::Path;

mod box_atlas;
mod text;
mod type_writer;

pub use box_atlas::*;
pub use text::*;
pub use type_writer::*;

/// Attaches to a [`crate::dialogue::DialogueEvent`] and displays it in a dialogue box.
pub struct DialogueBoxPlugin<P> {
    font_path: P,
    box_atlas_path: P,
    box_atlas_tile_size: UVec2,
}

impl<P> DialogueBoxPlugin<P> {
    pub fn new(font_path: P, box_atlas_path: P, box_atlas_tile_size: UVec2) -> Self {
        Self {
            font_path,
            box_atlas_path,
            box_atlas_tile_size,
        }
    }
}

impl<P: AsRef<Path> + Send + Sync + 'static> Plugin for DialogueBoxPlugin<P> {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            text::DialogueBoxTextPlugin::new(&self.font_path),
            box_atlas::BoxPlugin::new(&self.box_atlas_path, self.box_atlas_tile_size),
        ))
        .insert_resource(DialogueBoxRegistry::default())
        .add_systems(Update, bind_text_to_box);

        // app.add_systems(Update, test_dialogue_box_events);
    }
}

#[derive(Resource, Default)]
pub struct DialogueBoxRegistry {
    table: HashMap<DialogueBoxId, DialogueBoxDescriptor>,
}

impl DialogueBoxRegistry {
    pub fn register(&mut self, id: DialogueBoxId, descriptor: DialogueBoxDescriptor) {
        self.table.insert(id, descriptor);
    }

    pub fn remove(&mut self, id: &DialogueBoxId) {
        self.table.remove(id);
    }
}

pub struct DialogueBoxDescriptor {
    pub dimensions: DialogueBoxDimensions,
    pub tile_size: UVec2,
    pub transform: Transform,
}

#[allow(clippy::type_complexity)]
fn bind_text_to_box(
    mut text: Query<(&mut Transform, &DialogueBoxId), With<DialogueText>>,
    registery: Res<DialogueBoxRegistry>,
) {
    // for (id, desc) in registery.table.iter() {
    //     for (mut transform, text_dialogue_id) in text.iter_mut() {
    //         if id == text_dialogue_id {
    //             transform.translation = desc.transform.translation;
    //         }
    //     }
    // }
}
