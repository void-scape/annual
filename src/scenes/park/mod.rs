use self::fireflies::FireflySpawner;
use self::player::Player;
use super::{Scene, SceneCommands};
use crate::annual::{self, Interactions};
use crate::characters::*;
use crate::color::srgb_from_hex;
use crate::cutscene::CutsceneFragment;
use crate::frag_util::FragExt;
use crate::gfx::camera::CameraCurveFragment;
use crate::gfx::post_processing::PostProcessCommand;
use crate::interactions::BindInteraction;
use crate::textbox::prelude::*;
use bevy::audio::Volume;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy_light_2d::light::AmbientLight2d;
use bevy_pretty_text::prelude::*;
use bevy_seedling::sample::SamplePlayer;
use bevy_sequence::prelude::*;
use std::time::Duration;

mod fireflies;

pub struct ParkPlugin;

impl Plugin for ParkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                scene,
                fireflies::spawn_fireflies::<ParkScene>,
                fireflies::update_lifetime,
            )
                .run_if(super::scene_type_exists::<ParkScene>),
        );
    }
}

#[derive(Default, Clone, PartialEq)]
pub struct ParkScene;

impl Scene for ParkScene {
    fn spawn(&self, root: &mut EntityCommands) {
        let id = root.id();
        let mut commands = root.commands();
        commands.queue(init(id));
    }
}

pub fn init(entity: Entity) -> impl FnOnce(&mut World) {
    move |world: &mut World| {
        if let Err(e) = world.run_system_cached_with(crate::annual::park::spawn, entity) {
            error!("failed to load park: {e}");
        }

        let handle = world.load_asset("sounds/ambient/night2.mp3");
        world
            .entity_mut(entity)
            .with_child(SamplePlayer::new(handle));

        world.commands().post_process(AmbientLight2d {
            brightness: 5.,
            color: srgb_from_hex(0x03193f),
            ..Default::default()
        });

        let player = world
            .query_filtered::<Entity, With<Player>>()
            .single(&world);
        world.entity_mut(player).insert(FireflySpawner {
            max: 20,
            rate: 0.5,
            lifetime: 0.5,
        });

        (
            s!("Oh, `Mr. Tree|green`[0.25], you are so very big!").textbox(),
            "Do you have any pretty birds?".textbox(),
        )
            .once()
            .interaction(Interactions::LargeTree)
            .spawn_box_with(&mut world.commands(), ());
    }
}

pub fn scene(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut input: EventReader<KeyboardInput>,
) {
    if input
        .read()
        .any(|i| i.state == ButtonState::Pressed && i.key_code == KeyCode::KeyO)
    {
        one().spawn_box(&mut commands);

        commands.spawn(SamplePlayer::new(
            server.load("sounds/music/quiet-night.wav"),
        ));
    }
}

const OPENING_TRANSFORM: Transform =
    Transform::from_xyz(175., 175., -10.).with_scale(Vec3::splat(1. / 3.));

fn one() -> impl IntoBox<annual::ParkSceneFlower> {
    (
        "Hello!"
            .flower()
            .move_to(Izzy, Vec3::ZERO, Duration::from_secs(1)),
        s!("<1.2>...[0.5]!").izzy().move_to(
            Izzy,
            Vec3::new(-20., -20., 0.),
            Duration::from_millis(500),
        ),
        "Are you looking for something?".flower().move_camera_curve(
            Flower,
            Vec2::new(16., -16.),
            Duration::from_secs(1),
            EaseFunction::QuadraticInOut,
        ),
        s!("D-did you... [1] I mean, [0.5] are you a...")
            .izzy()
            .move_curve_then_bind_camera(
                Izzy,
                Vec2::ZERO,
                Duration::from_secs_f32(0.5),
                EaseFunction::QuadraticInOut,
            ),
        "Is something wrong?".flower(),
        s!("Are you... [0.5] talking?").izzy().move_to(
            Izzy,
            Vec3::ZERO,
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

fn two() -> impl IntoBox<annual::ParkSceneFlower> {
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

fn three() -> impl IntoBox<annual::ParkSceneFlower> {
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
