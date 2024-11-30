use std::marker::PhantomData;

use bevy::{
    prelude::*,
    render::{render_resource::*, view::RenderLayers},
    sprite::{Material2d, MaterialMesh2dBundle},
    window::{PrimaryWindow, WindowResized},
};

use super::TypeWriterState;

#[derive(Component)]
pub struct TextMaterialMarker<M: TextMaterial>(PhantomData<M>);

impl<M: TextMaterial> TextMaterialMarker<M> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

#[derive(Component)]
pub struct TextMaterialMarkerNone;

pub trait TextMaterial: Send + Sync + 'static {
    fn init(texture: Handle<Image>) -> Self;
    fn target() -> bevy_bits::TextEffect;
}

pub const WAVE_MATERIAL_LAYER: usize = 1;

#[derive(AsBindGroup, Debug, Clone, Asset, TypePath)]
pub struct WaveMaterial {
    #[texture(0)]
    #[sampler(1)]
    texture: Handle<Image>,
}

impl TextMaterial for WaveMaterial {
    fn init(texture: Handle<Image>) -> Self {
        Self { texture }
    }

    fn target() -> bevy_bits::TextEffect {
        bevy_bits::TextEffect::Wave
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

pub fn init_effect_material<E: TextMaterial + Asset + Material2d, const LAYER: usize>(
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
            transform: Transform::from_xyz(0.0, 0.0, 500.0),
            ..Default::default()
        },
        effect_target_image,
    ));
}

pub fn remove_effects_from_type_writer(
    mut type_writers: Query<
        (&mut bevy::text::Text, &TypeWriterState),
        With<TextMaterialMarkerNone>,
    >,
) {
    for (mut text, state) in type_writers.iter_mut() {
        for (section, effect) in text.sections.iter_mut().zip(state.effect_mapping().iter()) {
            if effect.is_some() {
                section.style.color.set_alpha(0.0);
            } else {
                section.style.color.set_alpha(1.0);
            }
        }
    }
}

pub fn update_effect_type_writer<E: TextMaterial + Asset + Material2d>(
    mut type_writers: Query<(&mut bevy::text::Text, &TypeWriterState), With<TextMaterialMarker<E>>>,
) {
    for (mut text, state) in type_writers.iter_mut() {
        for (section, effect) in text.sections.iter_mut().zip(state.effect_mapping().iter()) {
            if effect.is_some_and(|e| e == E::target()) {
                section.style.color.set_alpha(1.0);
            } else {
                section.style.color.set_alpha(0.0);
            }
        }
    }
}

// TODO: resizing will prevent the camera from rendering to the texture
pub fn resize_text_effect_textures(
    mut reader: EventReader<WindowResized>,
    image_handles: Query<&Handle<Image>>,
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
