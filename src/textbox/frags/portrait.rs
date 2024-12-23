use super::{IntoBox, TextBoxContext};
use bevy::prelude::*;
use bevy_sequence::prelude::FragmentExt;

#[derive(Debug, Default, Clone, Component)]
#[require(Transform, Visibility)]
pub struct Portrait {
    pub sprite: Sprite,
    pub transform: Transform,
}

pub trait TextBoxPortrait<C> {
    fn portrait(self, path: &'static str) -> impl IntoBox<C>;
    fn portrait_sprite(self, sprite: Sprite) -> impl IntoBox<C>;
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
                  mut portraits: Query<&mut Portrait>| {
                if let Ok(mut portrait) = portraits.get_mut(ctx.entity()) {
                    portrait.sprite.image = asset_server.load(path);
                }
            },
        )
    }

    fn portrait_sprite(self, sprite: Sprite) -> impl IntoBox<C> {
        self.on_start(
            move |InRef(ctx): InRef<TextBoxContext<C>>, mut portraits: Query<&mut Portrait>| {
                if let Ok(mut portrait) = portraits.get_mut(ctx.entity()) {
                    portrait.sprite = sprite.clone();
                }
            },
        )
    }

    fn portrait_transform(self, transform: Transform) -> impl IntoBox<C> {
        self.on_start(
            move |InRef(ctx): InRef<TextBoxContext<C>>, mut portraits: Query<&mut Portrait>| {
                if let Ok(mut portrait) = portraits.get_mut(ctx.entity()) {
                    portrait.transform = transform;
                }
            },
        )
    }
}

/// A reference to the portrait entity stored within a [`TextBox`].
#[derive(Component)]
pub struct PortraitChild(Entity);

impl PortraitChild {
    pub fn new(entity: Entity) -> Self {
        Self(entity)
    }

    pub fn entity(&self) -> Entity {
        self.0
    }
}

/// A reference to the [`TextBox`] stored within a portrait entity.
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

pub fn spawn_portrait(
    mut commands: Commands,
    portrait_query: Query<(Entity, &Portrait), Without<PortraitChild>>,
) {
    for (entity, portrait) in portrait_query.iter() {
        let id = commands
            .spawn((
                PortraitEntity::new(entity),
                portrait.sprite.clone(),
                portrait.transform,
            ))
            .id();

        commands
            .entity(entity)
            .insert(PortraitChild::new(id))
            .add_child(id);
    }
}

pub fn update_portrait(
    portrait_parents: Query<(&Portrait, &PortraitChild), Changed<Portrait>>,
    mut portraits: Query<(&mut Sprite, &mut Transform)>,
) {
    for (portrait, child) in portrait_parents.iter() {
        if let Ok((mut sprite, mut transform)) = portraits.get_mut(child.0) {
            *sprite = portrait.sprite.clone();
            *transform = portrait.transform;
        }
    }
}
