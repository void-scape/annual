#![allow(unused)]
use bevy::{
    asset::AssetPath,
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
    sprite::Material2dPlugin,
    utils::HashMap,
};
use material::WaveMaterial;
use std::path::Path;

mod material;
mod text;
mod tokens;
// mod type_writer;

pub use text::*;
pub use tokens::*;
// pub use type_writer::*;

use crate::dialogue::FragmentEvent;

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
                    // start_type_writers,
                    // update_type_writers,
                    material::resize_text_effect_textures,
                ),
            );
    }
}

/// Associates a [`FragmentEvent<DialogueBoxToken>`] with a specific [`DialogueBoxBundle`] entity.
#[derive(Event, Clone)]
pub struct DialogueBoxEvent {
    pub entity: Entity,
    pub event: FragmentEvent<DialogueBoxToken>,
}

/// Spawns a dialogue box with a texture atlas, font, and position.
#[derive(Bundle, Clone)]
pub struct DialogueBoxBundle {
    pub atlas: DialogueBoxAtlas,
    pub font: DialogueBoxFont,
    pub dimensions: DialogueBoxDimensions,
    pub spatial: SpatialBundle,
}

#[derive(Component, Clone)]
pub struct DialogueBoxFont {
    pub font: Handle<Font>,
    pub font_size: f32,
    pub default_color: bevy::color::Color,
}

#[derive(Component, Clone)]
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

#[derive(Component, Clone, Copy)]
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

pub fn spawn_dialogue_box(dialogue_box: Entity, bundle: DialogueBoxBundle) -> impl Fn(Commands) {
    move |mut commands: Commands| {
        let inner_width = bundle.dimensions.inner_width;
        let inner_height = bundle.dimensions.inner_height;
        let tile_size = bundle.atlas.tile_size;
        let transform = bundle.spatial.transform;
        let texture = bundle.atlas.texture.clone();
        let atlas_layout = bundle.atlas.atlas_layout.clone();

        // Bundle has to be cloned here so that we can make this closure comply with bevy's
        // `IntoSystem` trait
        commands.spawn(bundle.clone()).with_children(|parent| {
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
