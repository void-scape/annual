use super::{Scene, SceneCommands};
use crate::annual::Interactions;
use crate::characters::*;
use crate::cutscene::CutsceneFragment;
use crate::gfx::camera::CameraCurveFragment;
use crate::interactions::SpawnInteraction;
use crate::textbox::prelude::*;
use bevy::audio::Volume;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy_pretty_text::prelude::*;
use bevy_sequence::prelude::*;
use std::time::Duration;

pub struct ParkScene;

impl Scene for ParkScene {
    fn spawn(root: &mut EntityCommands) {
        let id = root.id();
        let mut commands = root.commands();
        commands.queue(init(id));
        commands.add_scoped_systems(ParkScene, Update, scene);
    }
}

pub fn init(entity: Entity) -> impl FnOnce(&mut World) {
    move |world: &mut World| {
        if let Err(e) = world.run_system_cached_with(crate::annual::park::spawn, entity) {
            error!("failed to load park: {e}");
        }

        let handle = world.load_asset("sounds/ambient/night2.mp3");
        world.entity_mut(entity).with_child((
            AudioPlayer::new(handle),
            PlaybackSettings::LOOP.with_volume(Volume::new(0.25)),
        ));

        [
            (s!("Wow! What a big tree."), Interactions::LargeTree),
            (
                s!("This one's a little [0.5] smaller..."),
                Interactions::SmallTree,
            ),
            (s!("You really like trees, huh?"), Interactions::TwistyTree),
        ]
        .map(|(s, i)| s.spawn_interaction(i, &mut world.commands()));
    }
}

pub fn scene(mut commands: Commands, mut input: EventReader<KeyboardInput>) {
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
    (
        "Hello!".flower().move_curve(
            Izzy,
            Vec3::new(20., 15., 0.),
            Duration::from_secs(1),
            EaseFunction::ElasticInOut,
        ),
        s!("<1.2>...[0.5]!").izzy().move_to(
            Izzy,
            Vec3::new(40., 20., 0.),
            Duration::from_millis(500),
        ),
        "Are you looking for something?".flower().move_camera_curve(
            Flower,
            Vec3::ZERO,
            Duration::from_secs(1),
            EaseFunction::ElasticIn,
        ),
        s!("D-did you... [1] I mean, [0.5] are you a...")
            .izzy()
            .move_curve_then_bind_camera(
                Izzy,
                Vec3::ZERO,
                Duration::from_secs_f32(0.5),
                EaseFunction::BackInOut,
            ),
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
        .delay(Duration::from_millis(2000), |mut commands: Commands| {
            two().spawn_box(&mut commands);
        })
}

fn two() -> impl IntoBox<Opening> {
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
