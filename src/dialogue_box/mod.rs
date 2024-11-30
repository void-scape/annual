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

pub trait IntoBox: IntoFragment<Entity, bevy_bits::DialogueBoxToken> {}

impl<T> IntoBox for T where T: IntoFragment<Entity, bevy_bits::DialogueBoxToken> {}

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

#[derive(Component, Debug, Default, Clone)]
pub struct DialogueBoxFont {
    pub font: Handle<Font>,
    pub font_size: f32,
    pub default_color: bevy::color::Color,
}

#[derive(Component, Debug, Default, Clone)]
pub struct DialogueBoxFontDescriptor {
    pub font: &'static str,
    pub font_size: f32,
    pub default_color: bevy::color::Color,
}

#[derive(Component, Default, Clone)]
pub struct DialogueBoxAtlas {
    pub atlas_layout: Handle<TextureAtlasLayout>,
    pub texture: Handle<Image>,
    pub tile_size: UVec2,
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct DialogueBoxAtlasDescriptor {
    pub texture: &'static str,
    pub tile_size: UVec2,
}

impl DialogueBoxAtlas {
    pub fn new<'a>(
        asset_server: &AssetServer,
        texture_atlases: &mut Assets<TextureAtlasLayout>,
        texture: impl Into<AssetPath<'a>>,
        tile_size: UVec2,
    ) -> Self {
        let texture_atlas = TextureAtlasLayout::from_grid(tile_size, 3, 3, None, None);
        let atlas_layout = texture_atlases.add(texture_atlas);

        Self {
            texture: asset_server.load(texture.into()),
            tile_size,
            atlas_layout,
        }
    }
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct DialogueBoxDimensions {
    pub inner_width: usize,
    pub inner_height: usize,
}

impl DialogueBoxDimensions {
    pub const fn new(inner_width: usize, inner_height: usize) -> Self {
        Self {
            inner_width,
            inner_height,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct DialogueBoxFragmentMap(pub Vec<FragmentId>);

#[derive(Component, Clone)]
pub struct BoxEntity(Entity);

impl BoxEntity {
    pub fn entity(&self) -> Entity {
        self.0
    }
}

pub trait SpawnBox {
    /// Populates `entity` with a dialogue box and children on fragment start.
    ///
    /// ```
    /// # fn func(mut commands: Commands, descriptor: DialogueBoxDescriptor) {
    /// use dialogue_box::WithBox;
    ///
    /// let fragment = ("Hello, World!").dialogue_box(
    ///     commands.spawn_empty().id(),
    ///     &descriptor,
    /// ;
    ///
    /// fragment.spawn_fragment(&mut commands);
    /// #}
    /// ```
    fn spawn_box(self, commands: &mut Commands, desc: &'static DialogueBoxDescriptor);
}

impl<T> SpawnBox for T
where
    T: IntoFragment<BoxEntity, bevy_bits::DialogueBoxToken>,
{
    fn spawn_box(self, commands: &mut Commands, desc: &'static DialogueBoxDescriptor) {
        let entity = commands.spawn_empty().id();

        let (fragment, tree) = self
            .once()
            .on_start(spawn_dialogue_box(entity, desc))
            .on_end(crate::characters::portrait::despawn_portrait)
            .into_fragment(commands);

        commands
            .entity(entity)
            .insert(DialogueBoxFragmentMap(tree.leaves()));
        crate::dialogue::fragment::spawn_fragment(fragment, tree, commands);
    }
}

#[derive(Debug, Clone)]
pub struct DialogueBoxDescriptor {
    pub dimensions: DialogueBoxDimensions,
    pub transform: Transform,
    pub atlas: DialogueBoxAtlasDescriptor,
    pub font: DialogueBoxFontDescriptor,
}

/// Spawns a [`DialogueBoxBundle`] with a [`TypeWriterBundle`] child.
///
/// A dialogue box's text is spawned with the [`TypeWriterBundle`], meaning it is affected by two
/// [`Transform`]s.
pub fn spawn_dialogue_box(
    entity: Entity,
    desc: &'static DialogueBoxDescriptor,
) -> impl Fn(Commands, Res<AssetServer>, ResMut<Assets<TextureAtlasLayout>>) {
    move |mut commands: Commands,
          asset_server: Res<AssetServer>,
          mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>| {
        let transform = desc.transform;
        let dialogue_box = DialogueBoxBundle {
            atlas: DialogueBoxAtlas::new(
                &asset_server,
                &mut texture_atlases,
                desc.atlas.texture,
                desc.atlas.tile_size,
            ),
            dimensions: desc.dimensions,
            spatial: SpatialBundle::from_transform(transform),
            ..Default::default()
        };

        let type_writer = TypeWriterBundle {
            font: DialogueBoxFont {
                font: asset_server.load(desc.font.font),
                font_size: desc.font.font_size,
                default_color: desc.font.default_color,
            },
            state: TypeWriterState::new(35.0),
            text_anchor: Anchor::TopLeft,
            spatial: SpatialBundle::from_transform(Transform::default().with_scale(Vec3::new(
                1.0 / transform.scale.x,
                1.0 / transform.scale.y,
                1.0,
            ))),
            ..Default::default()
        };

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

impl<Data> crate::dialogue::fragment::IntoFragment<BoxEntity, Data> for bevy_bits::DialogueBoxToken
where
    Data: From<bevy_bits::DialogueBoxToken> + crate::dialogue::fragment::Threaded,
{
    type Fragment = crate::dialogue::fragment::Leaf<bevy_bits::DialogueBoxToken>;

    fn into_fragment(
        self,
        _: &mut bevy::prelude::Commands,
    ) -> (Self::Fragment, crate::dialogue::fragment::FragmentNode) {
        crate::dialogue::fragment::Leaf::new(self)
    }
}
