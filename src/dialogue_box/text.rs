use super::{DialogueBoxId, DialogueBoxRegistry};
use crate::dialogue_box::type_writer::TypeWriter;
use crate::dialogue_parser::{DialogueTextSection, Effect, ShaderEffect};
use crate::{dialogue::*, dialogue_parser};
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

pub struct DialogueBoxTextPlugin {
    font_path: String,
}

impl DialogueBoxTextPlugin {
    pub fn new<P: AsRef<Path>>(font_path: &P) -> Self {
        Self {
            font_path: String::from(font_path.as_ref().to_str().expect("invalid font path")),
        }
    }
}

impl Plugin for DialogueBoxTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<WaveMaterial>::default())
            .add_event::<DialogueBoxEvent>()
            .insert_resource(DialogueBoxFontPath(self.font_path.clone()))
            .add_systems(
                Startup,
                (
                    setup_font,
                    init_effect_material::<WaveMaterial, WAVE_MATERIAL_LAYER>,
                ),
            )
            .add_systems(
                Update,
                (
                    start_type_writers,
                    update_type_writers,
                    resize_text_effect_textures,
                ),
            );
    }
}

trait TextMaterial {
    fn init(texture: Handle<Image>) -> Self;
}

pub const WAVE_MATERIAL_LAYER: usize = 1;

#[derive(AsBindGroup, Debug, Clone, Asset, TypePath)]
struct WaveMaterial {
    #[texture(0)]
    #[sampler(1)]
    texture: Handle<Image>,
}

impl TextMaterial for WaveMaterial {
    fn init(texture: Handle<Image>) -> Self {
        Self { texture }
    }
}

impl Material2d for WaveMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/text/wave.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/text/wave.wgsl".into()
    }
}

#[derive(Resource)]
struct DialogueBoxFontPath(String);

#[derive(Resource)]
struct DialogueBoxFont(Handle<Font>);

fn setup_font(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    path: Res<DialogueBoxFontPath>,
) {
    let font = DialogueBoxFont(asset_server.load(&path.0));
    commands.insert_resource(font);
}

#[derive(Event, Clone)]
pub struct DialogueBoxEvent(pub DialogueEvent, pub DialogueBoxId);

// TODO: this method can lead to duplication of shader text entities if mutliple sections use the
// same shader effect
fn start_type_writers(
    mut commands: Commands,
    font: Res<DialogueBoxFont>,
    mut reader: EventReader<DialogueBoxEvent>,
    mut writer: EventWriter<DialogueEndEvent>,
    registry: Res<DialogueBoxRegistry>,
) {
    for DialogueBoxEvent(event, box_id) in reader.read() {
        info!("received dialogue event: {event:?}, attached to {box_id:?}");

        let Some(box_desc) = registry.table.get(box_id) else {
            writer.send(event.id.end());
            error!("could not find dialogue box {box_id:?} for event {event:?}");
            return;
        };

        commands.spawn((
            TypeWriter::new_start(
                dialogue_parser::parse_dialogue(
                    &mut &*event.dialogue.clone(),
                    TextStyle {
                        font: font.0.clone(),
                        font_size: 32.0,
                        color: Color::WHITE,
                    },
                ),
                10.0,
            ),
            DialogueText {
                text: None,
                text_effects: Vec::new(),
                default_bundle: Text2dBundle {
                    text: Text::default(),
                    text_anchor: Anchor::TopLeft,
                    text_2d_bounds: Text2dBounds {
                        size: Vec2::new(
                            (box_desc.dimensions.inner_width as f32 + 1.0)
                                * box_desc.tile_size.x as f32
                                * box_desc.transform.scale.x,
                            (box_desc.dimensions.inner_height as f32 + 1.0)
                                * box_desc.tile_size.y as f32
                                * box_desc.transform.scale.x,
                        ),
                    },
                    ..Default::default()
                },
            },
            TransformBundle::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
            event.id,
            *box_id,
        ));
    }
}

#[derive(Component)]
struct AwaitingInput;

#[derive(Component)]
pub struct DialogueText {
    text: Option<Entity>,
    text_effects: Vec<(Entity, Effect)>,
    default_bundle: Text2dBundle,
}

