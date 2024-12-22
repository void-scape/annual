use super::{IntoBox, TextBoxContext};
use bevy::prelude::*;
use bevy_pretty_text::prelude::*;
use bevy_sequence::prelude::*;

pub trait TextBoxSfx<C> {
    fn sfx_char(self, path: &'static str) -> impl IntoBox<C>
    where
        Self: Sized,
        Self: IntoBox<C>;
}

impl<C, T> TextBoxSfx<C> for T
where
    T: IntoBox<C>,
    C: 'static,
{
    fn sfx_char(self, path: &'static str) -> impl IntoBox<C> {
        self.on_start(
            move |InRef(ctx): InRef<TextBoxContext<C>>,
                  mut commands: Commands,
                  asset_server: Res<AssetServer>| {
                commands
                    .entity(ctx.entity())
                    .insert(SfxChar(asset_server.load(path)));
            },
        )
    }
}
