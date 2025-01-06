use self::fireflies::FireflySpawner;
use self::player::Player;
use super::Scene;
use crate::annual::{self, Interactions};
use crate::cutscene::CutsceneFragment;
use crate::frag_util::FragExt;
use crate::gfx::camera::{Binded, CameraCurveFragment, MainCamera, MoveTo};
use crate::gfx::post_processing::PostProcessCommand;
use crate::gfx::zorder::YOrigin;
use crate::interactions::BindInteraction;
use crate::physics::prelude::*;
use crate::textbox::prelude::*;
use crate::{characters::*, TILE_SIZE};
use bevy::core_pipeline::bloom::Bloom;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use bevy::math::NormedVectorSpace;
use bevy::prelude::*;
use bevy_light_2d::light::AmbientLight2d;
use bevy_pretty_text::prelude::*;
use bevy_seedling::sample::SamplePlayer;
use bevy_seedling::RepeatMode;
use bevy_sequence::prelude::*;
use std::time::Duration;

mod fireflies;

#[derive(Default, Component)]
#[require(YOrigin(|| YOrigin(-TILE_SIZE * 3.5)), StaticBody)]
#[require(Collider(|| Collider::from_circle(Vec2::new(TILE_SIZE * 2., -TILE_SIZE * 3.5), 5.)))]
struct ParkTreeComponents1;

#[derive(Default, Component)]
#[require(YOrigin(|| YOrigin(-TILE_SIZE * 4.25)), StaticBody)]
#[require(Collider(|| Collider::from_rect(Vec2::new(TILE_SIZE * 1.75, -TILE_SIZE * 4.25), Vec2::new(28., 8.))))]
struct ParkTreeComponents2;

#[derive(Default, Component)]
#[require(YOrigin(|| YOrigin(-TILE_SIZE * 3.)), StaticBody)]
#[require(Collider(|| Collider::from_rect(Vec2::new(8., -48.), Vec2::new(10., 8.))))]
struct LampComponents;

#[derive(Default, Component)]
#[require(YOrigin(|| YOrigin(-TILE_SIZE * 2.)), StaticBody)]
#[require(Collider(|| Collider::from_rect(Vec2::new(3., -32.), Vec2::new(27., 8.))))]
struct BenchComponents;

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
        )
        .register_required_components::<annual::ParkTree1, ParkTreeComponents1>()
        .register_required_components::<annual::ParkTree2, ParkTreeComponents1>()
        .register_required_components::<annual::ParkTree3, ParkTreeComponents2>()
        .register_required_components::<annual::Lamp, LampComponents>()
        .register_required_components::<annual::Bench, BenchComponents>()
        .register_required_components_with::<annual::Flower, YOrigin>(|| YOrigin(-TILE_SIZE))
        .register_required_components_with::<annual::Flower1, YOrigin>(|| YOrigin(-TILE_SIZE))
        .register_required_components_with::<annual::Flower2, YOrigin>(|| YOrigin(-TILE_SIZE));
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
        world.commands().entity(entity).with_child((
            SamplePlayer::new(handle),
            bevy_seedling::sample::PlaybackSettings::LOOP,
        ));

        world.commands().post_process(AmbientLight2d {
            brightness: 0.1,
            color: Color::WHITE,
        });
        world.commands().post_process(Bloom::NATURAL);

        let player = world.query_filtered::<Entity, With<Player>>().single(world);
        world.entity_mut(player).insert(FireflySpawner {
            max: 20,
            rate: 0.5,
            lifetime: 0.5,
        });

        (
            s!("Oh, `Mr. Tree|green`,[0.25] you are so very big!").textbox(),
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

        let handle = server.load("sounds/music/quiet-night.wav");
        commands.spawn((
            SamplePlayer::new(handle),
            bevy_seedling::sample::PlaybackSettings {
                mode: RepeatMode::RepeatEndlessly,
                volume: 0.85,
            },
        ));
    }
}

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
            "<1.2>But you're a [0.25]<2> {`FLOWER|green`[Wave]}!",
            |frag| frag.sound("sounds/sfx/snd_bell.wav")
        )
        .izzy(),
        s!("<1>Oh, I guess so...").flower(),
    )
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
}
