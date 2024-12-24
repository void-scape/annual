use bevy::prelude::*;

pub mod camera;
pub mod post_processing;

pub struct GfxPlugin;

impl Plugin for GfxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((camera::CameraPlugin, bevy_light_2d::prelude::Light2dPlugin));
    }
}
