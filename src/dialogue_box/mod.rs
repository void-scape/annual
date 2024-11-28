#![allow(unused)]
use crate::{
    dialogue::{FragmentEndEvent, FragmentEvent, FragmentId},
    Fragment, FragmentExt, IntoFragment, SpawnFragment,
};
use bevy::{
    asset::AssetPath,
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
    render::view::RenderLayers,
    sprite::{Anchor, Material2dPlugin, SpriteSource},
    text::{Text2dBounds, TextLayoutInfo},
    utils::HashMap,
};
use bevy_bits::{DialogueBoxToken, TextCommand};
use material::{TextMaterialMarker, TextMaterialMarkerNone, WaveMaterial, WAVE_MATERIAL_LAYER};
use rand::Rng;
use std::{borrow::Cow, collections::VecDeque, path::Path, time::Duration};

mod material;
mod text;

pub use text::*;

/// Attaches to a [`crate::dialogue::FragmentEvent<DialogueBoxToken>`] and displays it in a dialogue box.
pub struct DialogueBoxPlugin;

impl Plugin for DialogueBoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<WaveMaterial>::default())
            .add_systems(
                Startup,
                material::init_effect_material::<
                    WaveMaterial,
                    // TODO: wtf
                    // material::WAVE_MATERIAL_LAYER,
                    1,
                >,
            )
            .add_systems(
                Update,
                (
                    text::handle_dialogue_box_events,
                    material::remove_effects_from_type_writer,
                    material::update_effect_type_writer::<WaveMaterial>,
                    material::resize_text_effect_textures,
                ),
            );
    }
}

/// [`AudioSourceBundle<AudioSource>`] that globally defines `revealed` text sfx for all dialogue
/// boxes.
#[derive(Resource, Clone)]
pub struct RevealedTextSfx {
    pub bundle: bevy::audio::AudioBundle,
    pub settings: TextSfxSettings,
}

impl RevealedTextSfx {
    pub fn bundle(&self) -> bevy::audio::AudioBundle {
        let mut bundle = self.bundle.clone();
        bundle.settings.speed = self.settings.pitch
            + if self.settings.pitch_variance != 0.0 {
                rand::thread_rng()
                    .gen_range(-self.settings.pitch_variance..self.settings.pitch_variance)
            } else {
                0.0
            };

        bundle
    }
}

/// [`AudioSourceBundle<AudioSource>`] that globally defines `deleted` text sfx for all dialogue
/// boxes.
#[derive(Resource, Clone)]
pub struct DeletedTextSfx {
    pub bundle: bevy::audio::AudioBundle,
    pub settings: TextSfxSettings,
}

impl DeletedTextSfx {
    pub fn bundle(&self) -> bevy::audio::AudioBundle {
        let mut bundle = self.bundle.clone();
        bundle.settings.speed = self.settings.pitch
            + if self.settings.pitch_variance != 0.0 {
                rand::thread_rng()
                    .gen_range(-self.settings.pitch_variance..self.settings.pitch_variance)
            } else {
                0.0
            };

        bundle
    }
}

#[derive(Clone)]
pub struct TextSfxSettings {
    pub pitch: f32,
    pub pitch_variance: f32,
    /// Audio samples per second
    pub rate: f32,
}

pub trait SetDialogueTextSfx {
    fn reveal_sfx(
        self,
        bundle: AudioBundle,
        settings: TextSfxSettings,
    ) -> impl IntoFragment<bevy_bits::DialogueBoxToken>;

    fn delete_sfx(
        self,
        bundle: AudioBundle,
        settings: TextSfxSettings,
    ) -> impl IntoFragment<bevy_bits::DialogueBoxToken>;
}

impl<T> SetDialogueTextSfx for T
where
    T: IntoFragment<bevy_bits::DialogueBoxToken>,
{
    fn reveal_sfx(
        self,
        bundle: AudioBundle,
        settings: TextSfxSettings,
    ) -> impl IntoFragment<bevy_bits::DialogueBoxToken> {
        self.set_resource(RevealedTextSfx { bundle, settings })
    }

    fn delete_sfx(
        self,
        bundle: AudioBundle,
        settings: TextSfxSettings,
    ) -> impl IntoFragment<bevy_bits::DialogueBoxToken> {
        self.set_resource(DeletedTextSfx { bundle, settings })
    }
}

