#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

use bevy::prelude::*;
use characters::portrait::Portrait;
use dialogue::fragment::*;
use dialogue_box::{audio::SetDialogueTextSfx, WithBox};
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

fn inner_seq() -> impl IntoFragment<bevy_bits::DialogueBoxToken> {
    ("Hello...", t!("<5>..."))
}

// fn thing<D: FragmentData>(input: impl IntoFragment<D>) -> impl IntoFragment<D> {
//     input.on_start(|mut commands: Commands, asset_server: Res<AssetServer>| {
//         commands.spawn(AudioBundle {
//             source: asset_server.load("snd_bell.wav"),
//             settings: PlaybackSettings {
//                 mode: bevy::audio::PlaybackMode::Despawn,
//                 ..Default::default()
//             },
//         });
//     })
// }

fn scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    use characters::*;
    (
        inner_seq()
            .init_portrait(Transform::from_xyz(-400.0, 200.0, 0.0).with_scale(Vec3::splat(0.2)))
            .sans(),
        t!("<20>What are you looking for?").flower(),
        t!("<15>D-did you... [1] I mean, [0.5] are you a...").sans(),
        t!("<20>Is something wrong?").flower(),
        "Are you... talking?".sans(),
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
        .spawn_with_box(&mut commands, &asset_server, &mut texture_atlases);
}
