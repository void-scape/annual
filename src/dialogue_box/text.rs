use crate::dialogue::*;
use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};
use bevy_bits::type_writer::TypeWriter;
use std::path::Path;

pub struct DialogueBoxTextPlugin {
    font_path: String,
}

impl DialogueBoxTextPlugin {
    pub fn new<P: AsRef<Path>>(path: &P) -> Self {
        Self {
            font_path: String::from(path.as_ref().to_str().expect("invalid font path")),
        }
    }
}

impl Plugin for DialogueBoxTextPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DialogueBoxFontPath(self.font_path.clone()))
            .add_systems(Startup, setup_font)
            .add_systems(Update, (start_type_writers, update_type_writers));
    }
}

#[derive(Resource)]
struct DialogueBoxFontPath(String);

#[derive(Resource)]
struct DialogueBoxFont(Handle<Font>);

fn setup_font(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    path: Res<DialogueBoxFontPath>,
) {
    let font = DialogueBoxFont(asset_server.load(&path.0));
    commands.insert_resource(font);
}

fn start_type_writers(
    mut commands: Commands,
    font: Res<DialogueBoxFont>,
    mut reader: EventReader<DialogueEvent>,
) {
    for event in reader.read() {
        info!("received dialogue event: {event:?}");

        commands.spawn((
            TextBundle::from_section(
                "",
                TextStyle {
                    font: font.0.clone(),
                    font_size: 32.0,
                    color: Color::WHITE,
                },
            ),
            TypeWriter::new_start(event.dialogue.clone(), 10.0),
            event.id,
        ));
    }
}

#[derive(Component)]
struct AwaitingInput;

fn update_type_writers(
    mut commands: Commands,
    time: Res<Time>,
    mut type_writers: Query<
        (Entity, &mut TypeWriter, &mut Text, &DialogueId),
        Without<AwaitingInput>,
    >,
    finished_type_writers: Query<(Entity, &DialogueId), With<AwaitingInput>>,
    mut writer: EventWriter<DialogueEndEvent>,
    mut reader: EventReader<KeyboardInput>,
) {
    let mut input_received = false;
    for event in reader.read() {
        if event.state == ButtonState::Pressed {
            input_received = true;
        }
    }

    for (entity, mut type_writer, mut text, id) in type_writers.iter_mut() {
        if input_received {
            type_writer.reveal_all_text();
        } else {
            type_writer
                .tick(&time, |type_writer| {
                    text.sections[0].value = type_writer.revealed_text_with_line_wrap();
                })
                .on_finish(|| {
                    info!("finished dialogue event: {id:?}");
                    info!("awaiting user input...");
                    commands.entity(entity).insert(AwaitingInput);
                });
        }
    }

    if let Ok((entity, id)) = finished_type_writers.get_single() {
        if input_received {
            info!("ending dialogue event: {id:?}");
            writer.send(id.end());
            commands.entity(entity).despawn();
        }
    }
}
