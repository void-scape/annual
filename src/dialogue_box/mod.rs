#![allow(unused)]
use crate::dialogue::{FragmentEvent, FragmentId};
use bevy::{
    asset::AssetPath,
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
    render::view::RenderLayers,
    sprite::{Anchor, Material2dPlugin, SpriteSource},
    text::{Text2dBounds, TextLayoutInfo},
    utils::HashMap,
};
use bevy_bits::TextCommand;
use material::{TextMaterialMarker, WaveMaterial, WAVE_MATERIAL_LAYER};
use std::{borrow::Cow, path::Path, time::Duration};

mod material;
mod text;

pub use text::*;

/// Attaches to a [`crate::dialogue::FragmentEvent<DialogueBoxToken>`] and displays it in a dialogue box.
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
                    // text::handle_dialogue_box_events_material::<WaveMaterial>,
                    material::resize_text_effect_textures,
                ),
            );
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

#[derive(Component, Default, Clone)]
pub struct TypeWriterState {
    timer: Timer,
    pause_timer: Timer,
    clear: bool,
    section_buf: Option<TypeWriterSectionBuffer>,
    id: Option<FragmentId>,
}

impl TypeWriterState {
    pub fn new(chars_per_sec: f32) -> Self {
        Self {
            timer: Timer::new(
                Duration::from_secs_f32(1.0 / chars_per_sec),
                TimerMode::Repeating,
            ),
            pause_timer: Timer::new(Duration::default(), TimerMode::Once),
            ..Default::default()
        }
    }

    pub fn push_section(&mut self, section: bevy_bits::tokens::TextSection, id: FragmentId) {
        self.section_buf = Some(TypeWriterSectionBuffer::new(section));
        self.id = Some(id);
    }

    pub fn push_cmd(&mut self, cmd: TextCommand, id: FragmentId) {
        self.id = Some(id);

        match cmd {
            TextCommand::Speed(speed) => {
                self.timer
                    .set_duration(Duration::from_secs_f32(1.0 / speed));
            }
            TextCommand::Pause(duration) => {
                self.pause_timer
                    .set_duration(Duration::from_secs_f32(duration));
                self.pause_timer.reset();
            }
            TextCommand::Clear => {
                self.clear = true;
            }
        }
    }

    pub fn tick(&mut self, time: &Time) -> Option<SectionOccurance<'_>> {
        if !self.pause_timer.finished() {
            self.pause_timer.tick(time.delta());
            return None;
        }

        self.timer.tick(time.delta());
        if self.section_buf.as_ref().is_some_and(|b| b.finished()) {
            self.section_buf = None;
        }

        if self.timer.finished() {
            self.section_buf.as_mut().map(|b| b.advance())
        } else {
            None
        }
    }

    pub fn finished(&mut self, input: &mut EventReader<KeyboardInput>, text: &mut Text) -> bool {
        if !self.pause_timer.finished() {
            false
        } else if self.clear {
            if input.read().next().is_some() {
                self.clear = false;
                text.sections.clear();

                true
            } else {
                false
            }
        } else {
            self.section_buf.is_none()
        }
    }

    pub fn fragment_id(&self) -> Option<FragmentId> {
        self.id
    }
}

#[derive(Component, Clone)]
struct TypeWriterSectionBuffer {
    section: bevy_bits::tokens::TextSection,
    in_progress: bevy_bits::tokens::TextSection,
    index: usize,
    finished: bool,
}

pub enum SectionOccurance<'a> {
    First(&'a bevy_bits::tokens::TextSection),
    Repeated(Cow<'a, str>),
    End,
}

impl TypeWriterSectionBuffer {
    pub fn new(section: bevy_bits::tokens::TextSection) -> Self {
        let in_progress = bevy_bits::tokens::TextSection {
            text: std::borrow::Cow::Owned(String::with_capacity(section.text.len())),
            color: section.color.clone(),
            effects: section.effects.clone(),
        };

        Self {
            index: 0,
            finished: false,
            section,
            in_progress,
        }
    }

    pub fn advance(&mut self) -> SectionOccurance<'_> {
        if !self.finished() {
            self.in_progress
                .text
                .to_mut()
                .push_str(&self.section.text[self.index..self.index + 1]);
            self.index += 1;

            let section = if self.index == 1 {
                SectionOccurance::First(&self.in_progress)
            } else {
                let str = if self.in_progress.text.as_bytes()[self.index.saturating_sub(1)] != b' '
                {
                    let mut buf = self.in_progress.text[0..self.index].to_owned();
                    if let Some(space) = self.section.text[self.index..].find(" ") {
                        for _ in 0..space + 1 {
                            buf.push(' ');
                        }
                    } else {
                        for _ in self.index..self.section.text.len() {
                            buf.push(' ');
                        }
                    }
                    Cow::Owned(buf)
                } else {
                    Cow::Borrowed(&self.section.text[..self.index])
                };

                SectionOccurance::Repeated(str)
            };

            section
        } else {
            self.finished = true;

            SectionOccurance::End
        }
    }

    pub fn finished(&self) -> bool {
        self.finished || self.section.text.len() == self.in_progress.text.len()
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
                parent.spawn(type_writer.clone());
                // parent.spawn((
                //     type_writer,
                //     TextMaterialMarker::<WaveMaterial>::new(),
                //     RenderLayers::layer(WAVE_MATERIAL_LAYER),
                // ));

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

impl crate::dialogue::fragment::IntoFragment for bevy_bits::DialogueBoxToken {
    type Fragment<Data> = crate::dialogue::fragment::Leaf<bevy_bits::DialogueBoxToken>;

    fn into_fragment<Data>(
        self,
        _: &mut bevy::prelude::Commands,
    ) -> (
        Self::Fragment<Data>,
        crate::dialogue::fragment::FragmentNode,
    ) {
        crate::dialogue::fragment::Leaf::new(self)
    }
}
