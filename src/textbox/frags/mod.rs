use self::portrait::{Portrait, PortraitEntity};

use super::{Continue, TextBox};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::text::TextBounds;
use bevy_pretty_text::prelude::{SfxChar, TypeWriterSection};
use bevy_sequence::{fragment::DataLeaf, prelude::*};
use std::marker::PhantomData;

pub mod portrait;
pub mod sfx;

pub trait IntoBox<C = EmptyCutscene>: IntoFragment<SectionFrag, TextBoxContext<C>> {
    fn spawn_box(self, commands: &mut Commands);
}

impl<C, T> IntoBox<C> for T
where
    T: IntoFragment<SectionFrag, TextBoxContext<C>>,
    C: 'static,
{
    fn spawn_box(self, commands: &mut Commands) {
        let entity = commands.spawn(Portrait::default()).id();
        spawn_root_with(
            self.on_start(
                move |mut commands: Commands, asset_server: Res<AssetServer>| {
                    insert_box(entity, &asset_server, &mut commands)
                },
            )
            .on_end(move |mut commands: Commands| commands.entity(entity).despawn_recursive()),
            commands,
            TextBoxContext::new(entity),
        );
    }
}

pub fn insert_box(entity: Entity, asset_server: &AssetServer, commands: &mut Commands) {
    let size = Vec2::new(48. * 8., 45. * 3.);
    let offset = Vec2::new(20., -20.);

    commands
        .entity(entity)
        .insert((
            TextBox {
                text_offset: offset,
                text_bounds: TextBounds::from(size - offset * 2.),
                font_size: 16.,
                font: Some(asset_server.load("textbox/joystix.otf")),
            },
            Sprite {
                // 48 x 45
                image: asset_server.load("textbox/textbox.png"),
                anchor: Anchor::TopLeft,
                custom_size: Some(size),
                image_mode: SpriteImageMode::Sliced(TextureSlicer {
                    center_scale_mode: SliceScaleMode::Tile { stretch_value: 1. },
                    sides_scale_mode: SliceScaleMode::Tile { stretch_value: 1. },
                    border: BorderRect::square(16.),
                    ..Default::default()
                }),
                ..Default::default()
            },
            Transform::from_xyz(-400., -150., 0.).with_scale(Vec3::splat(2.)),
            SfxChar::from_source(asset_server.load("characters/izzy/girl.mp3")),
        ))
        .with_child((
            Sprite {
                image: asset_server.load("textbox/collision_mask.png"),
                anchor: Anchor::TopLeft,
                ..Default::default()
            },
            Transform::from_translation(Vec3::default().with_z(100.)),
            Continue,
            Visibility::Hidden,
        ))
        .with_child(PortraitEntity::new(entity));
}

#[derive(Debug, Component)]
pub struct EmptyCutscene;

#[derive(Debug, Component)]
pub struct TextBoxContext<Cutscene = EmptyCutscene>(TextBoxEntity, PhantomData<fn() -> Cutscene>);

impl<C> Clone for TextBoxContext<C> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1)
    }
}

impl<C> TextBoxContext<C> {
    pub fn entity(&self) -> Entity {
        self.0.entity()
    }

    pub fn new(entity: Entity) -> Self {
        Self(TextBoxEntity(entity), PhantomData)
    }
}

#[derive(Debug, Clone, Component)]
pub struct TextBoxEntity(Entity);

impl TextBoxEntity {
    pub fn entity(&self) -> Entity {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct SectionFrag {
    pub textbox: Entity,
    pub section: TypeWriterSection,
}

macro_rules! impl_into_frag {
    ($ty:ty, $x:ident, $into:expr) => {
        impl<C> IntoFragment<SectionFrag, TextBoxContext<C>> for $ty {
            fn into_fragment(
                self,
                context: &Context<TextBoxContext<C>>,
                commands: &mut Commands,
            ) -> FragmentId {
                let $x = self;
                <_ as IntoFragment<SectionFrag, TextBoxContext<C>>>::into_fragment(
                    DataLeaf::new(SectionFrag {
                        textbox: context.read().unwrap().entity(),
                        section: $into,
                    }),
                    context,
                    commands,
                )
            }
        }
    };
}

impl_into_frag!(&'static str, slf, slf.into());
impl_into_frag!(String, slf, slf.into());
impl_into_frag!(TypeWriterSection, slf, slf);
