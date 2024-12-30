use bevy::prelude::*;

pub mod camera;
pub mod pixel_perfect;
pub mod post_processing;
pub mod zorder;

pub struct GfxPlugin;

impl Plugin for GfxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            camera::CameraPlugin,
            zorder::ZOrderPlugin,
            pixel_perfect::PixelPerfectPlugin,
            bevy_light_2d::prelude::Light2dPlugin,
        ));
    }
}
