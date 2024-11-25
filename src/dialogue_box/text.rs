use super::{DialogueBoxId, DialogueBoxRegistry};
use crate::dialogue_box::type_writer::TypeWriter;
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

const WAVY_MATERIAL_LAYER: usize = 1;

impl Plugin for DialogueBoxTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<WavyMaterial>::default())
            .add_event::<WhichBox>()
            .insert_resource(DialogueBoxFontPath(self.font_path.clone()))
            .add_systems(
                Startup,
                (
                    setup_font,
                    init_effect_material::<WavyMaterial, WAVY_MATERIAL_LAYER>,
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

#[derive(AsBindGroup, Debug, Clone, Asset, TypePath)]
struct WavyMaterial {
    #[texture(0)]
    #[sampler(1)]
    texture: Handle<Image>,
}

impl TextMaterial for WavyMaterial {
    fn init(texture: Handle<Image>) -> Self {
        Self { texture }
    }
}

impl Material2d for WavyMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/wavy_text.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/wavy_text.wgsl".into()
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

    // The cube that will be rendered to the texture.
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "Hello, World!",
                TextStyle {
                    font_size: 32.0,
                    color: Color::WHITE,
                    font: asset_server.load("joystix monospace.otf"),
                },
            ),
            ..Default::default()
        },
        effect_layer.clone(),
    ));

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
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 15.0))
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        effect_layer,
    ));

    let material_handle = custom_materials.add(E::init(effect_target_image.clone()));

    // Read from the target texture into a mesh
    // commands.spawn((
    //     MaterialMesh2dBundle {
    //         material: material_handle,
    //         // TODO: delete?
    //         mesh: meshes.add(Rectangle::default()).into(),
    //         ..Default::default()
    //     },
    //     effect_target_image,
    //     TextEffect,
    // ));
}

fn resize_text_effect_textures(
    mut reader: EventReader<WindowResized>,
    image_handles: Query<&Handle<Image>, With<TextEffect>>,
    mut images: ResMut<Assets<Image>>,
) {
    for event in reader.read() {
        for handle in image_handles.iter() {
            let size = Extent3d {
                width: event.width as u32,
                height: event.height as u32,
                ..default()
            };

            if let Some(image) = images.get_mut(handle) {
                image.resize(size);
            }
        }
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
pub struct WhichBox(pub DialogueId, pub DialogueBoxId);

#[derive(Component)]
pub(super) struct DialogueText;

fn start_type_writers(
    mut commands: Commands,
    font: Res<DialogueBoxFont>,
    mut reader: EventReader<DialogueEvent>,
    mut writer: EventWriter<DialogueEndEvent>,
    mut which_box: EventReader<WhichBox>,
    registry: Res<DialogueBoxRegistry>,
) {
    let Some(WhichBox(dialogue_id, box_id)) = which_box.read().next() else {
        return;
    };

    for event in reader.read() {
        info!("received dialogue event: {event:?}, attached to {box_id:?}");

        assert_eq!(*dialogue_id, event.id);
        let Some(box_desc) = registry.table.get(box_id) else {
            writer.send(event.id.end());
            error!("could not find dialogue box {box_id:?} for event {event:?}");
            return;
        };

        commands.spawn((
            Text2dBundle {
                text: Text::from_section(
                    "",
                    TextStyle {
                        font: font.0.clone(),
                        font_size: 32.0,
                        color: Color::WHITE,
                    },
                ),
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
            event.id,
            *box_id,
            DialogueText,
        ));
    }

    which_box.clear();
}

#[derive(Component)]
struct AwaitingInput;

#[allow(clippy::type_complexity)]
fn update_type_writers(
    mut commands: Commands,
    time: Res<Time>,
    mut type_writers: Query<
        (Entity, &mut TypeWriter, &mut Text, &DialogueId),
        (Without<AwaitingInput>, With<DialogueText>),
    >,
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

    for (entity, mut type_writer, mut text, id) in type_writers.iter_mut() {
        if input_received {
            type_writer.reveal_all_text();
        } else {
            type_writer
                .tick(&time, |type_writer| {
                    text.sections = type_writer
                        .revealed_text_with_line_wrap()
                        .into_iter()
                        .map(|t| t.section)
                        .collect();

                    // let sections = type_writer.revealed_text_with_line_wrap();
                    // if sections
                    //     .iter()
                    //     .any(|s| s.effect.is_some_and(|e| e.requires_shader()))
                    // {
                    //     commands
                    //         .entity(entity)
                    //         .with_children(|parent| parent.spawn(Text2dBundle {}));
                    // }
                })
                .on_finish(|| {
                    // info!("finished dialogue event: {id:?}");
                    // info!("awaiting user input...");
                    commands.entity(entity).insert(AwaitingInput);
                });
        }
    }

    if let Ok((entity, id)) = finished_type_writers.get_single() {
        if input_received {
            info!("ending dialogue event: {id:?}");
            writer.send(id.end());
            commands.entity(entity).despawn();
        }
    }
}
