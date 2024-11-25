#![allow(unused)]
use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};
use std::path::Path;

pub mod box_atlas;
pub mod text;

pub use box_atlas::*;
pub use text::*;

/// Attaches to a [`crate::dialogue::DialogueEvent`] and displays it in a dialogue box.
pub struct DialogueBoxPlugin<P> {
    font_path: P,
}

impl<P> DialogueBoxPlugin<P> {
    pub fn new(font_path: P) -> Self {
        Self { font_path }
    }
}

impl<P: AsRef<Path> + Send + Sync + 'static> Plugin for DialogueBoxPlugin<P> {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            text::DialogueBoxTextPlugin::new(&self.font_path),
            box_atlas::BoxPlugin,
        ));

        // app.add_systems(Update, test_dialogue_box_events);
    }
}

fn test_dialogue_box_events(
    mut show: EventWriter<ShowDialogueBox>,
    mut hide: EventWriter<HideDialogueBox>,
    mut reader: EventReader<KeyboardInput>,
) {
    for event in reader.read() {
        let id = DialogueBoxId(0);

        if event.state == ButtonState::Pressed {
            match event.key_code {
                KeyCode::KeyS => {
                    show.send(ShowDialogueBox {
                        id,
                        transform: Transform::default().with_scale(Vec3::new(2.0, 2.0, 1.0)),
                        inner_width: 2,
                        inner_height: 1,
                    });
                }
                KeyCode::KeyH => {
                    hide.send(HideDialogueBox { id });
                }
                _ => {}
            }
        }
    }
}
