use super::camera::{CameraSystem, MainCamera};
use crate::{color::srgb_from_hex, physics::debug::ShowCollision, HEIGHT, WIDTH};
use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    image::ImageSamplerDescriptor,
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
    window::WindowResized,
};

pub const HIGH_RES_LAYER: RenderLayers = RenderLayers::layer(1);

pub struct PixelPerfectPlugin;

impl Plugin for PixelPerfectPlugin {
    fn build(&self, app: &mut App) {
        setup_camera(app.world_mut());
        app.add_systems(Update, fit_canvas).add_systems(
            PostUpdate,
            move_canvas_and_outer_camera
                .before(TransformSystem::TransformPropagate)
                .after(CameraSystem::UpdateCamera),
        );
        // .add_systems(
        //     PostUpdate,
        //     (correct_camera
        //         .after(CameraSystem::UpdateCamera)
        //         .before(TransformSystem::TransformPropagate),),
        // )
        // .add_systems(PreUpdate, remove_offset);
    }
}

#[derive(Component)]
struct Canvas;

#[derive(Component)]
pub struct OuterCamera;

fn setup_camera(world: &mut World) {
    let canvas_size = Extent3d {
        width: WIDTH as u32,
        height: HEIGHT as u32,
        ..default()
    };

    let mut canvas = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size: canvas_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        sampler: bevy::image::ImageSampler::Descriptor(ImageSamplerDescriptor::nearest()),
        ..default()
    };

    canvas.resize(canvas_size);
    let image_handle = world.add_asset(canvas);

    world.commands().spawn((
        Camera2d,
        Camera {
            hdr: true,
            clear_color: ClearColorConfig::Custom(srgb_from_hex(0x6fa14d)),
            order: -1,
            target: RenderTarget::Image(image_handle.clone()),
            ..Default::default()
        },
        Tonemapping::TonyMcMapface,
        MainCamera,
        Msaa::Off,
    ));

    world
        .commands()
        .spawn((Sprite::from_image(image_handle), Canvas, HIGH_RES_LAYER));

    world.commands().spawn((
        Camera2d,
        Camera {
            hdr: true,
            ..Default::default()
        },
        OuterCamera,
        HIGH_RES_LAYER,
        Msaa::Off,
    ));
}

fn fit_canvas(
    mut resize_events: EventReader<WindowResized>,
    mut projection: Single<&mut OrthographicProjection, With<OuterCamera>>,
) {
    for event in resize_events.read() {
        let h_scale = event.width / WIDTH;
        let v_scale = event.height / HEIGHT;
        projection.scale = 1. / h_scale.min(v_scale);
    }
}

// this makes the physics overlay render to the high res camera, but doubles point lights
fn move_canvas_and_outer_camera(
    debug: Res<ShowCollision>,
    camera: Single<&mut Transform, (With<OuterCamera>, Without<Canvas>)>,
    canvas: Single<&mut Transform, (With<Canvas>, Without<OuterCamera>)>,
    main_camera: Single<&Transform, (With<MainCamera>, Without<OuterCamera>, Without<Canvas>)>,
) {
    if debug.0 {
        camera.into_inner().translation = main_camera.translation;
        canvas.into_inner().translation = main_camera.translation;
    } else {
        camera.into_inner().translation = Vec3::ZERO;
        canvas.into_inner().translation = Vec3::ZERO;
    }
}

// #[derive(Component)]
// struct TempOffset(Vec3);
//
// fn correct_camera(
//     mut commands: Commands,
//     main_camera_query: Option<Single<(&mut Transform, Option<&Binded>), With<MainCamera>>>,
//     outer_camera_query: Option<Single<&mut Transform, (With<OuterCamera>, Without<MainCamera>)>>,
//     mut binded_query: Query<&mut Transform, (Without<MainCamera>, Without<OuterCamera>)>,
// ) {
//     if let Some((mut inner, binded)) = main_camera_query.map(|q| q.into_inner()) {
//         if let Some(mut outer) = outer_camera_query.map(|q| q.into_inner()) {
//             let rounded = inner.translation.round();
//             outer.translation = inner.translation - rounded;
//             inner.translation = rounded;
//
//             if let Some((entity, Ok(mut binded))) = binded.map(|b| (b.0, binded_query.get_mut(b.0)))
//             {
//                 let offset = binded.translation - rounded;
//                 binded.translation -= offset;
//                 commands.entity(entity).insert(TempOffset(offset));
//             }
//         }
//     }
// }
//
// fn remove_offset(
//     mut commands: Commands,
//     mut offset_query: Query<(Entity, &mut Transform, &TempOffset)>,
// ) {
//     for (entity, mut transform, offset) in offset_query.iter_mut() {
//         transform.translation += offset.0;
//         commands.entity(entity).remove::<TempOffset>();
//     }
// }