/// Associates a [`FragmentEvent<DialogueBoxToken>`] with a specific [`DialogueBoxBundle`] entity.
#[derive(Event, Debug, Clone)]
pub struct DialogueBoxEvent {
    pub entity: Entity,
    pub event: FragmentEvent<bevy_bits::DialogueBoxToken>,
}

/// Spawns a dialogue box with a texture atlas, font, and position.
#[derive(Bundle, Default, Clone)]
pub struct DialogueBoxBundle {
    pub atlas: DialogueBoxAtlas,
    pub dimensions: DialogueBoxDimensions,
    pub spatial: SpatialBundle,
    pub dialogue_box: DialogueBox,
}

impl DialogueBoxBundle {
    pub fn new(
        atlas: DialogueBoxAtlas,
        dimensions: DialogueBoxDimensions,
        spatial: SpatialBundle,
    ) -> Self {
        Self {
            dialogue_box: DialogueBox,
            atlas,
            dimensions,
            spatial,
        }
    }

    pub fn text_bounds(&self) -> Text2dBounds {
        Text2dBounds {
            size: Vec2::new(
                (self.atlas.tile_size.x * self.dimensions.inner_width as u32 + 1) as f32
                    * self.spatial.transform.scale.x,
                (self.atlas.tile_size.y * self.dimensions.inner_height as u32 + 1) as f32
                    * self.spatial.transform.scale.y,
            ),
        }
    }
}

#[derive(Component, Default, Clone)]
pub struct DialogueBox;

#[derive(Component, Default, Clone)]
pub struct DialogueBoxFont {
    pub font: Handle<Font>,
    pub font_size: f32,
    pub default_color: bevy::color::Color,
}

#[derive(Component, Default, Clone)]
pub struct DialogueBoxAtlas {
    pub atlas_layout: Handle<TextureAtlasLayout>,
    pub tile_size: UVec2,
    pub texture: Handle<Image>,
}

impl DialogueBoxAtlas {
    pub fn new<'a>(
        asset_server: &AssetServer,
        texture_atlases: &mut Assets<TextureAtlasLayout>,
        atlas_path: impl Into<AssetPath<'a>>,
        tile_size: UVec2,
    ) -> Self {
        let texture_atlas = TextureAtlasLayout::from_grid(tile_size, 3, 3, None, None);
        let atlas_layout = texture_atlases.add(texture_atlas);

        Self {
            texture: asset_server.load(atlas_path),
            atlas_layout,
            tile_size,
        }
    }
}

#[derive(Component, Default, Clone, Copy)]
pub struct DialogueBoxDimensions {
    pub inner_width: usize,
    pub inner_height: usize,
}

impl DialogueBoxDimensions {
    pub fn new(inner_width: usize, inner_height: usize) -> Self {
        Self {
            inner_width,
            inner_height,
        }
    }
}

#[derive(Bundle, Default, Clone)]
pub struct TypeWriterBundle {
    pub font: DialogueBoxFont,
    pub state: TypeWriterState,
    pub spatial: SpatialBundle,
    pub text: Text,
    pub text_anchor: Anchor,
    pub text_2d_bounds: Text2dBounds,
    pub text_layout_info: TextLayoutInfo,
    pub sprite_source: SpriteSource,
}

#[derive(Component, Debug, Clone)]
pub struct TypeWriterState {
    chars_per_sec: f32,
    state: State,
    force_update: bool,
    effect_mapping: Vec<Option<bevy_bits::TextEffect>>,
    fragment_id: Option<FragmentId>,
    reveal_accum: f32,
    delete_accum: f32,
}

impl Default for TypeWriterState {
    fn default() -> Self {
        Self {
            chars_per_sec: 20.0,
            state: State::Ready,
            force_update: false,
            effect_mapping: Vec::new(),
            fragment_id: None,
            reveal_accum: 0.0,
            delete_accum: 0.0,
        }
    }
}

impl TypeWriterState {
    pub fn new(chars_per_sec: f32) -> Self {
        Self {
            chars_per_sec,
            ..Default::default()
        }
    }

