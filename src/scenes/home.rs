use super::park::ParkScene;
use super::{Scene, SceneCommands, SceneTransition};
use crate::annual::Interactions;
use crate::characters::*;
use crate::cutscene::CutsceneFragment;
use crate::interactions::BindInteraction;
use crate::textbox::prelude::*;
use bevy::prelude::*;
use bevy_pretty_text::prelude::*;
use bevy_sequence::prelude::*;

#[derive(Default, Clone)]
pub struct HomeScene;

impl Scene for HomeScene {
    fn spawn(root: &mut EntityCommands) {
        let id = root.id();
        root.commands().queue(init(id));
        root.commands().add_scoped_systems(
            HomeScene,
            PreUpdate,
            super::scene_transition::<HomeScene, ParkScene>,
        );
    }
}

pub fn init(entity: Entity) -> impl FnOnce(&mut World) {
    move |world: &mut World| {
        if let Err(e) = world.run_system_cached_with(crate::annual::home::spawn, entity) {
            error!("failed to load home: {e}");
        }

        //let handle = world.load_asset("sounds/ambient/night2.mp3");
        //world.entity_mut(entity).with_child((
        //    AudioPlayer::new(handle),
        //    PlaybackSettings::LOOP.with_volume(Volume::new(0.5)),
        //));

        //world.commands().post_process(AmbientLight2d {
        //    brightness: 5.,
        //    color: srgb_from_hex(0x03193f),
        //    ..Default::default()
        //});

        //let player = world
        //    .query_filtered::<Entity, With<Player>>()
        //    .single(&world);
        //world.entity_mut(player).insert(FireflySpawner {
        //    max: 20,
        //    rate: 0.5,
        //    lifetime: 0.5,
        //});

        cabinet().spawn_box(&mut world.commands());
        spawn_root(
            SceneTransition::<HomeScene, ParkScene>::default()
                .always()
                .interaction(Interactions::BedroomDoor),
            &mut world.commands(),
        );
    }
}

const TRANSFORM: Transform = Transform::from_xyz(175., 175., -10.).with_scale(Vec3::splat(1. / 3.));

fn cabinet() -> impl IntoBox {
    (
        s!("I have good news `Mittens|blue`!")
            .interaction(Interactions::BedroomCabinet)
            .izzy(),
        s!("I met a very nice flower today. [0.5] And he is blue just like you!"),
        "This is a test",
    )
        .portrait_transform(TRANSFORM)
        .lock(Izzy)
        .once()
        .always()
}

fn door() -> impl IntoBox {
    "This is a door"
        .portrait_transform(TRANSFORM)
        .lock(Izzy)
        .once()
        .always()
}

//pub fn scene(
//    mut commands: Commands,
//    server: Res<AssetServer>,
//    mut input: EventReader<KeyboardInput>,
//) {
//    if input
//        .read()
//        .any(|i| i.state == ButtonState::Pressed && i.key_code == KeyCode::KeyO)
//    {
//        one().spawn_box(&mut commands);
//
//        commands.spawn((
//            AudioPlayer::new(server.load("sounds/music/quiet-night.wav")),
//            PlaybackSettings::LOOP,
//        ));
//    }
//}
//
//const OPENING_TRANSFORM: Transform =
//    Transform::from_xyz(175., 175., -10.).with_scale(Vec3::splat(1. / 3.));
//
//fn one() -> impl IntoBox<annual::ParkSceneFlower> {
//    (
//        "Hello!"
//            .flower()
//            .move_to(Izzy, Vec3::ZERO, Duration::from_secs(1)),
//        s!("<1.2>...[0.5]!").izzy().move_to(
//            Izzy,
//            Vec3::new(-20., -20., 0.),
//            Duration::from_millis(500),
//        ),
//        "Are you looking for something?".flower().move_camera_curve(
//            Flower,
//            Vec2::new(16., -16.),
//            Duration::from_secs(1),
//            EaseFunction::QuadraticInOut,
//        ),
//        s!("D-did you... [1] I mean, [0.5] are you a...")
//            .izzy()
//            .move_curve_then_bind_camera(
//                Izzy,
//                Vec2::ZERO,
//                Duration::from_secs_f32(0.5),
//                EaseFunction::QuadraticInOut,
//            ),
//        "Is something wrong?".flower(),
//        s!("Are you... [0.5] talking?").izzy().move_to(
//            Izzy,
//            Vec3::ZERO,
//            Duration::from_millis(800),
//        ),
//        "Well, are you?".flower(),
//        s!(
//            "<1.2>But you're a [0.25]<2> {`FLOWER|green`[wave]}!",
//            |frag| frag.sound("sounds/sfx/snd_bell.wav")
//        )
//        .izzy(),
//        s!("<1>Oh, I guess so...").flower(),
//    )
//        .portrait_transform(OPENING_TRANSFORM)
//        .lock(Izzy)
//        .always()
//        .once()
//        .delay(Duration::from_millis(2000), |mut commands: Commands| {
//            two().spawn_box(&mut commands);
//        })
//}
//
//fn two() -> impl IntoBox<annual::ParkSceneFlower> {
//    (
//        "Do you want to go on a walk?".izzy(),
//        "I'd love to!".flower(),
//        s!("But [0.5] I can't move.").flower(),
//    )
//        .once()
//        .always()
//        .portrait_transform(OPENING_TRANSFORM)
//        .delay(Duration::from_millis(4000), |mut commands: Commands| {
//            three().spawn_box(&mut commands);
//        })
//}
//
//fn three() -> impl IntoBox<annual::ParkSceneFlower> {
//    (
//        s!("I know! [0.25] I'll come by tomorrow.").izzy(),
//        "Okay!".flower(),
//        "I'll bring all my friends.".izzy(),
//        "I'll be right here!".flower(),
//    )
//        .once()
//        .always()
//        .portrait_transform(OPENING_TRANSFORM)
//}
