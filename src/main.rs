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
use characters::*;
use cutscene::*;
use scenes::SceneRoot;

mod animation;
mod annual;
mod asset_loading;
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

const TILE_SIZE: f32 = 8.;
const CAMERA_SCALE: f32 = 0.15;
const WIDTH: f32 = 1280.;
const HEIGHT: f32 = 720.;

fn main() {
    App::default()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: [WIDTH, HEIGHT].into(),
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
        .insert_resource(GlobalVolume::new(0.5))
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

fn startup(mut commands: Commands) {
    //commands.spawn(SceneRoot::new(scenes::park::ParkScene));
    commands.spawn(SceneRoot::new(scenes::home::BedroomScene::PotBreak));
}
