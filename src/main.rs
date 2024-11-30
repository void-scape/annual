#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

use std::time::Duration;

use bevy::prelude::*;
use characters::portrait::Portrait;
use dialogue::fragment::*;
use dialogue_box::{DialogueBoxDescriptor, WithBox};
use macros::t;

mod characters;
mod dialogue;
mod dialogue_box;
mod editor;
mod ldtk;

fn main() {
    App::default()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((
            editor::EditorPlugin,
            dialogue_box::DialogueBoxPlugin,
            dialogue::DialoguePlugin,
            ldtk::LdtkPlugin,
        ))
        .add_systems(Update, bevy_bits::close_on_escape)
        .add_systems(Startup, scene)
        .run();
}

fn one() -> impl IntoFragment<bevy_bits::DialogueBoxToken> {
    use characters::*;
    (
        "Hello!"
            .init_portrait(Transform::from_xyz(-400.0, 200.0, 0.0).with_scale(Vec3::splat(0.2)))
            .flower(),
        t!("<7>...[0.5]!").sans(),
        "Are you looking for something?".flower(),
        t!("D-did you... [1] I mean, [0.5] are you a...").sans(),
        "Is something wrong?".flower(),
        t!("Are you... [0.5] talking?").sans(),
        "Well, are you?".flower(),
        t!("<12>But you're a [0.25]<20> {`FLOWER`[wave]}!", |frag| frag
            .on_start(
                |mut commands: Commands, asset_server: Res<AssetServer>| {
                    commands.spawn(AudioBundle {
                        source: asset_server.load("snd_bell.wav"),
                        settings: PlaybackSettings {
                            mode: bevy::audio::PlaybackMode::Despawn,
                            ..Default::default()
                        },
                    });
                }
            ))
        .sans(),
        "Oh, I guess so...".flower(),
    )
        .delay(Duration::from_millis(2000), |mut commands: Commands| {
            two()
                .dialogue_box(commands.spawn_empty().id(), &DESC)
                .spawn_fragment(&mut commands);
        })
}

fn two() -> impl IntoFragment<bevy_bits::DialogueBoxToken> {
    use characters::*;
    (
        "Do you want to go on a walk?"
            .init_portrait(Transform::from_xyz(-400.0, 200.0, 0.0).with_scale(Vec3::splat(0.2)))
            .sans(),
        "I'd love to!".flower(),
        t!("But [0.5] I can't move.").flower(),
    )
        .delay(Duration::from_millis(4000), |mut commands: Commands| {
            three()
                .dialogue_box(commands.spawn_empty().id(), &DESC)
                .spawn_fragment(&mut commands);
        })
}

fn three() -> impl IntoFragment<bevy_bits::DialogueBoxToken> {
    use characters::*;
    (
        t!("I know! [0.25] I'll come by tomorrow.")
            .init_portrait(Transform::from_xyz(-400.0, 200.0, 0.0).with_scale(Vec3::splat(0.2)))
            .sans(),
        "Okay!".flower(),
        "I'll bring all my friends.".sans(),
        "I'll be waiting.".flower(),
    )
}

fn scene(mut commands: Commands) {
    use characters::*;
    one()
        .dialogue_box(commands.spawn_empty().id(), &DESC)
        .spawn_fragment(&mut commands);
}

const DESC: DialogueBoxDescriptor = DialogueBoxDescriptor {
    transform: Transform::from_xyz(-500.0, 0.0, 0.0).with_scale(Vec3::new(3.0, 3.0, 1.0)),
    dimensions: dialogue_box::DialogueBoxDimensions::new(20, 4),
    atlas: dialogue_box::DialogueBoxAtlasDescriptor {
        texture: "Scalable txt screen x1.png",
        tile_size: UVec2::new(16, 16),
    },
    font: dialogue_box::DialogueBoxFontDescriptor {
        font_size: 32.0,
        default_color: Color::WHITE,
        font: "joystix monospace.otf",
    },
};
