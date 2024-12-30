use super::park::ParkScene;
use super::{Scene, SceneTransition};
use crate::annual::{self, Interactions};
use crate::color::srgb_from_hex;
use crate::cutscene::CutsceneFragment;
use crate::frag_util::FragExt;
use crate::gfx::post_processing::PostProcessCommand;
use crate::interactions::BindInteraction;
use crate::textbox::frags::{textbox_once, EmptyCutscene};
use crate::textbox::prelude::*;
use crate::{characters::*, HEIGHT, WIDTH};
use bevy::audio::Volume;
use bevy::prelude::*;
use bevy_light_2d::light::AmbientLight2d;
use bevy_pretty_text::prelude::*;
use bevy_sequence::combinators::delay::run_after;
use bevy_sequence::prelude::*;
use std::time::Duration;

pub struct HomePlugin;

//#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, SystemSet)]
//pub enum HomeSystems {
//    Transition,
//}

impl Plugin for HomePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            super::scene_transition::<BedroomScene, LivingRoomScene>
                .run_if(super::scene_type_exists::<BedroomScene>),
        );
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BedroomScene {
    Test,
    PotBreak,
}

impl Scene for BedroomScene {
    fn spawn(&self, root: &mut EntityCommands) {
        let id = root.id();
        match self {
            Self::Test => root.commands().queue(init_test(id)),
            Self::PotBreak => root.commands().queue(init_pot_break(id)),
        }
    }
}

pub fn init_test(entity: Entity) -> impl FnOnce(&mut World) {
    move |world: &mut World| {
        if let Err(e) = world.run_system_cached_with(annual::bedroom::spawn, entity) {
            error!("failed to load level: {e}");
        }

        cabinet().spawn_box(&mut world.commands());
        spawn_root(
            SceneTransition::new(BedroomScene::Test, ParkScene)
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

pub fn init_pot_break(entity: Entity) -> impl FnOnce(&mut World) {
    move |world: &mut World| {
        if let Err(e) = world.run_system_cached_with(annual::bedroom::spawn, entity) {
            error!("failed to load level: {e}");
        }

        let handle = world.load_asset("sounds/sfx/wind.mp3");
        world.spawn((
            AudioPlayer::new(handle),
            PlaybackSettings::LOOP.with_volume(Volume::new(0.1)),
        ));

        world.commands().post_process(AmbientLight2d {
            brightness: 3.,
            color: srgb_from_hex(0x03193f),
            ..Default::default()
        });

        let transform = world
            .query_filtered::<&Transform, With<annual::Player>>()
            .single(world)
            .clone();
        let black = world
            .commands()
            .spawn((
                Sprite {
                    color: Color::BLACK,
                    custom_size: Some(Vec2::new(WIDTH, HEIGHT)),
                    ..Default::default()
                },
                transform.with_translation(transform.translation.xy().extend(999.)),
            ))
            .id();

        run_after(
            Duration::from_secs_f32(0.5),
            move |mut commands: Commands, asset_server: Res<AssetServer>| {
                commands.spawn((
                    AudioPlayer::new(asset_server.load("sounds/sfx/pot_break.mp3")),
                    PlaybackSettings::DESPAWN.with_volume(Volume::new(0.1)),
                ));
                run_after(
                    Duration::from_secs_f32(2.5),
                    move |mut commands: Commands| {
                        textbox_once::<()>(
                            s!("<0.2>...<1>[0.5]!").on_end(move |mut commands: Commands| {
                                commands.entity(black).despawn()
                            }),
                            &mut commands,
                        );
                    },
                    &mut commands,
                );
            },
            &mut world.commands(),
        );

        spawn_root(
            SceneTransition::new(BedroomScene::PotBreak, LivingRoomScene::PotBreak)
                .sound_with("sounds/music/home.wav", PlaybackSettings::LOOP)
                .always()
                .interaction(Interactions::BedroomDoor),
            &mut world.commands(),
        );
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum LivingRoomScene {
    PotBreak,
}

impl Scene for LivingRoomScene {
    fn spawn(&self, root: &mut EntityCommands) {
        let id = root.id();
        match self {
            Self::PotBreak => root.commands().queue(init_living_room_pot_break(id)),
        }
    }
}

pub fn init_living_room_pot_break(entity: Entity) -> impl FnOnce(&mut World) {
    move |world: &mut World| {
        if let Err(e) = world.run_system_cached_with(annual::living_room::spawn, entity) {
            error!("failed to load level: {e}");
        }

        world.commands().post_process(AmbientLight2d {
            brightness: 3.,
            color: srgb_from_hex(0x03193f),
            ..Default::default()
        });

        let entity = world
            .query_filtered::<Entity, With<annual::BrokenPot>>()
            .single(world);
        let particle = world.load_asset("particles/dust.ron");
        world.entity_mut(entity).with_child((
            bevy_enoki::ParticleSpawner::default(),
            bevy_enoki::ParticleEffectHandle(particle),
            Transform::from_xyz(10., -10., 0.),
        ));

        (
            s!("Mr. `Flower|blue`?")
                .interaction(Interactions::BrokenPot)
                .izzy(),
            s!("I'm sorry, `Izzy|green`... [1.5] Your pot is broken.").flower(),
        )
            .portrait_transform(TRANSFORM)
            .lock(Izzy)
            .once()
            .always()
            .spawn_box_with(&mut world.commands(), EmptyCutscene);
    }
}
