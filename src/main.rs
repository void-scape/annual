#![allow(clippy::too_many_arguments)]

use bevy::prelude::*;
use bevy_bits::DialogueBoxToken;
use dialogue::fragment::*;
use dialogue_box::{SetDialogueTextSfx, WithBox};
use macros::t;

mod dialogue;
mod dialogue_box;

fn main() {
    App::default()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((dialogue::DialoguePlugin, dialogue_box::DialogueBoxPlugin))
        .add_systems(Startup, scene)
        .add_systems(Update, bevy_bits::close_on_escape)
        .run();
}

fn inner_seq() -> impl IntoFragment<bevy_bits::DialogueBoxToken> {
    ("Hello...", t!("<5>..."))
}

fn thing<D: FragmentData>(input: impl IntoFragment<D>) -> impl IntoFragment<D> {
    input.on_start(|mut commands: Commands, asset_server: Res<AssetServer>| {
        commands.spawn(AudioBundle {
            source: asset_server.load("snd_bell.wav"),
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Despawn,
                ..Default::default()
            },
        });
    })
}

fn scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    (
        DialogueBoxToken::Command(bevy_bits::TextCommand::Speed(5.0)),
        DialogueBoxToken::Section(bevy_bits::tokens::TextSection {
            text: "Hello!".into(),
            color: None,
            effect: None,
        }),
        DialogueBoxToken::Command(bevy_bits::TextCommand::Delete(6)),
        DialogueBoxToken::Command(bevy_bits::TextCommand::Speed(20.0)),
        DialogueBoxToken::Command(bevy_bits::TextCommand::ClearAfter(0.2)),
        inner_seq(),
        t!("<20>What are you looking for?"),
        t!("<15>D-did you... [1] I mean, [0.5] are you a..."),
        t!("<20>Is something wrong?"),
        "Are you... talking?",
        "Well, are you?",
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
            )),
        "Oh, I guess so...",
    )
        .reveal_sfx(
            AudioBundle {
                source: asset_server.load("snd_txtsans.wav"),
                settings: PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    ..Default::default()
                },
            },
            dialogue_box::TextSfxSettings {
                pitch: 1.,
                pitch_variance: 0.0,
                rate: 1.0 / 2.0,
            },
        )
        .delete_sfx(
            AudioBundle {
                source: asset_server.load("just-a-normal-sans-sound-effect-made-with-Voicemod.mp3"),
                settings: PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    ..Default::default()
                },
            },
            dialogue_box::TextSfxSettings {
                pitch: 0.75,
                pitch_variance: 0.0,
                rate: 1.0 / 2.0,
            },
        )
        .spawn_with_box(&mut commands, &asset_server, &mut texture_atlases);

    commands.spawn(Camera2dBundle::default());
}