    pub fn push_section(
        &mut self,
        section: bevy_bits::tokens::TextSection,
        id: Option<FragmentId>,
        font: &DialogueBoxFont,
    ) {
        debug_assert!(!matches!(self.state, State::Sequence { .. }));

        self.state = State::Section {
            section: TypeWriterSectionBuffer::new(section, font),
            timer: Timer::new(
                Duration::from_secs_f32(1.0 / self.chars_per_sec),
                TimerMode::Repeating,
            ),
        };
        self.fragment_id = id;
    }

    pub fn push_cmd(&mut self, command: TextCommand, id: Option<FragmentId>) {
        debug_assert!(!matches!(self.state, State::Sequence { .. }));

        self.state = State::Command(command);
        self.fragment_id = id;
    }

    pub fn push_seq(
        &mut self,
        sequence: Cow<'static, [DialogueBoxToken]>,
        id: Option<FragmentId>,
        font: &DialogueBoxFont,
    ) {
        debug_assert!(sequence.len() > 0);

        let mut type_writer = TypeWriterState::new(self.chars_per_sec);
        match sequence[0].clone() {
            DialogueBoxToken::Section(sec) => {
                type_writer.push_section(sec, Some(FragmentId::random()), font)
            }
            DialogueBoxToken::Command(cmd) => type_writer.push_cmd(cmd, Some(FragmentId::random())),
            DialogueBoxToken::Sequence(seq) => {
                type_writer.push_seq(seq, Some(FragmentId::random()), font)
            }
        }

        self.state = State::Sequence {
            sequence,
            type_writer: Box::new(type_writer),
            index: 1,
            force_update: false,
        };
        self.fragment_id = id;
    }

    pub fn tick(
        &mut self,
        time: &Time,
        mut received_input: bool,
        text: &mut Text,
        box_font: &DialogueBoxFont,
        commands: &mut Commands,
        reveal: Option<&RevealedTextSfx>,
        delete: Option<&DeletedTextSfx>,
        force_update: bool,
    ) -> Option<FragmentEndEvent> {
        let mut end_event = None;
        let new_state = match &mut self.state {
            State::Ready => None,
            State::Delete { amount, timer } => {
                timer.tick(time.delta());
                if timer.finished() {
                    if let Some(delete) = delete {
                        // self.delete_accum += time.delta_seconds();
                        // if text
                        //     .sections
                        //     .last()
                        //     .is_some_and(|s| s.value.chars().last().is_some_and(|c| c != ' '))
                        //     && self.delete_accum >= delete.settings.rate
                        // {
                        if !force_update {
                            commands.spawn(delete.bundle());
                        }
                        //     self.delete_accum -= delete.settings.rate;
                        // }
                    }

                    if let Some(section) = text.sections.last_mut() {
                        section.value.pop();
                        *amount -= 1;
                    } else {
                        warn!("tried to delete from section that does not exist");
                    }
                }

                if *amount == 0 {
                    end_event = self.fragment_id;
                    Some(State::Ready)
                } else {
                    None
                }
            }
            State::Section { section, timer } => {
                timer.tick(time.delta());
                if let Some(occurance) = timer.finished().then(|| section.advance()) {
                    Self::update_text(text, &mut self.effect_mapping, box_font, occurance);

                    if let Some(reveal) = reveal {
                        // self.reveal_accum += time.delta_seconds();
                        // println!("{}", self.reveal_accum);
                        // if section
                        //     .current_section()
                        //     .text
                        //     .chars()
                        //     .last()
                        //     .is_some_and(|c| c != ' ')
                        //     && self.reveal_accum >= reveal.settings.rate
                        // {
                        if !force_update {
                            commands.spawn(reveal.bundle());
                        }
                        //     self.reveal_accum -= reveal.settings.rate;
                        // }
                    }
                }

                section.finished().then(|| {
                    end_event = self.fragment_id;
                    State::Ready
                })
            }
            State::Command(command) => match command {
                TextCommand::Speed(speed) => {
                    self.chars_per_sec = *speed;
                    end_event = self.fragment_id;
                    Some(State::Ready)
                }
                TextCommand::Pause(duration) => Some(State::Paused(Timer::new(
                    Duration::from_secs_f32(*duration),
                    TimerMode::Once,
                ))),
                TextCommand::Clear => {
                    self.clear(text);
                    end_event = self.fragment_id;
                    Some(State::Ready)
                }
                TextCommand::AwaitClear => Some(State::AwaitClear),
                TextCommand::ClearAfter(duration) => Some(State::ClearAfter(Timer::new(
                    Duration::from_secs_f32(*duration),
                    TimerMode::Once,
                ))),
                TextCommand::Delete(amount) => Some(State::Delete {
                    amount: *amount,
                    timer: Timer::new(
                        Duration::from_secs_f32(1.0 / self.chars_per_sec),
                        TimerMode::Repeating,
                    ),
                }),
            },
            State::ClearAfter(timer) => {
                timer.tick(time.delta());
                timer.finished().then(|| {
                    self.clear(text);
                    end_event = self.fragment_id;
                    State::Ready
                })
            }
            State::AwaitClear => received_input.then(|| {
                self.clear(text);
                end_event = self.fragment_id;
                State::Ready
            }),
            State::Paused(duration) => {
                duration.tick(time.delta());
                duration.finished().then(|| {
                    end_event = self.fragment_id;
                    State::Ready
                })
            }
            State::Sequence {
                sequence,
                type_writer,
                index,
                force_update,
            } => {
                let finished = type_writer.finished() && *index >= sequence.len();

                let mut must_render = false;
                if received_input && !*force_update && !finished {
                    *force_update = true;

                    loop {
                        Self::update_seq_type_writer(
                            type_writer,
                            time,
                            false,
                            text,
                            box_font,
                            index,
                            sequence,
                            commands,
                            reveal,
                            delete,
                            true,
                        );

                        if type_writer.finished() && *index >= sequence.len() {
                            must_render = true;
                            break;
                        }
                    }
                }

                if !must_render
                    && *index >= sequence.len()
                    && matches!(type_writer.state, State::Ready)
                {
                    received_input.then(|| {
                        self.clear(text);
                        end_event = self.fragment_id;
                        State::Ready
                    })
                } else {
                    Self::update_seq_type_writer(
                        type_writer,
                        time,
                        false,
                        text,
                        box_font,
                        index,
                        sequence,
                        commands,
                        reveal,
                        delete,
                        false,
                    );
                    None
                }
            }
        };

        if let Some(new_state) = new_state {
            self.state = new_state;
        }

        end_event.map(|id| id.end())
    }

