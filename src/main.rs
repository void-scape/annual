#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

use bevy::{audio::Volume, math::VectorSpace, prelude::*};
use dialogue::fragment::*;
use dialogue_box::{DialogueBoxDescriptor, IntoBox, SpawnBox};
use macros::t;
use std::time::Duration;

mod animation;
mod asset_loading;
mod characters;
mod dialogue;
mod dialogue_box;
mod editor;
mod flower;
mod ldtk;
mod player;

fn main() {
    App::default()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((
            editor::EditorPlugin,
            asset_loading::AssetLoadingPlugin,
            ldtk::LdtkPlugin,
            player::PlayerPlugin,
            flower::FlowerPlugin,
            dialogue_box::DialogueBoxPlugin,
            dialogue::DialoguePlugin,
        ))
        .add_systems(Update, bevy_bits::close_on_escape)
        // .add_systems(Startup, scene)
        .run();
}

fn one() -> impl IntoBox {
    use characters::*;
    (
        "Hello!".flower(),
        t!("<7>...[0.5]!").izzy(),
        "Are you looking for something?".flower(),
        t!("D-did you... [1] I mean, [0.5] are you a...").izzy(),
        "Is something wrong?".flower(),
        t!("Are you... [0.5] talking?").izzy(),
        "Well, are you?".flower(),
        t!("<12>But you're a [0.25]<20> {`FLOWER`[wave]}!", |frag| frag
            .sound("snd_bell.wav"))
        .izzy(),
        "Oh, I guess so...".flower(),
    )
        .sound_with(
            "night.mp3",
            PlaybackSettings::LOOP.with_volume(Volume::new(0.5)),
        )
        .delay(Duration::from_millis(2000), |mut commands: Commands| {
            two().spawn_box(&mut commands, &DESC);
        })
}

fn two() -> impl IntoBox {
    use characters::*;
    (
        "Do you want to go on a walk?".izzy(),
        "I'd love to!".flower(),
        t!("But [0.5] I can't move.").flower(),
    )
        .delay(Duration::from_millis(4000), |mut commands: Commands| {
            three().spawn_box(&mut commands, &DESC);
        })
}

fn three() -> impl IntoBox {
    use characters::*;
    (
        t!("I know! [0.25] I'll come by tomorrow.").izzy(),
        "Okay!".flower(),
        "I'll bring all my friends.".izzy(),
        "I'll be waiting.".flower(),
    )
}

fn scene(mut commands: Commands) {
    one().spawn_box(&mut commands, &DESC)
}

const DESC: DialogueBoxDescriptor = DialogueBoxDescriptor {
    transform: Transform::from_xyz(-200.0, 0.0, 0.0).with_scale(Vec3::new(3.0, 3.0, 1.0)),
    dimensions: dialogue_box::DialogueBoxDimensions::new(15, 4),
    atlas: dialogue_box::DialogueBoxAtlasDescriptor {
        texture: "Scalable txt screen x1.png",
        tile_size: UVec2::new(16, 16),
    },
    font: dialogue_box::DialogueBoxFontDescriptor {
        font_size: 32.0,
        default_color: Color::WHITE,
        font: "joystix monospace.otf",
    },
    portrait: Transform::IDENTITY
        .with_translation(Vec3::new(-80.0, -40.0, -10.0))
        .with_scale(Vec3::splat(1.0 / 6.0)),
};
