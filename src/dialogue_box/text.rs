use super::{
    material::{TextMaterial, TextMaterialMarker},
    DialogueBox, DialogueBoxAtlas, DialogueBoxDimensions, DialogueBoxEvent, DialogueBoxFont,
    SectionOccurance, TypeWriterState,
};
use crate::dialogue::FragmentEndEvent;
use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
    sprite::Anchor,
    text::Text2dBounds,
};
use rand::Rng;
use std::marker::PhantomData;

pub fn handle_dialogue_box_events(
    mut reader: EventReader<DialogueBoxEvent>,
    mut writer: EventWriter<FragmentEndEvent>,
    time: Res<Time>,
    boxes: Query<&Children, With<DialogueBox>>,
    mut type_writers: Query<(
        &mut bevy::text::Text,
        &mut TypeWriterState,
        &DialogueBoxFont,
    )>,
    mut input: EventReader<KeyboardInput>,
    reveal_sfx: Option<Res<super::audio::RevealedTextSfx>>,
    delete_sfx: Option<Res<super::audio::DeletedTextSfx>>,
    mut commands: Commands,
) {
    for event in reader.read() {
        if let Ok(text_box) = boxes.get(event.entity) {
            for child in text_box.iter() {
                match event.event.data.clone() {
                    bevy_bits::DialogueBoxToken::Section(section) => {
                        if let Ok((mut text, mut state, box_font)) = type_writers.get_mut(*child) {
                            state.push_section(section, Some(event.event.id), box_font);
                        }
                    }
                    bevy_bits::DialogueBoxToken::Command(cmd) => {
                        if let Ok((mut text, mut state, _)) = type_writers.get_mut(*child) {
                            state.push_cmd(cmd, Some(event.event.id));
                        }
                    }
                    bevy_bits::DialogueBoxToken::Sequence(seq) => {
                        if let Ok((mut text, mut state, box_font)) = type_writers.get_mut(*child) {
                            state.push_seq(seq, Some(event.event.id), box_font);
                        }
                    }
                }
            }
        }
    }

    let received_input = input
        .read()
        .next()
        .is_some_and(|i| i.state == ButtonState::Pressed);

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
