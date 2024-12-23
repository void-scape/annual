#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

use self::{
    camera::CameraFragment,
    collision::{trigger::TriggerLayer, Collider},
    frags::portrait::TextBoxPortrait,
};
use bevy::{
    audio::Volume,
    diagnostic::FrameTimeDiagnosticsPlugin,
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
    render::{
        settings::{RenderCreation, WgpuSettings},
        RenderPlugin,
    },
};
use bevy_pretty_text::prelude::*;
use bevy_sequence::prelude::*;
use characters::*;
use cutscene::*;
use std::time::Duration;
use textbox::*;

mod animation;
mod annual;
mod asset_loading;
mod camera;
mod characters;
mod collision;
mod cutscene;
mod interactions;
mod textbox;

const TILE_SIZE: f32 = 8.;

fn main() {
    App::default()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
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
            bevy_ldtk_scene::LdtkScenePlugin,
            textbox::TextBoxPlugin,
            characters::CharacterPlugin,
            cutscene::CutscenePlugin,
            camera::CameraPlugin,
            collision::CollisionPlugin,
            interactions::InteractionPlugin,
        ))
        .add_systems(Update, close_on_escape)
        .add_systems(Startup, (annual::park::spawn, spawn_interaction_dialogue))
        .add_systems(Update, scene)
        .run();
}

fn close_on_escape(mut reader: EventReader<KeyboardInput>, mut writer: EventWriter<AppExit>) {
    for input in reader.read() {
        if input.state == ButtonState::Pressed && input.key_code == KeyCode::Escape {
            writer.send(AppExit::Success);
        }
    }
}

fn spawn_interaction_dialogue(mut commands: Commands) {
    use annual::Interactions;
    use interactions::SpawnInteraction;

    "Wow! What a big tree.".spawn_interaction(Interactions::LargeTree, &mut commands);

    s!("This one's a little [0.5] smaller...")
        .spawn_interaction(Interactions::SmallTree, &mut commands);

    "You really like trees, huh?".spawn_interaction(Interactions::TwistyTree, &mut commands);
}

fn scene(mut commands: Commands, mut input: EventReader<KeyboardInput>) {
    if input
        .read()
        .any(|i| i.state == ButtonState::Pressed && i.key_code == KeyCode::KeyO)
    {
        one().spawn_box(&mut commands);

        commands.spawn((
            Opening,
            Transform::default().with_translation(Vec3::new(700., -750., 0.)),
        ));
    }
}

const OPENING_TRANSFORM: Transform =
    Transform::from_xyz(0., 0., -10.).with_scale(Vec3::splat(1. / 6.));

#[derive(Component)]
pub struct Opening;

fn one() -> impl IntoBox<Opening> {
    use characters::*;
    use cutscene::CutsceneFragment;
    use textbox::TextBoxExt;

    (
        "Hello!"
            .flower()
            .move_to(Izzy, Vec3::new(20., 15., 0.), Duration::from_secs(1)),
        s!("<1.2>...[0.5]!").izzy().move_to(
            Izzy,
            Vec3::new(40., 20., 0.),
            Duration::from_millis(500),
        ),
        "Are you looking for something?".flower().move_camera_to(
            Flower,
            Vec3::ZERO,
            Duration::from_secs(1),
        ),
        s!("D-did you... [1] I mean, [0.5] are you a...")
            .izzy()
            .move_then_bind_camera(Izzy, Vec3::ZERO, Duration::from_secs_f32(0.5)),
        "Is something wrong?".flower().move_to(
            Izzy,
            Vec3::new(60., 30., 0.),
            Duration::from_millis(1500),
        ),
        s!("Are you... [0.5] talking?").izzy().move_to(
            Izzy,
            Vec3::new(70., 50., 0.),
            Duration::from_millis(800),
        ),
        "Well, are you?".flower(),
        s!(
            "<1.2>But you're a [0.25]<2> {`FLOWER|green`[wave]}!",
            |frag| frag.sound("sounds/sfx/snd_bell.wav")
        )
        .izzy(),
        s!("<1>Oh, I guess so...").flower(),
    )
        .portrait_transform(OPENING_TRANSFORM)
        .lock(Izzy)
        .always()
        .once()
        .sound_with(
            "sounds/ambient/night.mp3",
            PlaybackSettings::LOOP.with_volume(Volume::new(0.5)),
        )
        .delay(Duration::from_millis(2000), |mut commands: Commands| {
            two().spawn_box(&mut commands);
        })
}

fn two() -> impl IntoBox<Opening> {
    use characters::*;
    (
        "Do you want to go on a walk?".izzy(),
        "I'd love to!".flower(),
        s!("But [0.5] I can't move.").flower(),
    )
        .once()
        .always()
        .portrait_transform(OPENING_TRANSFORM)
        .delay(Duration::from_millis(4000), |mut commands: Commands| {
            three().spawn_box(&mut commands);
        })
}

fn three() -> impl IntoBox<Opening> {
    use characters::*;
    (
        s!("I know! [0.25] I'll come by tomorrow.").izzy(),
        "Okay!".flower(),
        "I'll bring all my friends.".izzy(),
        "I'll be right here!".flower(),
    )
        .once()
        .always()
        .portrait_transform(OPENING_TRANSFORM)
}