    pub fn update_reveal_sfx(
        &mut self,
        time: &Time,
        reveal: AudioBundle,
        rate: f32,
        commands: &mut Commands,
    ) {
        match &mut self.state {
            State::Section { section, .. } => {
                if section
                    .current_section()
                    .text
                    .chars()
                    .last()
                    .is_some_and(|c| c != ' ')
                {
                    self.reveal_accum -= time.delta_seconds();
                    if self.reveal_accum <= 0.0 {
                        commands.spawn(reveal);
                        self.reveal_accum = rate;
                    }
                }
            }
            State::Sequence { type_writer, .. } => {
                type_writer.update_reveal_sfx(time, reveal, rate, commands);
            }
            _ => {}
        }
    }

    pub fn update_delete_sfx(
        &mut self,
        time: &Time,
        text: &Text,
        delete: AudioBundle,
        rate: f32,
        commands: &mut Commands,
    ) {
        match &mut self.state {
            State::Delete { .. } => {
                if text
                    .sections
                    .last()
                    .is_some_and(|s| s.value.chars().last().is_some_and(|c| c != ' '))
                {
                    self.delete_accum -= time.delta_seconds();
                    if self.delete_accum <= 0.0 {
                        commands.spawn(delete);
                        self.delete_accum = rate;
                    }
                }
            }
            State::Sequence { type_writer, .. } => {
                type_writer.update_delete_sfx(time, text, delete, rate, commands);
            }
            _ => {}
        }
    }

    pub fn effect_mapping(&self) -> Vec<Option<bevy_bits::TextEffect>> {
        match &self.state {
            State::Sequence { type_writer, .. } => {
                let mut effects = type_writer.effect_mapping();
                effects.extend(self.effect_mapping.clone());
                effects
            }
            _ => self.effect_mapping.clone(),
        }
    }

