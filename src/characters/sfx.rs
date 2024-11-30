use crate::{
    dialogue_box::{
        audio::{RevealedTextSfx, TextSfxSettings},
        IntoBox,
    },
    FragmentExt,
};
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
    fn reveal(self, desc: SfxDescriptor) -> impl IntoBox;
}

impl<T> Sfx for T
where
    T: IntoBox,
{
    fn reveal(self, desc: SfxDescriptor) -> impl IntoBox {
        self.on_start(reveal(desc))
    }
}

fn reveal(desc: SfxDescriptor) -> impl Fn(Commands, Res<AssetServer>) {
    move |mut commands: Commands, asset_server: Res<AssetServer>| {
        commands.insert_resource(RevealedTextSfx {
            bundle: AudioBundle {
                source: asset_server.load(desc.path.clone()),
                settings: desc.playback_settings,
            },
            settings: desc.sfx_settings,
        });
    }
}
