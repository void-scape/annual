use super::{IntoBox, TextBoxContext};
use crate::TextBox;
use bevy::prelude::*;
use bevy_sequence::prelude::FragmentExt;

#[derive(Component)]
#[require(Sprite)]
pub struct PortraitEntity(Entity);

impl PortraitEntity {
    pub fn new(entity: Entity) -> Self {
        Self(entity)
    }

    pub fn entity(&self) -> Entity {
        self.0
    }
}

pub trait TextBoxPortrait<C> {
    fn portrait(self, path: &'static str) -> impl IntoBox<C>;
    fn portrait_transform(self, transform: Transform) -> impl IntoBox<C>;
}

impl<C, T> TextBoxPortrait<C> for T
where
    T: IntoBox<C>,
    C: 'static,
{
    fn portrait(self, path: &'static str) -> impl IntoBox<C> {
        self.on_start(
            move |InRef(ctx): InRef<TextBoxContext<C>>,
                  asset_server: Res<AssetServer>,
                  textbox_query: Query<&Children, With<TextBox>>,
                  mut portrait_query: Query<&mut Sprite, With<PortraitEntity>>| {
                if let Ok(children) = textbox_query.get(ctx.entity()) {
                    for child in children.iter() {
                        if let Ok(mut sprite) = portrait_query.get_mut(*child) {
                            sprite.image = asset_server.load(path);
                        }
                    }
                }
            },
        )
    }

    fn portrait_transform(self, transform: Transform) -> impl IntoBox<C> {
        self.on_start(
            move |InRef(ctx): InRef<TextBoxContext<C>>,
                  textbox_query: Query<&Children, With<TextBox>>,
                  mut portrait_query: Query<&mut Transform, With<PortraitEntity>>| {
                if let Ok(children) = textbox_query.get(ctx.entity()) {
                    for child in children.iter() {
                        if let Ok(mut t) = portrait_query.get_mut(*child) {
                            *t = transform;
                        }
                    }
                }
            },
        )
    }
}
