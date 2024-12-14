use super::{
    audio::{DeletedTextSfx, RevealedTextSfx},
    BoxToken, DialogueBox, Font, TypeWriterState,
};
use crate::{
    dialogue::{FragmentEndEvent, FragmentEvent},
    player::Action,
};
use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
    window::PrimaryWindow,
};
use leafwing_input_manager::prelude::ActionState;

pub fn handle_dialogue_box_events(
    mut reader: EventReader<FragmentEvent<BoxToken>>,
    mut writer: EventWriter<FragmentEndEvent>,
    time: Res<Time>,
    boxes: Query<(&Children, &RevealedTextSfx, &DeletedTextSfx), With<DialogueBox>>,
    mut type_writers: Query<(Entity, &mut bevy::text::Text, &mut TypeWriterState, &Font)>,
    input: Query<&ActionState<Action>>,
    window: Query<Entity, With<PrimaryWindow>>,
    mut commands: Commands,
) {
    for event in reader.read() {
        if let Ok((children, _, _)) = boxes.get(event.data.1.entity()) {
            for child in children.iter() {
                match event.data.0.clone() {
                    bevy_bits::DialogueBoxToken::Section(section) => {
                        if let Ok((_, _, mut state, _)) = type_writers.get_mut(*child) {
                            state.push_section(section, Some(event.id));
                        }
                    }
                    bevy_bits::DialogueBoxToken::Command(cmd) => {
                        if let Ok((_, _, mut state, _)) = type_writers.get_mut(*child) {
                            state.push_cmd(cmd, Some(event.id));
                        }
                    }
                    bevy_bits::DialogueBoxToken::Sequence(seq) => {
                        if let Ok((_, _, mut state, _)) = type_writers.get_mut(*child) {
                            state.push_seq(seq, Some(event.id));
                        }
                    }
                }
            }
        }
    }

    let received_input = if let Ok(a) = input.get_single() {
        a.just_pressed(&Action::Interact)
    } else {
        false
    };

    for (i, (entity, mut text, mut state, box_font)) in type_writers.iter_mut().enumerate() {
        // TODO: this will be cheap in the custom pipeline

        for (_, reveal, delete) in boxes.iter() {}

        let reveal_sfx = boxes
            .iter()
            .find_map(|(c, r, _)| (i == 0 && c.contains(&entity)).then_some(r));
        let delete_sfx = boxes
            .iter()
            .find_map(|(c, _, d)| (i == 0 && c.contains(&entity)).then_some(d));

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
