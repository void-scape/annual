use super::material::WaveMaterial;
use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
    render::{
        render_resource::{
            AsBindGroup, Extent3d, ShaderRef, TextureDescriptor, TextureDimension, TextureFormat,
            TextureUsages,
        },
        view::RenderLayers,
    },
    sprite::{Anchor, Material2d, Material2dPlugin, MaterialMesh2dBundle},
    text::Text2dBounds,
    window::{PrimaryWindow, WindowResized},
};
use std::path::Path;

#[derive(Debug, Clone)]
pub enum TextCommand {
    Speed(f32),
    Pause(f32),
}

#[derive(Debug, Clone)]
pub struct TextSection {
    pub text: RawText,
    pub color: Option<TextColor>,
    pub effects: Vec<TextEffect>,
}

impl From<&'static str> for TextSection {
    fn from(value: &'static str) -> Self {
        TextSection {
            text: RawText::Str(value),
            color: None,
            effects: Vec::new(),
        }
    }
}

impl From<String> for TextSection {
    fn from(value: String) -> Self {
        TextSection {
            text: RawText::String(value),
            color: None,
            effects: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum RawText {
    Str(&'static str),
    String(String),
}

#[derive(Debug, Clone)]
pub enum TextEffect {
    Wave,
}

#[derive(Debug, Clone)]
pub enum TextColor {
    Red,
    Green,
    Blue,
}

// TODO: this method can lead to duplication of shader text entities if mutliple sections use the
// same shader effect
// fn start_type_writers(
//     mut commands: Commands,
//     font: Res<DialogueBoxFont>,
//     mut reader: EventReader<DialogueBoxEvent>,
//     mut writer: EventWriter<FragmentEndEvent>,
//     registry: Res<DialogueBoxRegistry>,
// ) {
//     for DialogueBoxEvent(event, box_id) in reader.read() {
//         info!("received dialogue event: {event:?}, attached to {box_id:?}");
//
//         let Some(box_desc) = registry.table.get(box_id) else {
//             writer.send(event.id.end());
//             error!("could not find dialogue box {box_id:?} for event {event:?}");
//             return;
//         };
//
//         commands.spawn((
//             (),
//             // TypeWriter::new_start(event.data.clone(), 20.0),
//             DialogueText {
//                 text: None,
//                 text_effects: Vec::new(),
//                 default_bundle: Text2dBundle {
//                     text: Text::default(),
//                     text_anchor: Anchor::TopLeft,
//                     text_2d_bounds: Text2dBounds {
//                         size: Vec2::new(
//                             (box_desc.dimensions.inner_width as f32 + 1.0)
//                                 * box_desc.tile_size.x as f32
//                                 * box_desc.transform.scale.x,
//                             (box_desc.dimensions.inner_height as f32 + 1.0)
//                                 * box_desc.tile_size.y as f32
//                                 * box_desc.transform.scale.x,
//                         ),
//                     },
//                     transform: Transform::default()
//                         .with_translation(box_desc.transform.translation),
//                     ..Default::default()
//                 },
//             },
//             TransformBundle::default(),
//             Visibility::default(),
//             InheritedVisibility::default(),
//             ViewVisibility::default(),
//             event.id,
//             *box_id,
//         ));
//     }
// }

#[derive(Component)]
struct AwaitingInput;

// impl DialogueText {
//     pub fn display_dialogue(
//         &mut self,
//         commands: &mut Commands,
//         sections: &[&DialogueTextSection],
//         dialogue_text: &mut Query<&mut Text>,
//         parent: Entity,
//     ) {
//         for (i, section) in sections.iter().enumerate() {
//             if let Some(effect) = section
//                 .effect
//                 .and_then(|e| e.requires_shader().then_some(e))
//             {
//                 let sections = sections
//                     .iter()
//                     .map(|s| {
//                         if Some(effect) == s.effect {
//                             s.section.clone()
//                         } else {
//                             let mut section = s.section.clone();
//                             section.style.color = Color::NONE;
//                             section
//                         }
//                     })
//                     .collect();
//
//                 if let Some(entity) = self
//                     .text_effects
//                     .iter()
//                     .find_map(|(e, ef)| (*ef == effect).then_some(e))
//                 {
//                     if let Ok(mut text) = dialogue_text.get_mut(*entity) {
//                         text.sections = sections;
//                     } else {
//                         error!("dialogue effect entity is invalid");
//                     }
//                 } else {
//                     let mut default_bundle = self.default_bundle.clone();
//                     default_bundle.text.sections = sections;
//                     commands.entity(parent).with_children(|parent| {
//                         let entity = parent.spawn((default_bundle, effect.render_layer())).id();
//                         self.text_effects.push((entity, effect));
//                     });
//                 }
//             } else if sections
//                 .iter()
//                 .any(|s| !s.effect.is_some_and(|e| e.requires_shader()))
//             {
//                 let sections = sections
//                     .iter()
//                     .map(|s| {
//                         if s.effect.is_none_or(|e| !e.requires_shader()) {
//                             s.section.clone()
//                         } else {
//                             let mut section = s.section.clone();
//                             section.style.color = Color::NONE;
//                             section
//                         }
//                     })
//                     .collect();
//
//                 if let Some(entity) = self.text {
//                     if let Ok(mut text) = dialogue_text.get_mut(entity) {
//                         text.sections = sections;
//                     } else {
//                         error!("dialogue normal text entity is invalid");
//                     }
//                 } else {
//                     let mut default_bundle = self.default_bundle.clone();
//                     default_bundle.text.sections = sections;
//                     commands.entity(parent).with_children(|parent| {
//                         let entity = parent.spawn(default_bundle).id();
//                         self.text = Some(entity);
//                     });
//                 }
//             }
//         }
//     }
// }
//
// #[allow(clippy::type_complexity)]
// fn update_type_writers(
//     mut commands: Commands,
//     time: Res<Time>,
//     mut type_writers: Query<
//         (Entity, &mut TypeWriter, &FragmentId, &mut DialogueText),
//         Without<AwaitingInput>,
//     >,
//     mut text: Query<&mut Text>,
//     finished_type_writers: Query<(Entity, &FragmentId), With<AwaitingInput>>,
//     mut writer: EventWriter<FragmentEndEvent>,
//     mut reader: EventReader<KeyboardInput>,
// ) {
//     let mut input_received = false;
//     for event in reader.read() {
//         if event.state == ButtonState::Pressed {
//             input_received = true;
//         }
//     }
//
//     for (entity, mut type_writer, id, mut dialogue_text) in type_writers.iter_mut() {
//         if input_received {
//             type_writer.reveal_all_text();
//         } else {
//             type_writer
//                 .tick(&time, |type_writer| {
//                     let mut tokens = type_writer.revealed_text_with_line_wrap();
//
//                     if let Some(TypeWriterToken::Command(command)) = tokens.last() {
//                         match *command {
//                             TypeWriterEffect::Pause(duration) => {
//                                 type_writer.pause_for(duration);
//                                 return;
//                             }
//                             _ => {}
//                         }
//                     }
//
//                     for token in tokens.iter() {
//                         if let TypeWriterToken::Command(command) = token {
//                             match *command {
//                                 TypeWriterEffect::Speed(speed) => {
//                                     type_writer.with_speed(speed);
//                                 }
//                                 _ => {}
//                             }
//                         }
//                     }
//
//                     let tokens = tokens
//                         .iter()
//                         .filter_map(|t| match t {
//                             TypeWriterToken::Dialogue(d) => Some(d),
//                             _ => None,
//                         })
//                         .collect::<Vec<_>>();
//                     dialogue_text.display_dialogue(&mut commands, &tokens, &mut text, entity);
//                 })
//                 .on_finish(|| {
//                     commands.entity(entity).insert(AwaitingInput);
//                 });
//         }
//     }
//
//     if let Ok((entity, id)) = finished_type_writers.get_single() {
//         if input_received {
//             info!("ending dialogue event: {id:?}");
//             writer.send(id.end());
//             commands.entity(entity).despawn_recursive();
//         }
//     }
// }