    fn update_seq_type_writer(
        type_writer: &mut TypeWriterState,
        time: &Time,
        received_input: bool,
        text: &mut Text,
        box_font: &DialogueBoxFont,
        index: &mut usize,
        sequence: &mut Cow<'static, [DialogueBoxToken]>,
        commands: &mut Commands,
        reveal: Option<&RevealedTextSfx>,
        delete: Option<&DeletedTextSfx>,
        force_update: bool,
    ) {
        if type_writer
            .tick(
                time,
                received_input,
                text,
                box_font,
                commands,
                reveal,
                delete,
                force_update,
            )
            .is_some()
            && *index < sequence.len()
        {
            match sequence[*index].clone() {
                DialogueBoxToken::Section(sec) => {
                    type_writer.push_section(sec, Some(FragmentId::random()), box_font)
                }
                DialogueBoxToken::Command(cmd) => {
                    type_writer.push_cmd(cmd, Some(FragmentId::random()))
                }
                DialogueBoxToken::Sequence(seq) => {
                    type_writer.push_seq(seq, Some(FragmentId::random()), box_font)
                }
            }

            *index += 1;
        }
    }

    fn update_text(
        text: &mut Text,
        effect_mapping: &mut Vec<Option<bevy_bits::TextEffect>>,
        box_font: &DialogueBoxFont,
        section: SectionOccurance,
    ) {
        match section {
            SectionOccurance::First(section) => {
                effect_mapping.push(section.effects.first().cloned());
                let mut section = section.clone().bevy_section(
                    box_font.font.clone(),
                    box_font.font_size,
                    box_font.default_color,
                );
                section.style.color.set_alpha(0.0);
                text.sections.push(section);
            }
            SectionOccurance::Repeated(section) => {
                text.sections.last_mut().as_mut().unwrap().value = section.text.into();
            }
            SectionOccurance::End(section) => {
                text.sections.last_mut().as_mut().unwrap().value = section.text.into();
            }
        }
    }

    fn clear(&mut self, text: &mut Text) {
        text.sections.clear();
        self.effect_mapping.clear();
    }

    fn finished(&self) -> bool {
        matches!(self.state, State::Ready)
    }
}

#[derive(Debug, Clone)]
enum State {
    Ready,
    Command(TextCommand),
    Section {
        section: TypeWriterSectionBuffer,
        timer: Timer,
    },
    Paused(Timer),
    AwaitClear,
    ClearAfter(Timer),
    Delete {
        amount: usize,
        timer: Timer,
    },
    Sequence {
        sequence: Cow<'static, [bevy_bits::DialogueBoxToken]>,
        index: usize,
        type_writer: Box<TypeWriterState>,
        force_update: bool,
    },
}

#[derive(Component, Debug, Clone)]
struct TypeWriterSectionBuffer {
    state: SectionBufferState,
}

pub enum SectionOccurance {
    First(bevy_bits::tokens::TextSection),
    Repeated(bevy_bits::tokens::TextSection),
    End(bevy_bits::tokens::TextSection),
}

#[derive(Debug, Clone)]
enum SectionBufferState {
    First {
        section: bevy_bits::tokens::TextSection,
    },
    Repeated {
        section: bevy_bits::tokens::TextSection,
        index: usize,
    },
    End {
        section: bevy_bits::tokens::TextSection,
    },
}

impl TypeWriterSectionBuffer {
    pub fn new(section: bevy_bits::tokens::TextSection, font: &DialogueBoxFont) -> Self {
        // let section = section.bevy_section(font.font.clone(), font.font_size, font.default_color);

        Self {
            state: SectionBufferState::First { section },
        }
    }

    pub fn current_section(&self) -> bevy_bits::tokens::TextSection {
        match &self.state {
            SectionBufferState::First { section } => bevy_bits::tokens::TextSection {
                color: section.color.clone(),
                effects: section.effects.clone(),
                text: Cow::Owned(section.text[..1].to_string()),
            },
            SectionBufferState::Repeated { section, index } => bevy_bits::tokens::TextSection {
                color: section.color.clone(),
                effects: section.effects.clone(),
                text: Cow::Owned(section.text[..*index].to_owned()),
            },
            SectionBufferState::End { section } => section.clone(),
        }
    }

