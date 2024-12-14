use crate::dialogue_box::{
    audio::{RevealedTextSfx, TextSfxSettings},
    IntoBox,
};
use crate::dialogue_box::{BoxContext, DialogueBox};
use bevy::{asset::AssetPath, prelude::*};

/// Configuration for dialogue box text sound effects.
///
/// Use [`Sfx::reveal`] to set descriptor for a fragment.
#[derive(Default)]
pub struct SfxDescriptor {
    pub path: AssetPath<'static>,
    pub playback_settings: PlaybackSettings,
    pub sfx_settings: TextSfxSettings,
}

#[allow(dead_code)]
impl SfxDescriptor {
    pub fn from_path(path: impl Into<AssetPath<'static>>) -> Self {
        Self {
            path: path.into(),
            ..Default::default()
        }
    }

    pub fn with_playback(mut self, playback_settings: PlaybackSettings) -> Self {
        self.playback_settings = playback_settings;
        self
    }

    pub fn with_sfx_settings(mut self, sfx_settings: TextSfxSettings) -> Self {
        self.sfx_settings = sfx_settings;
        self
    }
}

impl From<&'static str> for SfxDescriptor {
    fn from(value: &'static str) -> Self {
        Self::from_path(value)
    }
}

/// Configure dialogue box text sound effects for a fragment.
pub trait Sfx {
    fn reveal<C>(self, desc: SfxDescriptor) -> impl IntoBox<C>
    where
        C: Component,
        Self: IntoBox<C>;
}

impl<T> Sfx for T {
    fn reveal<C>(self, _desc: SfxDescriptor) -> impl IntoBox<C>
    where
        C: Component,
        Self: IntoBox<C>,
    {
        self
        // .on_start_ctx(
        // move |ctx: In<BoxContext<C>>,
        //       asset_server: Res<AssetServer>,
        //       mut boxes: Query<&mut RevealedTextSfx, With<DialogueBox>>| {
        //     if let Ok(mut sfx) = boxes.get_mut(ctx.entity()) {
        //         sfx.bundle = AudioBundle {
        //             source: asset_server.load(&desc.path),
        //             settings: desc.playback_settings,
        //         };
        //         sfx.settings = desc.sfx_settings;
        //     } else {
        //         warn!("could not set reveal sfx for box: {}", ctx.entity());
        //     }
        // },
        // )
    }
}
