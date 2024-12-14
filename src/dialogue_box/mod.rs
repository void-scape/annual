// use audio::{DeletedTextSfx, RevealedTextSfx};
use bevy::{
    asset::{AssetPath, RenderAssetUsages},
    prelude::*,
    render::storage::ShaderStorageBuffer,
    sprite::{Anchor, Material2dPlugin},
    text::Update2dText,
};
use bevy_bits::text::{
    IndexedCommand, IndexedTextMod, TextMod, TypeWriterCommand, TypeWriterSection,
};
use bevy_sequence::prelude::*;
use material::{TextMaterial2dPlugin, UpdateTextEffects};
use text::WaveMaterial;
use type_writer::{TypeWriterIndex, TypeWriterTimer};
// use material::{TextMaterialMarker, TextMaterialMarkerNone, WaveMaterial};
use std::{borrow::Cow, marker::PhantomData, time::Duration};

// pub use material::{DIALOGUE_BOX_RENDER_LAYER, WAVE_MATERIAL_LAYER};
// pub mod audio;
mod material;
// mod text;
// mod type_writer;
mod effect;
mod text;
mod type_writer;

fn some_function(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WaveMaterial>>,
    mut storage: ResMut<Assets<ShaderStorageBuffer>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2d);
    // let mat = materials.add(TextEffectMaterial {
    //     texture: asset_server.load("flower.png"),
    //     // texture: texture.clone(),
    //     atlas_uvs: storage.add(ShaderStorageBuffer::new(
    //         // bytemuck::cast_slice(&atlas_uvs),
    //         bytemuck::cast_slice(&[0., 0., 0., 0.]),
    //         RenderAssetUsages::RENDER_WORLD,
    //     )),
    // });
    //
    // let mesh = meshes.add(Rectangle::new(20., 20.));
    //
    // commands.spawn((
    //     Mesh2d(mesh.clone()),
    //     MeshMaterial2d(mat.clone()),
    //     Transform::from_xyz(-100., 0., 0.),
    // ));
    //
    // commands.spawn((
    //     Mesh2d(meshes.add(Rectangle::new(20., 20.))),
    //     MeshMaterial2d(mat.clone()),
    //     Transform::from_xyz(100., 0., 0.),
    // ));

    let val = "Im between two!";
    commands.spawn((
        TypeWriterIndex(val.len()),
        TypeWriterTimer(Timer::new(
            Duration::from_secs_f32(1. / 1.),
            TimerMode::Repeating,
        )),
        TypeWriterSection {
            text: bevy_bits::text::Text {
                value: Cow::Borrowed(val),
                modifiers: &[
                    IndexedTextMod {
                        start: 3,
                        end: 5,
                        text_mod: TextMod::Wave,
                    },
                    IndexedTextMod {
                        start: 6,
                        end: 7,
                        text_mod: TextMod::Wave,
                    },
                    IndexedTextMod {
                        start: 0,
                        end: 2,
                        text_mod: TextMod::Wave,
                    },
                ],
            },
            commands: &[],
        },
        Transform::from_scale(Vec3::splat(4.)).with_translation(Vec3::new(100., 100., 0.)),
        // DialogueBox,
    ));

    let val = "Hello, World";
    commands.spawn((
        TypeWriterIndex(val.len()),
        TypeWriterTimer(Timer::new(
            Duration::from_secs_f32(1.),
            TimerMode::Repeating,
        )),
        TextColor(LinearRgba::RED.into()),
        TypeWriterSection {
            text: bevy_bits::text::Text {
                value: Cow::Borrowed(val),
                modifiers: &[
                    IndexedTextMod {
                        start: 0,
                        end: 2,
                        text_mod: TextMod::Wave,
                    },
                    IndexedTextMod {
                        start: 5,
                        end: 8,
                        text_mod: TextMod::Wave,
                    },
                ],
            },
            commands: &[], // &[IndexedCommand {
                           //     index: 13,
                           //     command: TypeWriterCommand::AwaitClear,
                           // }],
        },
        Transform::from_scale(Vec3::splat(4.)),
        // DialogueBox,
    ));
}

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct DialogueBox;