    pub fn advance(&mut self) -> SectionOccurance {
        let section = match &mut self.state {
            SectionBufferState::First { section } => SectionOccurance::First({
                let space = section.text.find(" ").unwrap_or(section.text.len() - 1);
                let mut text = String::with_capacity(space);
                text.push_str(&section.text[..1]);
                for _ in 0..space {
                    text.push(' ');
                }

                bevy_bits::tokens::TextSection {
                    color: section.color.clone(),
                    effects: section.effects.clone(),
                    text: Cow::Owned(text),
                }
            }),
            SectionBufferState::Repeated { section, index } => {
                *index += 1;

                let text = if section.text.as_bytes()[index.saturating_sub(1)] != b' ' {
                    let mut buf = section.text[..*index].to_owned();
                    if let Some(space) = section.text[*index..].find(" ") {
                        for _ in 0..space + 1 {
                            buf.push(' ');
                        }
                    } else {
                        for _ in *index..section.text.len() {
                            buf.push(' ');
                        }
                    }
                    buf
                } else {
                    section.text[..*index].to_owned()
                };

                SectionOccurance::Repeated(bevy_bits::tokens::TextSection {
                    color: section.color.clone(),
                    effects: section.effects.clone(),
                    text: Cow::Owned(text),
                })
            }
            SectionBufferState::End { section } => SectionOccurance::End(section.clone()),
        };

        let new_state = match &self.state {
            SectionBufferState::First { section } => {
                if section.text.len() == 1 {
                    Some(SectionBufferState::End {
                        section: section.clone(),
                    })
                } else {
                    Some(SectionBufferState::Repeated {
                        section: section.clone(),
                        index: 1,
                    })
                }
            }
            SectionBufferState::Repeated { section, index } => {
                (section.text.len() == *index).then(|| SectionBufferState::End {
                    section: section.clone(),
                })
            }
            _ => None,
        };

        if let Some(new_state) = new_state {
            self.state = new_state;
        }

        section
    }

    pub fn finish(&mut self) {
        self.state = match self.state.clone() {
            SectionBufferState::First { section } => SectionBufferState::End { section },
            SectionBufferState::Repeated { section, .. } => SectionBufferState::End { section },
            SectionBufferState::End { section } => SectionBufferState::End { section },
        }
    }

    pub fn finished(&self) -> bool {
        matches!(self.state, SectionBufferState::End { .. })
    }
}

pub trait WithBox {
    /// Maps outgoing [`FragmentEvent`] data into [`DialogueBoxEvent`] events that are binded to a [`DialogueBoxBundle`].
    fn spawn_with_box(
        self,
        commands: &mut Commands,
        asset_server: &AssetServer,
        texture_atlases: &mut Assets<TextureAtlasLayout>,
    );
}

impl<T> WithBox for T
where
    T: IntoFragment<bevy_bits::DialogueBoxToken>,
{
    fn spawn_with_box(
        self,
        commands: &mut Commands,
        asset_server: &AssetServer,
        texture_atlases: &mut Assets<TextureAtlasLayout>,
    ) {
        let dialogue_box = DialogueBoxBundle {
            atlas: DialogueBoxAtlas::new(
                asset_server,
                texture_atlases,
                "Scalable txt screen x1.png",
                UVec2::new(16, 16),
            ),
            dimensions: DialogueBoxDimensions::new(20, 4),
            spatial: SpatialBundle::from_transform(
                Transform::default()
                    .with_scale(Vec3::new(3.0, 3.0, 1.0))
                    .with_translation(Vec3::new(-500.0, 0.0, 0.0)),
            ),
            ..Default::default()
        };

        let box_entity = commands.spawn_empty().id();
        self.once()
            .on_start(spawn_dialogue_box(
                box_entity,
                TypeWriterBundle {
                    font: DialogueBoxFont {
                        font_size: 32.0,
                        default_color: bevy::color::Color::WHITE,
                        font: asset_server.load("joystix monospace.otf"),
                    },
                    state: TypeWriterState::new(20.0),
                    text_anchor: Anchor::TopLeft,
                    spatial: SpatialBundle::from_transform(
                        Transform::default().with_scale(Vec3::new(1.0 / 3.0, 1.0 / 3.0, 1.0)),
                    ),
                    ..Default::default()
                },
                dialogue_box,
            ))
            .on_end(despawn_dialogue_box(box_entity))
            .map_event(move |event| DialogueBoxEvent {
                event: event.clone(),
                entity: box_entity,
            })
            .spawn_fragment::<bevy_bits::DialogueBoxToken>(commands);
    }
}

