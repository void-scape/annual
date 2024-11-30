use crate::{FragmentExt, IntoFragment, Threaded};
use bevy::{asset::AssetPath, ecs::query::QuerySingleError, prelude::*};

pub trait Portrait<D: Threaded> {
    /// Initializes portrait entity in the ECS.
    ///
    /// Must be called before [`Portrait::portrait`].
    fn init_portrait(self, transform: Transform) -> impl IntoFragment<D>;

    /// Set the texture and, optionally, the position of the active character portrait
    fn portrait(
        self,
        texture: AssetPath<'static>,
        position: Option<Transform>,
    ) -> impl IntoFragment<D>;
}

impl<T, D> Portrait<D> for T
where
    T: IntoFragment<D>,
    D: Threaded,
{
    fn init_portrait(self, transform: Transform) -> impl IntoFragment<D> {
        self.on_start(init_portrait(transform))
    }

    fn portrait(
        self,
        texture: AssetPath<'static>,
        transform: Option<Transform>,
    ) -> impl IntoFragment<D> {
        self.on_start(portrait(texture, transform))
    }
}

pub fn init_portrait(
    transform: Transform,
) -> impl Fn(Commands, Query<Entity, With<PortraitMarker>>) {
    move |mut commands: Commands, portrait: Query<Entity, With<PortraitMarker>>| {
        assert!(portrait.is_empty());
        commands.spawn(PortraitBundle::new(Handle::default(), transform));
    }
}

pub fn portrait(
    texture: AssetPath<'static>,
    transform: Option<Transform>,
) -> impl Fn(Query<(&mut Handle<Image>, &mut Transform), With<PortraitMarker>>, Res<AssetServer>) {
    move |mut portrait: Query<(&mut Handle<Image>, &mut Transform), With<PortraitMarker>>,
          asset_server: Res<AssetServer>| {
        match portrait.get_single_mut() {
            Ok((mut tex, mut trans)) => {
                *tex = asset_server.load(texture.clone());
                if let Some(transform) = transform {
                    *trans = transform;
                }
            }
            Err(err) => match err {
                QuerySingleError::MultipleEntities(err) => {
                    error!("cannot yet handle mutliple portraits: {err}");
                    panic!();
                }
                QuerySingleError::NoEntities(_) => {
                    error!("A portrait entity was never initialized. Use: `characters::portrait::init_portrait` before calling `characters::portrait::portrait`: {err}");
                    panic!();
                }
            },
        }
    }
}

pub fn despawn_portrait(mut commands: Commands, portrait: Query<Entity, With<PortraitMarker>>) {
    #[allow(clippy::single_match)]
    match portrait.get_single() {
        Ok(entity) => commands.entity(entity).despawn(),
        Err(err) => match err {
            QuerySingleError::MultipleEntities(err) => {
                error!("cannot yet handle mutliple portraits: {err}");
                panic!();
            }
            _ => {}
        },
    }
}

#[derive(Bundle)]
struct PortraitBundle {
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
}

#[derive(Component)]
pub struct PortraitMarker;
