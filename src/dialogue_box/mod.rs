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

pub mod audio;
mod material;
mod text;
mod type_writer;

pub use text::*;
pub use type_writer::*;

/// Attaches to a [`crate::dialogue::FragmentEvent<DialogueBoxToken>`] and displays it in a dialogue box.
pub struct DialogueBoxPlugin;

impl Plugin for DialogueBoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DialogueBoxEvent>()
            .add_plugins(Material2dPlugin::<WaveMaterial>::default())
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
            // .on_start(crate::characters::portrait::init_portrait(
            //     Transform::from_translation(Vec3::default().with_x(-200.0)),
            // ))
            .on_end(crate::characters::portrait::despawn_portrait)
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
