use crate::dialogue_box::{BoxContext, DialogueBox, IntoBox};
use bevy::{asset::AssetPath, prelude::*};

pub trait Portrait {
    /// Set the texture and, optionally, the position of the active character portrait
    fn portrait<C>(
        self,
        texture: AssetPath<'static>,
        position: Option<Transform>,
    ) -> impl IntoBox<C>
    where
        C: Component,
        Self: IntoBox<C>;
}

impl<T> Portrait for T {
    fn portrait<C>(
        self,
        _texture: AssetPath<'static>,
        _transform: Option<Transform>,
    ) -> impl IntoBox<C>
    where
        C: Component,
        Self: IntoBox<C>,
    {
        // self.on_start_ctx(portrait::<C>(texture, transform))
        self
    }
}

pub fn portrait<C>(
    texture: AssetPath<'static>,
    transform: Option<Transform>,
) -> impl Fn(
    In<BoxContext<C>>,
    Query<(&mut Sprite, &mut Transform), With<CharacterPortrait>>,
    Query<&Children, With<DialogueBox>>,
    Res<AssetServer>,
) {
    move |ctx: In<BoxContext<C>>,
          mut portraits: Query<(&mut Sprite, &mut Transform), With<CharacterPortrait>>,
          boxes: Query<&Children, With<DialogueBox>>,
          asset_server: Res<AssetServer>| {
        if let Ok(children) = boxes.get(ctx.entity()) {
            for child in children.iter() {
                if let Ok((mut sprite, mut trans)) = portraits.get_mut(*child) {
                    sprite.image = asset_server.load(texture.clone());
                    if let Some(transform) = transform {
                        *trans = transform;
                    }
                }
            }
        }
    }
}

#[derive(Component)]
#[require(Sprite)]
pub struct CharacterPortrait;