// pub use type_writer::*;
//
// pub const DIALOGUE_BOX_SPRITE_Z: f32 = -1.;
// pub const DIALOGUE_BOX_TEXT_Z: f32 = 0.;
//
/// Attaches to a [`crate::dialogue::FragmentEvent<DialogueBoxToken>`] and displays it in a dialogue box.
pub struct DialogueBoxPlugin;

impl Plugin for DialogueBoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<WaveMaterial>::default())
            .add_systems(
                Update,
                (
                    type_writer::update_section_with_index,
                    type_writer::scroll_text,
                ),
            )
            .add_systems(
                PostUpdate,
                (
                    effect::compute_info,
                    effect::flush_outdated_effect_glyphs,
                    effect::extract_effect_glyphs,
                    effect::order_effect_glyphs,
                    effect::update_buffers,
                )
                    .chain()
                    .after(Update2dText)
                    .in_set(UpdateTextEffects),
            );

        app.add_systems(Startup, some_function);
        // .add_event::<FragmentEvent<BoxToken>>()
        // .add_plugins(Material2dPlugin::<WaveMaterial>::default())
        // .add_systems(
        //     Startup,
        //     (
        //         init_camera,
        //         material::init_effect_material::<
        //             WaveMaterial,
        //             // TODO: wtf
        //             // material::WAVE_MATERIAL_LAYER,
        //             2,
        //         >,
        //     ),
        // )
        // .add_systems(
        //     Update,
        //     (
        //         text::handle_dialogue_box_events,
        //         material::remove_effects_from_type_writer,
        //         material::update_effect_type_writer::<WaveMaterial>,
        //         material::resize_text_effect_textures,
        //     ),
        // );
    }
}

// fn init_camera(mut commands: Commands) {
//     commands.spawn((
//         Camera {
//             // Render after the main camera
//             order: 1,
//             clear_color: Color::NONE.into(),
//             ..default()
//         },
//         DIALOGUE_BOX_RENDER_LAYER,
//     ));
// }
//
// #[derive(Debug, Clone)]
// pub struct BoxToken(pub TypeWriterToken, pub BoxEntity);
//
// pub trait IntoBox<C = EmptyCutscene>: IntoFragment<BoxContext<C>, BoxToken> {}
// impl<C, T> IntoBox<C> for T where T: IntoFragment<BoxContext<C>, BoxToken> {}
//
// /// Spawns a dialogue box with a texture atlas, font, and position.
// #[derive(Default, Clone, Component)]
// #[require(TypeWriter, Transform, Visibility)]
// pub struct DialogueBox {
//     atlas: Atlas,
//     dimensions: Dimensions,
//     sfx: TextSfx,
// }
//
// // impl DialogueBoxBundle {
// //     pub fn text_bounds(&self) -> Text2dBounds {
// //         Text2dBounds {
// //             size: Vec2::new(
// //                 (self.atlas.tile_size.x * self.dimensions.inner_width as u32 + 1) as f32
// //                     * self.dimensions.scale.x,
// //                 (self.atlas.tile_size.y * self.dimensions.inner_height as u32 + 1) as f32
// //                     * self.dimensions.scale.y,
// //             ),
// //         }
// //     }
// // }
//
// #[derive(Component, Default, Clone)]
// pub struct Atlas {
//     pub atlas_layout: Handle<TextureAtlasLayout>,
//     pub texture: Handle<Image>,
//     pub tile_size: UVec2,
// }
//
// #[derive(Component, Debug, Default, Clone, Copy)]
// pub struct AtlasDescriptor {
//     pub texture: &'static str,
//     pub tile_size: UVec2,
// }
//
// impl Atlas {
//     pub fn new<'a>(
//         asset_server: &AssetServer,
//         texture_atlases: &mut Assets<TextureAtlasLayout>,
//         texture: impl Into<AssetPath<'a>>,
//         tile_size: UVec2,
//     ) -> Self {
//         let texture_atlas = TextureAtlasLayout::from_grid(tile_size, 3, 3, None, None);
//         let atlas_layout = texture_atlases.add(texture_atlas);
//
//         Self {
//             texture: asset_server.load(texture.into()),
//             tile_size,
//             atlas_layout,
//         }
//     }
// }
//
// #[derive(Component, Debug, Default, Clone, Copy)]
// pub struct Dimensions {
//     pub inner_width: usize,
//     pub inner_height: usize,
//     pub scale: Vec3,
// }
//
// impl Dimensions {
//     pub const fn new(inner_width: usize, inner_height: usize) -> Self {
//         Self {
//             inner_width,
//             inner_height,
//             scale: Vec3::ONE,
//         }
//     }
//
//     pub const fn new_with_scale(inner_width: usize, inner_height: usize, scale: Vec3) -> Self {
//         Self {
//             inner_width,
//             inner_height,
//             scale,
//         }
//     }
// }
//
// #[derive(Bundle, Default, Clone)]
// pub struct TextSfx {
//     pub reveal: RevealedTextSfx,
//     pub delete: DeletedTextSfx,
// }
//
/// Represents an empty cutscene.
#[derive(Component, Debug)]
pub struct EmptyCutscene;

