#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

use asset_loading::AssetState;
use bevy::{
    audio::Volume,
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
    render::{
        settings::{RenderCreation, WgpuSettings},
        RenderPlugin,
    },
};
// use camera::CameraFragment;
// use characters::Izzy;
// use dialogue::fragment::*;
// use dialogue::{fragment::*, FragmentEndEvent, FragmentEvent, FragmentId};
// use dialogue_box::{BoxToken, DialogueBoxDescriptor, IntoBox, SpawnBox, DIALOGUE_BOX_SPRITE_Z};
// use macros::t;
use std::time::Duration;

mod animation;
mod asset_loading;
mod text;
// mod camera;
// mod characters;
mod collision;
// mod cutscene;
// mod dialogue;
// mod dialogue_box;
mod flower;
// mod interactions;
mod ldtk;
// mod player;

fn main() {
    App::default()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        // WARN this is a native only feature. It will not work with webgl or webgpu
                        features: bevy::render::render_resource::WgpuFeatures::POLYGON_MODE_LINE,
                        ..default()
                    }),
                    ..default()
                }),
            FrameTimeDiagnosticsPlugin,
        ))
        .add_plugins((
            asset_loading::AssetLoadingPlugin,
            bevy_sequence::SequencePlugin,
            ldtk::LdtkPlugin,
            // player::PlayerPlugin,
            flower::FlowerPlugin,
            // dialogue_box::DialogueBoxPlugin,
            // dialogue::DialoguePlugin,
            // cutscene::CutscenePlugin,
            // camera::CameraPlugin,
            collision::CollisionPlugin,
            // interactions::InteractionPlugin,
        ))
        .add_systems(Update, bevy_bits::close_on_escape)
        .add_systems(Startup, load_level)
        // .add_systems(OnEnter(AssetState::Loaded), scene)
        .run();
}

fn load_level(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut proj = OrthographicProjection::default_2d();
    proj.scale = 0.5;
    commands.spawn((Camera2d, Transform::from_xyz(500., -750., 0.), proj));
    commands.spawn(DynamicSceneRoot(
        asset_server.load("ldtk/annual_scene/annual.scn.ron"),
    ));
}

// #[derive(Component)]
// pub struct Opening;
//
// fn one() -> impl IntoBox<Opening> {
//     use characters::*;
//     use cutscene::CutsceneFragment;
//     (
//         "Hello!"
//             .flower()
//             .move_to(Izzy, Vec3::new(20., 15., 0.), Duration::from_secs(1)),
//         t!("<7>...[0.5]!").izzy().move_to(
//             Izzy,
//             Vec3::new(40., 20., 0.),
//             Duration::from_millis(500),
//         ),
//         "Are you looking for something?".flower().move_camera_to(
//             flower::Flower,
//             Vec3::ZERO,
//             Duration::from_secs(1),
//         ),
//         t!("D-did you... [1] I mean, [0.5] are you a...")
//             .izzy()
//             .move_then_bind_camera(Izzy, Vec3::ZERO, Duration::from_secs_f32(0.5)),
//         "Is something wrong?".flower().move_to(
//             Izzy,
//             Vec3::new(60., 30., 0.),
//             Duration::from_millis(1500),
//         ),
//         t!("Are you... [0.5] talking?").izzy().move_to(
//             Izzy,
//             Vec3::new(70., 50., 0.),
//             Duration::from_millis(800),
//         ),
//         "Well, are you?".flower(),
//         t!("<12>But you're a [0.25]<20> {`FLOWER`[wave]}!", |frag| frag
//             .sound("snd_bell.wav"))
//         .izzy(),
//         "Oh, I guess so...".flower(),
//     )
//         .lock(Izzy)
//         .sound_with(
//             "night.mp3",
//             PlaybackSettings::LOOP.with_volume(Volume::new(0.5)),
//         )
//         .delay(Duration::from_millis(2000), |mut commands: Commands| {
//             two().spawn_box(&mut commands, &DESC);
//         })
// }
//
// fn two() -> impl IntoBox<Opening> {
//     use characters::*;
//     (
//         "Do you want to go on a walk?".izzy(),
//         "I'd love to!".flower(),
//         t!("But [0.5] I can't move.").flower(),
//     )
//         .delay(Duration::from_millis(4000), |mut commands: Commands| {
//             three().spawn_box(&mut commands, &DESC);
//         })
// }
//
// fn three() -> impl IntoBox<Opening> {
//     use characters::*;
//     (
//         t!("I know! [0.25] I'll come by tomorrow.").izzy(),
//         "Okay!".flower(),
//         "I'll bring all my friends.".izzy(),
//         "I'll be right here!".flower(),
//     )
// }
//
// fn scene(mut commands: Commands) {
//     use collision::trigger::*;
//     use collision::*;
//
//     commands.spawn((
//         SpatialBundle::from_transform(Transform::from_xyz(700., 700., 0.)),
//         Trigger(Collider::from_rect(Vec2::ZERO, Vec2::splat(40.))),
//         TriggerLayer(0),
//     ));
//
//     // commands.spawn((
//     //     Opening,
//     //     Transform::default().with_translation(Vec3::new(800., 800., 0.)),
//     // ));
//     //
//     // crate::dialogue::fragment::run_after(
//     //     Duration::from_secs(1),
//     //     |mut commands: Commands| one().bind_camera(Izzy).spawn_box(&mut commands, &DESC),
//     //     &mut commands,
//     // );
// }
//
// const DESC: DialogueBoxDescriptor = DialogueBoxDescriptor {
//     transform: Transform::from_xyz(-250., -50., 0.),
//     dimensions: dialogue_box::DialogueBoxDimensions::new_with_scale(15, 4, Vec3::new(3., 3., 1.)),
//     atlas: dialogue_box::DialogueBoxAtlasDescriptor {
//         texture: "Scalable txt screen x1.png",
//         tile_size: UVec2::new(16, 16),
//     },
//     font: dialogue_box::DialogueBoxFontDescriptor {
//         font_size: 42.,
//         default_color: Color::WHITE,
//         font: "joystix monospace.otf",
//     },
//     portrait: Transform::IDENTITY
//         .with_translation(Vec3::new(-230., -100., DIALOGUE_BOX_SPRITE_Z - 1.))
//         .with_scale(Vec3::splat(1. / 2.0)),
// };
