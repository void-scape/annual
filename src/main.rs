#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    input::{keyboard::KeyboardInput, ButtonState},
    log::LogPlugin,
    prelude::*,
    render::{
        settings::{RenderCreation, WgpuSettings},
        RenderPlugin,
    },
};
use bevy_seedling::{MainBus, VolumeNode};
use characters::*;
use cutscene::*;
use scenes::SceneRoot;

mod animation;
mod annual;
mod asset_loading;
mod audio;
mod characters;
mod color;
mod curves;
mod cutscene;
mod frag_util;
mod gfx;
mod interactions;
mod physics;
mod scenes;
mod textbox;

const TILE_SIZE: f32 = 16.;
const WINDOW_WIDTH: f32 = 1280.;
const WINDOW_HEIGHT: f32 = 720.;
const WIDTH: f32 = 320.;
const HEIGHT: f32 = 180.;

fn main() {
    App::default()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: [WINDOW_WIDTH, WINDOW_HEIGHT].into(),
                        mode: bevy::window::WindowMode::BorderlessFullscreen(
                            MonitorSelection::Primary,
                        ),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(LogPlugin {
                    filter: String::from("symphonia=warn"),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest())
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        features: bevy::render::render_resource::WgpuFeatures::POLYGON_MODE_LINE,
                        ..default()
                    }),
                    ..default()
                }),
            FrameTimeDiagnosticsPlugin,
        ))
        .add_plugins((
            asset_loading::AssetLoadingPlugin,
            bevy_sequence::SequencePlugin,
            gfx::GfxPlugin,
            textbox::TextBoxPlugin,
            characters::CharacterPlugin,
            cutscene::CutscenePlugin,
            physics::PhysicsPlugin,
            interactions::InteractionPlugin,
            scenes::ScenePlugin,
            bevy_enoki::EnokiPlugin,
            bevy_seedling::SeedlingPlugin::default(),
            audio::AnnualAudioPlugin,
        ))
        .add_systems(Update, close_on_escape)
        .add_systems(Startup, startup)
        .run();
}

fn close_on_escape(mut reader: EventReader<KeyboardInput>, mut writer: EventWriter<AppExit>) {
    for input in reader.read() {
        if input.state == ButtonState::Pressed && input.key_code == KeyCode::Escape {
            writer.send(AppExit::Success);
        }
    }
}

fn startup(
    global: Single<&mut VolumeNode, With<MainBus>>,
    mut commands: Commands,
    _server: Res<AssetServer>,
) {
    global.into_inner().0.set(0.25);

    commands.spawn(SceneRoot::new(scenes::park::ParkScene));
    //commands.spawn(SceneRoot::new(scenes::home::BedroomScene::PotBreak));
    // commands.spawn(SceneRoot::new(scenes::sandbox::SandboxScene));
}