#[derive(Component, Debug, Clone)]
pub struct BoxEntity(Entity);

#[derive(Component, Debug)]
pub struct BoxContext<Cutscene = EmptyCutscene>(BoxEntity, PhantomData<fn() -> Cutscene>);
//
// impl<C> Clone for BoxContext<C> {
//     fn clone(&self) -> Self {
//         Self(self.0.clone(), self.1)
//     }
// }
//
// impl<C> BoxContext<C> {
//     pub fn entity(&self) -> Entity {
//         self.0.entity()
//     }
//
//     pub fn new(entity: Entity) -> Self {
//         Self(BoxEntity(entity), PhantomData)
//     }
// }
//
// impl BoxEntity {
//     pub fn entity(&self) -> Entity {
//         self.0
//     }
// }
//
// pub trait SpawnBox {
//     fn spawn_box<Cutscene>(self, commands: &mut Commands, desc: &'static DialogueBoxDescriptor)
//     where
//         Cutscene: Component,
//         Self: IntoFragment<BoxContext<Cutscene>, BoxToken>;
// }
//
// impl<T> SpawnBox for T {
//     fn spawn_box<Cutscene>(self, commands: &mut Commands, desc: &'static DialogueBoxDescriptor)
//     where
//         Cutscene: Component,
//         Self: IntoFragment<BoxContext<Cutscene>, BoxToken>,
//     {
//         let entity = BoxContext(
//             BoxEntity(
//                 commands
//                     .spawn_empty()
//                     .insert((DialogueBox, DIALOGUE_BOX_RENDER_LAYER))
//                     .id(),
//             ),
//             PhantomData,
//         );
//
//         let (fragment, tree) = self
//             .once()
//             .on_start(spawn_dialogue_box(entity.0 .0, desc))
//             .on_end(despawn_dialogue_box(entity.0 .0))
//             .into_fragment(&entity, commands);
//
//         let portrait = commands
//             .spawn_empty()
//             .insert((
//                 PortraitBundle::new_empty(desc.portrait),
//                 DIALOGUE_BOX_RENDER_LAYER,
//             ))
//             .id();
//         commands.entity(entity.0 .0).add_child(portrait);
//         crate::dialogue::fragment::spawn_fragment(fragment, entity, tree, commands);
//     }
// }
//
// #[derive(Debug, Clone)]
// pub struct DialogueBoxDescriptor {
//     pub dimensions: Dimensions,
//     pub transform: Transform,
//     pub atlas: AtlasDescriptor,
//     pub font: FontDescriptor,
//     pub portrait: Transform,
// }
//
// /// Spawns a [`DialogueBoxBundle`] with a [`TypeWriterBundle`] child.
// ///
// /// A dialogue box's text is spawned with the [`TypeWriterBundle`], meaning it is affected by two
// /// [`Transform`]s.
// pub fn spawn_dialogue_box(
//     entity: Entity,
//     desc: &'static DialogueBoxDescriptor,
// ) -> impl Fn(Commands, Res<AssetServer>, ResMut<Assets<TextureAtlasLayout>>) {
//     move |mut commands: Commands,
//           asset_server: Res<AssetServer>,
//           mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>| {
//         let transform = desc.transform;
//
//         let dialogue_box = DialogueBoxBundle {
//             atlas: Atlas::new(
//                 &asset_server,
//                 &mut texture_atlases,
//                 desc.atlas.texture,
//                 desc.atlas.tile_size,
//             ),
//             dimensions: desc.dimensions,
//             spatial: SpatialBundle::from_transform(transform),
//             ..Default::default()
//         };
//
//         let type_writer = TypeWriter {
//             font: type_writer::Font {
//                 handle: asset_server.load(desc.font.path),
//                 size: desc.font.size,
//                 color: desc.font.color,
//             },
//             // state: TypeWriterState::new(35.),
//             // text_anchor: Anchor::TopLeft,
//             // text_2d_bounds: dialogue_box.text_bounds(),
//             // ..Default::default()
//         };
//
//         let inner_width = dialogue_box.dimensions.inner_width;
//         let inner_height = dialogue_box.dimensions.inner_height;
//         let scale = dialogue_box.dimensions.scale;
//         let tile_size = dialogue_box.atlas.tile_size;
//         let texture = dialogue_box.atlas.texture.clone();
//         let atlas_layout = dialogue_box.atlas.atlas_layout.clone();
//         let type_writer = type_writer.clone();
//
//         // dialogue_box has to be cloned here so that we can make this closure comply with bevy's
//         // `IntoSystem` trait
//         commands
//             .entity(entity)
//             .insert(dialogue_box.clone())
//             .with_children(move |parent| {
//                 parent.spawn((
//                     Sprite {
//                         image: asset_server.load("black.png"),
//                         color: Color::srgba(1., 1., 1., 0.8),
//                         ..Default::default()
//                     },
//                     Transform::from_scale(Vec3::splat(10.))
//                         .with_translation(Vec3::new(0., 0., -100.)),
//                     DIALOGUE_BOX_RENDER_LAYER,
//                 ));
//
//                 parent.spawn((
//                     type_writer.clone(),
//                     TextMaterialMarkerNone,
//                     DIALOGUE_BOX_RENDER_LAYER,
//                 ));
//                 parent.spawn((
//                     type_writer.clone(),
//                     TextMaterialMarker::<WaveMaterial>::new(),
//                     WAVE_MATERIAL_LAYER,
//                 ));
//
//                 let width = 2 + inner_width;
//                 let height = 2 + inner_height;
//
//                 for y in 0..height {
//                     for x in 0..width {
//                         #[allow(clippy::collapsible_else_if)]
//                         let current_component = if y == 0 {
//                             if x == 0 {
//                                 DialogueBoxComponent::TopLeft
//                             } else if x < width - 1 {
//                                 DialogueBoxComponent::Top
//                             } else {
//                                 DialogueBoxComponent::TopRight
//                             }
//                         } else if y > 0 && y < height - 1 {
//                             if x == 0 {
//                                 DialogueBoxComponent::MiddleLeft
//                             } else if x < width - 1 {
//                                 DialogueBoxComponent::Middle
//                             } else {
//                                 DialogueBoxComponent::MiddleRight
//                             }
//                         } else {
//                             if x == 0 {
//                                 DialogueBoxComponent::BottomLeft
//                             } else if x < width - 1 {
//                                 DialogueBoxComponent::Bottom
//                             } else {
//                                 DialogueBoxComponent::BottomRight
//                             }
//                         };
//
//                         parent.spawn((
//                             Sprite {
//                                 image: texture.clone(),
//                                 ..Default::default()
//                             },
//                             Transform::default()
//                                 .with_translation(Vec3::new(
//                                     x as f32 * tile_size.x as f32 * scale.x,
//                                     -(y as f32 * tile_size.y as f32 * scale.y),
//                                     DIALOGUE_BOX_SPRITE_Z,
//                                 ))
//                                 .with_scale(scale),
//                             TextureAtlas {
//                                 layout: atlas_layout.clone(),
//                                 index: current_component.atlas_index(),
//                             },
//                             DIALOGUE_BOX_RENDER_LAYER,
//                         ));
//                     }
//                 }
//             });
//     }
// }
//
// pub fn despawn_dialogue_box(dialogue_box: Entity) -> impl Fn(Commands, Query<Entity>) {
//     move |mut commands: Commands, boxes: Query<Entity>| {
//         if boxes.get(dialogue_box).is_ok() {
//             commands.entity(dialogue_box).despawn_recursive();
//         } else {
//             error!("tried to despawn dialogue box that does not exist: {dialogue_box}");
//         }
//     }
// }
//
// enum DialogueBoxComponent {
//     TopLeft,
//     Top,
//     TopRight,
//     MiddleLeft,
//     Middle,
//     MiddleRight,
//     BottomLeft,
//     Bottom,
//     BottomRight,
// }
//
// impl DialogueBoxComponent {
//     pub fn atlas_index(&self) -> usize {
//         match self {
//             Self::TopLeft => 0,
//             Self::Top => 1,
//             Self::TopRight => 2,
//             Self::MiddleLeft => 3,
//             Self::Middle => 4,
//             Self::MiddleRight => 5,
//             Self::BottomLeft => 6,
//             Self::Bottom => 7,
//             Self::BottomRight => 8,
//         }
//     }
// }
//
// // impl<C> crate::dialogue::fragment::IntoFragment<BoxContext<C>, BoxToken>
// //     for bevy_bits::DialogueBoxToken
// // {
// //     type Fragment = crate::dialogue::fragment::Leaf<BoxToken>;
// //
// //     fn into_fragment(
// //         self,
// //         context: &BoxContext<C>,
// //         _: &mut bevy::prelude::Commands,
// //     ) -> (Self::Fragment, crate::dialogue::fragment::FragmentNode) {
// //         crate::dialogue::fragment::Leaf::new(BoxToken(self, context.0.clone()))
// //     }
// // }
// //
// // impl<C> crate::dialogue::fragment::IntoFragment<BoxContext<C>, BoxToken> for &'static str {
// //     type Fragment = crate::dialogue::fragment::Leaf<BoxToken>;
// //
// //     fn into_fragment(
// //         self,
// //         context: &BoxContext<C>,
// //         _: &mut bevy::prelude::Commands,
// //     ) -> (Self::Fragment, crate::dialogue::fragment::FragmentNode) {
// //         crate::dialogue::fragment::Leaf::new(BoxToken(self.into(), context.0.clone()))
// //     }
// // }
// //
// // impl<C> crate::dialogue::fragment::IntoFragment<BoxContext<C>, BoxToken> for String {
// //     type Fragment = crate::dialogue::fragment::Leaf<BoxToken>;
// //
// //     fn into_fragment(
// //         self,
// //         context: &BoxContext<C>,
// //         _: &mut bevy::prelude::Commands,
// //     ) -> (Self::Fragment, crate::dialogue::fragment::FragmentNode) {
// //         crate::dialogue::fragment::Leaf::new(BoxToken(self.into(), context.0.clone()))
// //     }
// // }
