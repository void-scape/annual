use super::{Continue, TextBox};
use crate::{WINDOW_HEIGHT, WINDOW_WIDTH};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::text::TextBounds;
use bevy_pretty_text::prelude::{SfxChar, TypeWriterSection};
use bevy_sequence::{fragment::DataLeaf, prelude::*};
use portrait::{Portrait, TextBoxPortrait};
use std::marker::PhantomData;

pub mod portrait;
pub mod sfx;

pub fn textbox_once<C: 'static>(section: impl IntoBox<C>, commands: &mut Commands) {
    section
        .once()
        .eval_id(
            |In(fragment): In<FragmentId>,
             //mut commands: Commands,
             frag_query: Query<&FragmentState>| {
                if let Ok(state) = frag_query.get(fragment.entity()) {
                    if state.completed > 0 {
                        // TODO: despawn crashes
                        //commands.entity(fragment.entity()).despawn();
                        false
                    } else if state.active {
                        false
                    } else {
                        true
                    }
                } else {
                    false
                }
            },
        )
        .spawn_box(commands);
}

pub trait IntoBox<C = EmptyCutscene>: IntoFragment<SectionFrag, TextBoxContext<C>> {
    fn spawn_box(self, commands: &mut Commands);
    fn spawn_box_with(self, commands: &mut Commands, _root: C);
    fn textbox(self) -> impl IntoBox<C>;
    fn textbox_with(
        self,
        f: impl Fn(Entity, &AssetServer, &mut Commands) + 'static + Send + Sync,
    ) -> impl IntoBox<C>;
}

impl<C, T> IntoBox<C> for T
where
    T: IntoFragment<SectionFrag, TextBoxContext<C>>,
    C: 'static,
{
    fn spawn_box(self, commands: &mut Commands) {
        let entity = commands.spawn(Portrait::default()).id();
        spawn_root_with(self.textbox(), commands, TextBoxContext::new(entity));
    }

    fn spawn_box_with(self, commands: &mut Commands, _root: C) {
        self.spawn_box(commands);
    }

    fn textbox(self) -> impl IntoBox<C> {
        //self.textbox_with(traditional_textbox)
        // self.textbox_with(fade_textbox)
        self.textbox_with(void_stranger_textbox)
            .portrait_transform(Transform::from_xyz(150., 130., 0.).with_scale(Vec3::splat(4.)))
    }

    fn textbox_with(
        self,
        f: impl Fn(Entity, &AssetServer, &mut Commands) + 'static + Send + Sync,
    ) -> impl IntoBox<C> {
        self.on_start(
            move |InRef(ctx): InRef<TextBoxContext<C>>,
                  mut commands: Commands,
                  asset_server: Res<AssetServer>| {
                f(ctx.entity(), &asset_server, &mut commands);
            },
        )
        .on_end(
            |InRef(ctx): InRef<TextBoxContext<C>>, mut commands: Commands| {
                commands
                    .entity(ctx.entity())
                    .despawn_descendants()
                    .clear()
                    .insert(Portrait::default());
            },
        )
    }
}

pub fn traditional_textbox(entity: Entity, asset_server: &AssetServer, commands: &mut Commands) {
    let size = Vec2::new(48. * 8., 45. * 3.);
    let offset = Vec2::new(20., -20.);

    commands
        .entity(entity)
        .insert((
            TextBox {
                text_offset: offset,
                text_bounds: TextBounds::from(size - offset * 2.),
                text_anchor: Some(Anchor::TopLeft),
                font_size: 16.,
                font: Some(asset_server.load("textbox/joystix.otf")),
            },
            SfxChar::from_source(asset_server.load("characters/izzy/girl.mp3")),
            Transform::from_xyz(-400., -150., 0.).with_scale(Vec3::splat(2.)),
        ))
        .with_child((Sprite {
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
        },))
        .with_child((
            Sprite {
                image: asset_server.load("textbox/collision_mask.png"),
                anchor: Anchor::TopLeft,
                ..Default::default()
            },
            Transform::from_translation(Vec3::default().with_z(100.)),
            Continue,
            Visibility::Hidden,
        ));
}

pub fn fade_textbox(entity: Entity, asset_server: &AssetServer, commands: &mut Commands) {
    commands
        .entity(entity)
        .insert((
            TextBox {
                text_offset: Vec2::new(400., 150.),
                text_bounds: TextBounds::from(Vec2::new(700., 100.)),
                text_anchor: Some(Anchor::TopLeft),
                font_size: 28.,
                font: Some(asset_server.load("textbox/joystix.otf")),
            },
            Transform::from_xyz(-WINDOW_WIDTH / 2., -WINDOW_HEIGHT / 2., 0.),
        ))
        .with_child((
            Sprite {
                image: asset_server.load("textbox/black.png"),
                anchor: Anchor::BottomLeft,
                ..Default::default()
            },
            Transform::from_xyz(0., 0., -1.),
        ))
        .with_child((
            Sprite {
                image: asset_server.load("textbox/collision_mask.png"),
                anchor: Anchor::BottomLeft,
                ..Default::default()
            },
            Transform::from_xyz(0., 0., 100.),
            Continue,
            Visibility::Hidden,
        ));
}

pub fn void_stranger_textbox(entity: Entity, asset_server: &AssetServer, commands: &mut Commands) {
    commands
        .entity(entity)
        .insert((
            TextBox {
                text_offset: Vec2::new(325., 225.),
                text_bounds: TextBounds::from(Vec2::new(800., 200.)),
                text_anchor: Some(Anchor::TopLeft),
                font_size: 48.,
                font: Some(asset_server.load("textbox/Pixellari.ttf")),
            },
            Transform::from_xyz(-WINDOW_WIDTH / 2., -WINDOW_HEIGHT / 2., 0.),
        ))
        .with_child((
            Sprite {
                image: asset_server.load("sprites/textbox.png"),
                anchor: Anchor::BottomLeft,
                ..Default::default()
            },
            Transform::from_xyz(0., 0., -1.).with_scale(Vec3::splat(4.)),
        ))
        .with_child((
            Sprite {
                image: asset_server.load("sprites/textbox_continue.png"),
                anchor: Anchor::BottomLeft,
                ..Default::default()
            },
            Transform::from_xyz(0., 0., 100.).with_scale(Vec3::splat(4.)),
            Continue,
            Visibility::Hidden,
        ));
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
    pub fn new(entity: Entity) -> Self {
        Self(TextBoxEntity(entity), PhantomData)
    }

    pub fn entity(&self) -> Entity {
        self.0.entity()
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