impl DialogueText {
    pub fn display_dialogue(
        &mut self,
        commands: &mut Commands,
        sections: &[DialogueTextSection],
        dialogue_text: &mut Query<&mut Text>,
        parent: Entity,
    ) {
        for section in sections.iter() {
            if let Some(effect) = section
                .effect
                .and_then(|e| e.requires_shader().then_some(e))
            {
                let sections = sections
                    .iter()
                    .map(|s| {
                        if Some(effect) == s.effect {
                            s.section.clone()
                        } else {
                            let mut section = s.section.clone();
                            section.style.color = Color::NONE;
                            section
                        }
                    })
                    .collect();

                if let Some(entity) = self
                    .text_effects
                    .iter()
                    .find_map(|(e, ef)| (*ef == effect).then_some(e))
                {
                    if let Ok(mut text) = dialogue_text.get_mut(*entity) {
                        text.sections = sections;
                    } else {
                        error!("dialogue effect entity is invalid");
                    }
                } else {
                    let mut default_bundle = self.default_bundle.clone();
                    default_bundle.text.sections = sections;
                    commands.entity(parent).with_children(|parent| {
                        let entity = parent.spawn((default_bundle, effect.render_layer())).id();
                        self.text_effects.push((entity, effect));
                    });
                }
            } else if sections
                .iter()
                .any(|s| !s.effect.is_some_and(|e| e.requires_shader()))
            {
                let sections = sections
                    .iter()
                    .map(|s| {
                        if s.effect.is_none_or(|e| !e.requires_shader()) {
                            s.section.clone()
                        } else {
                            let mut section = s.section.clone();
                            section.style.color = Color::NONE;
                            section
                        }
                    })
                    .collect();

                if let Some(entity) = self.text {
                    if let Ok(mut text) = dialogue_text.get_mut(entity) {
                        text.sections = sections;
                    } else {
                        error!("dialogue normal text entity is invalid");
                    }
                } else {
                    let mut default_bundle = self.default_bundle.clone();
                    default_bundle.text.sections = sections;
                    commands.entity(parent).with_children(|parent| {
                        let entity = parent.spawn(default_bundle).id();
                        self.text = Some(entity);
                    });
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn update_type_writers(
    mut commands: Commands,
    time: Res<Time>,
    mut type_writers: Query<
        (Entity, &mut TypeWriter, &DialogueId, &mut DialogueText),
        Without<AwaitingInput>,
    >,
    mut text: Query<&mut Text>,
    finished_type_writers: Query<(Entity, &DialogueId), With<AwaitingInput>>,
    mut writer: EventWriter<DialogueEndEvent>,
    mut reader: EventReader<KeyboardInput>,
) {
    let mut input_received = false;
    for event in reader.read() {
        if event.state == ButtonState::Pressed {
            input_received = true;
        }
    }

    for (entity, mut type_writer, id, mut dialogue_text) in type_writers.iter_mut() {
        if input_received {
            type_writer.reveal_all_text();
        } else {
            type_writer
                .tick(&time, |type_writer| {
                    dialogue_text.display_dialogue(
                        &mut commands,
                        &type_writer.revealed_text_with_line_wrap(),
                        &mut text,
                        entity,
                    );
                })
                .on_finish(|| {
                    commands.entity(entity).insert(AwaitingInput);
                });
        }
    }

    if let Ok((entity, id)) = finished_type_writers.get_single() {
        if input_received {
            info!("ending dialogue event: {id:?}");
            writer.send(id.end());
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(Component)]
struct TextEffect;

fn init_effect_material<E: TextMaterial + Asset + Material2d, const LAYER: usize>(
    mut commands: Commands,
    mut custom_materials: ResMut<Assets<E>>,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window.single();
    let size = Extent3d {
        width: window.width() as u32,
        height: window.height() as u32,
        ..default()
    };

    let mut effect_target = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    effect_target.resize(size);
    let effect_target_image = images.add(effect_target);
    let effect_layer = RenderLayers::layer(LAYER);

    // Render layer into effect target texture
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                // render before the "main pass" camera
                order: -1,
                target: effect_target_image.clone().into(),
                clear_color: Color::NONE.into(),
                ..default()
            },
            ..default()
        },
        effect_layer,
    ));

    let material_handle = custom_materials.add(E::init(effect_target_image.clone()));

    // Read from the target texture into a mesh
    commands.spawn((
        MaterialMesh2dBundle {
            material: material_handle,
            mesh: meshes.add(Rectangle::default()).into(),
            ..Default::default()
        },
        effect_target_image,
        TextEffect,
    ));
}

// TODO: resizing will prevent the camera from rendering to the texture
fn resize_text_effect_textures(
    mut reader: EventReader<WindowResized>,
    image_handles: Query<&Handle<Image>, With<TextEffect>>,
    mut images: ResMut<Assets<Image>>,
) {
    // for event in reader.read() {
    //     for handle in image_handles.iter() {
    //         let size = Extent3d {
    //             width: event.width as u32,
    //             height: event.height as u32,
    //             ..default()
    //         };
    //
    //         if let Some(image) = images.get_mut(handle) {
    //             image.resize(size);
    //         }
    //     }
    // }
}