/// Spawns a [`DialogueBoxBundle`] with a [`TypeWriterBundle`] child.
///
/// A dialogue box's text is spawned with the [`TypeWriterBundle`], meaning it is affected by two
/// [`Transform`]s.
pub fn spawn_dialogue_box(
    entity: Entity,
    type_writer: TypeWriterBundle,
    dialogue_box: DialogueBoxBundle,
) -> impl Fn(Commands) {
    move |mut commands: Commands| {
        let inner_width = dialogue_box.dimensions.inner_width;
        let inner_height = dialogue_box.dimensions.inner_height;
        let tile_size = dialogue_box.atlas.tile_size;
        let transform = dialogue_box.spatial.transform;
        let texture = dialogue_box.atlas.texture.clone();
        let atlas_layout = dialogue_box.atlas.atlas_layout.clone();
        let type_writer = type_writer.clone();

        // dialogue_box has to be cloned here so that we can make this closure comply with bevy's
        // `IntoSystem` trait
        commands
            .entity(entity)
            .insert(dialogue_box.clone())
            .with_children(|parent| {
                parent.spawn((type_writer.clone(), TextMaterialMarkerNone));
                parent.spawn((
                    type_writer.clone(),
                    TextMaterialMarker::<WaveMaterial>::new(),
                    RenderLayers::layer(WAVE_MATERIAL_LAYER),
                ));

                let width = 2 + inner_width;
                let height = 2 + inner_height;

                for y in 0..height {
                    for x in 0..width {
                        #[allow(clippy::collapsible_else_if)]
                        let current_component = if y == 0 {
                            if x == 0 {
                                DialogueBoxComponent::TopLeft
                            } else if x < width - 1 {
                                DialogueBoxComponent::Top
                            } else {
                                DialogueBoxComponent::TopRight
                            }
                        } else if y > 0 && y < height - 1 {
                            if x == 0 {
                                DialogueBoxComponent::MiddleLeft
                            } else if x < width - 1 {
                                DialogueBoxComponent::Middle
                            } else {
                                DialogueBoxComponent::MiddleRight
                            }
                        } else {
                            if x == 0 {
                                DialogueBoxComponent::BottomLeft
                            } else if x < width - 1 {
                                DialogueBoxComponent::Bottom
                            } else {
                                DialogueBoxComponent::BottomRight
                            }
                        };

                        parent.spawn((
                            SpriteBundle {
                                texture: texture.clone(),
                                transform: Transform::default().with_translation(Vec3::new(
                                    x as f32 * tile_size.x as f32,
                                    -(y as f32 * tile_size.y as f32),
                                    0.0,
                                )),
                                ..Default::default()
                            },
                            TextureAtlas {
                                layout: atlas_layout.clone(),
                                index: current_component.atlas_index(),
                            },
                        ));
                    }
                }
            });
    }
}

pub fn despawn_dialogue_box(dialogue_box: Entity) -> impl Fn(Commands) {
    move |mut commands: Commands| {
        commands.entity(dialogue_box).despawn_recursive();
    }
}

enum DialogueBoxComponent {
    TopLeft,
    Top,
    TopRight,
    MiddleLeft,
    Middle,
    MiddleRight,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl DialogueBoxComponent {
    pub fn atlas_index(&self) -> usize {
        match self {
            Self::TopLeft => 0,
            Self::Top => 1,
            Self::TopRight => 2,
            Self::MiddleLeft => 3,
            Self::Middle => 4,
            Self::MiddleRight => 5,
            Self::BottomLeft => 6,
            Self::Bottom => 7,
            Self::BottomRight => 8,
        }
    }
}

impl<Data> crate::dialogue::fragment::IntoFragment<Data> for bevy_bits::DialogueBoxToken
where
    Data: From<bevy_bits::DialogueBoxToken> + crate::dialogue::fragment::FragmentData,
{
    type Fragment = crate::dialogue::fragment::Leaf<bevy_bits::DialogueBoxToken>;

    fn into_fragment(
        self,
        _: &mut bevy::prelude::Commands,
    ) -> (Self::Fragment, crate::dialogue::fragment::FragmentNode) {
        crate::dialogue::fragment::Leaf::new(self)
    }
}
