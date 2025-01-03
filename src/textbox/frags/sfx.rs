use super::{IntoBox, SectionFrag, TextBoxContext};
use bevy::prelude::*;
use bevy_pretty_text::prelude::*;
use bevy_sequence::prelude::*;

pub trait TextBoxSfx<C>
where
    Self: IntoFragment<SectionFrag, TextBoxContext<C>> + Sized,
    C: 'static,
{
    fn sfx_char(self, path: &'static str) -> impl IntoBox<C> {
        self.sfx_char_with(path, PlaybackSettings::DESPAWN)
    }

    fn sfx_char_with(self, path: &'static str, settings: PlaybackSettings) -> impl IntoBox<C> {
        self.on_start(
            move |InRef(ctx): InRef<TextBoxContext<C>>,
                  mut commands: Commands,
                  asset_server: Res<AssetServer>| {
                commands.entity(ctx.entity()).insert(SfxChar {
                    // source: asset_server.load(path),
                    settings,
                });
            },
        )
    }
}

impl<C, T> TextBoxSfx<C> for T
where
    T: IntoBox<C>,
    C: 'static,
{
}
