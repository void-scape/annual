use crate::dialogue::fragment::FragmentExt;
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
        texture: AssetPath<'static>,
        transform: Option<Transform>,
    ) -> impl IntoBox<C>
    where
        C: Component,
        Self: IntoBox<C>,
    {
        self.on_start_ctx(portrait::<C>(texture, transform))
    }
}

pub fn portrait<C>(
    texture: AssetPath<'static>,
    transform: Option<Transform>,
) -> impl Fn(
    In<BoxContext<C>>,
    Query<(&mut Handle<Image>, &mut Transform), With<PortraitMarker>>,
    Query<&Children, With<DialogueBox>>,
    Res<AssetServer>,
) {
    move |ctx: In<BoxContext<C>>,
          mut portraits: Query<(&mut Handle<Image>, &mut Transform), With<PortraitMarker>>,
          boxes: Query<&Children, With<DialogueBox>>,
          asset_server: Res<AssetServer>| {
        if let Ok(children) = boxes.get(ctx.entity()) {
            for child in children.iter() {
                if let Ok((mut tex, mut trans)) = portraits.get_mut(*child) {
                    *tex = asset_server.load(texture.clone());
                    if let Some(transform) = transform {
                        *trans = transform;
                    }
                }
            }
        }
    }
}

#[derive(Bundle, Default, Clone)]
pub struct PortraitBundle {
    sprite: SpriteBundle,
    marker: PortraitMarker,
}

impl PortraitBundle {
    pub fn new(texture: Handle<Image>, transform: Transform) -> Self {
        Self {
            sprite: SpriteBundle {
                texture,
                transform,
                ..Default::default()
            },
            marker: PortraitMarker,
        }
    }

    pub fn new_empty(transform: Transform) -> Self {
        Self {
            sprite: SpriteBundle {
                transform,
                ..Default::default()
            },
            marker: PortraitMarker,
        }
    }
}

#[derive(Component, Default, Clone)]
pub struct PortraitMarker;
