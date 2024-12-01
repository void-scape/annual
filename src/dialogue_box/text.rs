use super::{BoxToken, DialogueBox, DialogueBoxFont, TypeWriterState};
use crate::dialogue::{FragmentEndEvent, FragmentEvent};
use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
    window::PrimaryWindow,
};

pub fn handle_dialogue_box_events(
    mut reader: EventReader<FragmentEvent<BoxToken>>,
    mut writer: EventWriter<FragmentEndEvent>,
    time: Res<Time>,
    boxes: Query<&Children, With<DialogueBox>>,
    mut type_writers: Query<(
        &mut bevy::text::Text,
        &mut TypeWriterState,
        &DialogueBoxFont,
    )>,
    mut input: EventReader<KeyboardInput>,
    window: Query<Entity, With<PrimaryWindow>>,
    reveal_sfx: Option<Res<super::audio::RevealedTextSfx>>,
    delete_sfx: Option<Res<super::audio::DeletedTextSfx>>,
    mut commands: Commands,
) {
    for event in reader.read() {
        if let Ok(children) = boxes.get(event.data.1.entity()) {
            for child in children.iter() {
                match event.data.0.clone() {
                    bevy_bits::DialogueBoxToken::Section(section) => {
                        if let Ok((_, mut state, _)) = type_writers.get_mut(*child) {
                            state.push_section(section, Some(event.id));
                        }
                    }
                    bevy_bits::DialogueBoxToken::Command(cmd) => {
                        if let Ok((_, mut state, _)) = type_writers.get_mut(*child) {
                            state.push_cmd(cmd, Some(event.id));
                        }
                    }
                    bevy_bits::DialogueBoxToken::Sequence(seq) => {
                        if let Ok((_, mut state, _)) = type_writers.get_mut(*child) {
                            state.push_seq(seq, Some(event.id));
                        }
                    }
                }
            }
        }
    }

    let received_input = input.read().next().is_some_and(|i| {
        (i.state == ButtonState::Pressed && i.window == window.single())
            && (i.key_code == KeyCode::Space || i.key_code == KeyCode::Enter)
    });

    let mut reveal_sfx = reveal_sfx.map(|s| s.into_inner());
    let mut delete_sfx = delete_sfx.map(|s| s.into_inner());

    for (i, (mut text, mut state, box_font)) in type_writers.iter_mut().enumerate() {
        // TODO: this will be cheap in the custom pipeline

        if i != 0 {
            reveal_sfx = None;
            delete_sfx = None;
        }

        if let Some(end) = state.tick(
            &time,
            received_input,
            &mut text,
            box_font,
            &mut commands,
            reveal_sfx,
            delete_sfx,
            false,
        ) {
            // HACK: There is one typewriter per material + 1 for none. All of them update, but we only
            // want to send the end event once.
            if i == 0 {
                writer.send(end);
            }
        }
    }
}
