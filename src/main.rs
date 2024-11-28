#![allow(clippy::too_many_arguments)]

use bevy::prelude::*;
use bevy_bits::DialogueBoxToken;
use dialogue::fragment::*;
use dialogue_box::{DialogueTextSfx, SetDialogueTextSfx, WithBox};
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
    ("Hello...", t!("[15](speed)..."))
}

fn scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    (
        DialogueBoxToken::Section(bevy_bits::tokens::TextSection {
            text: "Hello!".into(),
            color: None,
            effects: (&[]).into(),
        }),
        DialogueBoxToken::Command(bevy_bits::TextCommand::ClearAfter(1.0)),
        // inner_seq(),
        t!("[20](speed)What are you looking for?"),
        t!("[15](speed)D-did you... [1.0](pause)I mean, [0.5](pause)are you a..."),
        t!("[20](speed)Is something wrong?"),
        "Are you... talking?",
        "Well, are you?",
        // t!(
        //     "[12](speed)But you're a [0.25](pause)[20](speed){[FLOWER](wave)!}",
        //     |frag| frag.on_start(|mut commands: Commands| commands.spawn(AudioBundle {
        //         source: asset_server.load(""),
        //         settings: PlaybackSettings {
        //             mode: bevy::audio::PlaybackMode::Despawn,
        //             ..Default::default()
        //         },
        //     }))
        // ),
        "Oh, I guess so...",
    )
        .text_sfx(AudioBundle {
            source: asset_server.load("short-beep-tone-47916.mp3"),
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Despawn,
                ..Default::default()
            },
        })
        .spawn_with_box(&mut commands, &asset_server, &mut texture_atlases);

    commands.spawn(Camera2dBundle::default());
}
