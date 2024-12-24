#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
    render::{
        settings::{RenderCreation, WgpuSettings},
        RenderPlugin,
    },
};
use characters::*;
use cutscene::*;
use textbox::*;
use scenes::{park::ParkScene, SceneRoot};

mod animation;
mod annual;
mod asset_loading;
mod characters;
mod collision;
mod curves;
mod cutscene;
mod gfx;
mod interactions;
mod scenes;
mod textbox;

const TILE_SIZE: f32 = 8.;
const CAMERA_SCALE: f32 = 0.15;

fn main() {
    App::default()
        .add_plugins((
            DefaultPlugins
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
            collision::CollisionPlugin,
            interactions::InteractionPlugin,
            scenes::ScenePlugin,
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
    commands.spawn(SceneRoot::new(ParkScene));
    //let entity = commands.spawn(SceneRoot::new(ParkScene)).id();
    //run_after(
    //    Duration::from_secs(1),
    //    move |mut commands: Commands| commands.entity(entity).despawn_recursive(),
    //    &mut commands,
    //);
}
