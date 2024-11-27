use super::{
    material::{TextMaterial, TextMaterialMarker},
    DialogueBox, DialogueBoxAtlas, DialogueBoxDimensions, DialogueBoxEvent, DialogueBoxFont,
    SectionOccurance, TypeWriterState,
};
use crate::dialogue::FragmentEndEvent;
use bevy::{input::keyboard::KeyboardInput, prelude::*, sprite::Anchor, text::Text2dBounds};
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
) {
    for event in reader.read() {
        if let Ok(text_box) = boxes.get(event.entity) {
            for child in text_box.iter() {
                match event.event.data.clone() {
                    bevy_bits::DialogueBoxToken::Section(section) => {
                        if let Ok((mut text, mut state, _)) = type_writers.get_mut(*child) {
                            state.push_section(section, event.event.id);
                        }
                    }
                    bevy_bits::DialogueBoxToken::Command(cmd) => {
                        if let Ok((mut text, mut state, _)) = type_writers.get_mut(*child) {
                            state.push_cmd(cmd, event.event.id);
                        }
                    }
                }
            }
        }
    }

    for (mut text, mut state, box_font) in type_writers.iter_mut() {
        // TODO: this will be cheap in the custom pipeline
        if let Some(section) = state.tick(&time) {
            match section {
                SectionOccurance::First(section) => {
                    text.sections.push(bevy::text::TextSection::new(
                        section.text.to_string(),
                        bevy::text::TextStyle {
                            font_size: box_font.font_size,
                            font: box_font.font.clone(),
                            color: section
                                .color
                                .as_ref()
                                .map(|c| c.bevy_color())
                                .unwrap_or_else(|| box_font.default_color),
                            // color: if section.effects.is_empty() {
                            //     section
                            //         .color
                            //         .as_ref()
                            //         .map(|c| c.bevy_color())
                            //         .unwrap_or_else(|| box_font.default_color)
                            // } else {
                            //     bevy::color::Color::NONE
                            // },
                        },
                    ))
                }
                SectionOccurance::Repeated(updated_text) => {
                    text.sections.last_mut().as_mut().unwrap().value =
                        updated_text.as_ref().to_owned();
                }
                SectionOccurance::End => {}
            }
        }

        if state.finished(&mut input, &mut text) {
            if let Some(id) = state.fragment_id() {
                writer.send(id.end());
            }
        }
    }
}

// TODO: handle text materials for effects
// pub fn handle_dialogue_box_events_material<M: TextMaterial>(
//     mut reader: EventReader<DialogueBoxEvent>,
//     mut writer: EventWriter<FragmentEndEvent>,
//     time: Res<Time>,
//     boxes: Query<&Children, With<DialogueBox>>,
//     mut type_writers: Query<(
//         &mut bevy::text::Text,
//         &mut TypeWriterState,
//         &DialogueBoxFont,
//         &TextMaterialMarker<M>,
//     )>,
//     mut input: EventReader<KeyboardInput>,
// ) {
//     for (mut text, mut state, box_font, _) in type_writers.iter_mut() {
//         if let Some(section) = state.tick(&time) {
//             match section {
//                 SectionOccurance::First(section) => {
//                     println!("{section:#?}");
//                     text.sections.push(bevy::text::TextSection::new(
//                         section.text.to_string(),
//                         bevy::text::TextStyle {
//                             font_size: box_font.font_size,
//                             font: box_font.font.clone(),
//                             color: if let Some(effect) = section.effects.last() {
//                                 if M::can_render_effect(effect) {
//                                     section
//                                         .color
//                                         .as_ref()
//                                         .map(|c| c.bevy_color())
//                                         .unwrap_or_else(|| box_font.default_color)
//                                 } else {
//                                     bevy::color::Color::NONE
//                                 }
//                             } else {
//                                 bevy::color::Color::NONE
//                             },
//                         },
//                     ))
//                 }
//                 SectionOccurance::Repeated(updated_text) => {
//                     text.sections.last_mut().as_mut().unwrap().value =
//                         updated_text.as_ref().to_owned();
//                 }
//                 SectionOccurance::End => {}
//             }
//         }
//     }
// }
