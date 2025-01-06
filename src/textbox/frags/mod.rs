use super::{Continue, TextBox};
use crate::{HEIGHT, WIDTH};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::text::TextBounds;
use bevy_pretty_text::prelude::{SfxWord, TypeWriterSection};
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
                    !(state.completed > 0 || state.active)
                    // TODO: despawn crashes
                    //commands.entity(fragment.entity()).despawn();
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
        self.textbox_with(void_stranger_textbox)
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

pub fn void_stranger_textbox(entity: Entity, _server: &AssetServer, commands: &mut Commands) {
    commands
        .entity(entity)
        .insert((Portrait::default(), TextBox::default(), SfxWord::default()));

    commands.queue(move |world: &mut World| {
        let window = world.query::<&Window>().single(world);
        let window_resolution = window.resolution.size();
        let textbox_scale = window_resolution / Vec2::new(WIDTH, HEIGHT);
        let font = world.load_asset("textbox/Pixellari.ttf");
        let textbox_sprite = world.load_asset("sprites/textbox.png");
        let continue_sprite = world.load_asset("sprites/textbox_continue.png");

        world
            .entity_mut(entity)
            .insert((
                Portrait {
                    transform: Transform::from_xyz(
                        textbox_scale.x * 38.,
                        textbox_scale.y * 32.,
                        1.,
                    )
                    .with_scale(textbox_scale.extend(1.)),
                    ..Default::default()
                },
                TextBox {
                    text_transform: Transform::from_xyz(
                        textbox_scale.x * 82.,
                        textbox_scale.y * 52.,
                        1.,
                    )
                    .with_scale(textbox_scale.extend(1.) / 4.),
                    text_bounds: TextBounds::from(Vec2::new(800., 200.)),
                    text_anchor: Some(Anchor::TopLeft),
                    font_size: 48.,
                    font: Some(font),
                },
                Transform::from_xyz(-window_resolution.x / 2., -window_resolution.y / 2., 0.),
            ))
            .with_child((
                Sprite {
                    image: textbox_sprite,
                    anchor: Anchor::BottomLeft,
                    ..Default::default()
                },
                Transform::from_scale(textbox_scale.extend(1.)).with_translation(Vec3::NEG_Z),
            ))
            .with_child((
                Sprite {
                    image: continue_sprite,
                    anchor: Anchor::BottomLeft,
                    ..Default::default()
                },
                Transform::from_scale(textbox_scale.extend(1.)).with_translation(Vec3::Z),
                Continue,
                Visibility::Hidden,
            ));
    });
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

// pub fn traditional_textbox(entity: Entity, asset_server: &AssetServer, commands: &mut Commands) {
//     let size = Vec2::new(48. * 8., 45. * 3.);
//     let offset = Vec2::new(20., -20.);
//
//     commands
//         .entity(entity)
//         .insert((
//             TextBox {
//                 text_offset: offset,
//                 text_bounds: TextBounds::from(size - offset * 2.),
//                 text_anchor: Some(Anchor::TopLeft),
//                 font_size: 16.,
//                 font: Some(asset_server.load("textbox/joystix.otf")),
//             },
//             // SfxChar::from_source(asset_server.load("characters/izzy/girl.mp3")),
//             SfxWord::default(),
//             Transform::from_xyz(-400., -150., 0.).with_scale(Vec3::splat(2.)),
//         ))
//         .with_child((Sprite {
//             image: asset_server.load("textbox/textbox.png"),
//             anchor: Anchor::TopLeft,
//             custom_size: Some(size),
//             image_mode: SpriteImageMode::Sliced(TextureSlicer {
//                 center_scale_mode: SliceScaleMode::Tile { stretch_value: 1. },
//                 sides_scale_mode: SliceScaleMode::Tile { stretch_value: 1. },
//                 border: BorderRect::square(16.),
//                 ..Default::default()
//             }),
//             ..Default::default()
//         },))
//         .with_child((
//             Sprite {
//                 image: asset_server.load("textbox/collision_mask.png"),
//                 anchor: Anchor::TopLeft,
//                 ..Default::default()
//             },
//             Transform::from_translation(Vec3::default().with_z(100.)),
//             Continue,
//             Visibility::Hidden,
//         ));
// }
//
// pub fn fade_textbox(entity: Entity, asset_server: &AssetServer, commands: &mut Commands) {
//     commands
//         .entity(entity)
//         .insert((
//             TextBox {
//                 text_offset: Vec2::new(400., 150.),
//                 text_bounds: TextBounds::from(Vec2::new(700., 100.)),
//                 text_anchor: Some(Anchor::TopLeft),
//                 font_size: 28.,
//                 font: Some(asset_server.load("textbox/joystix.otf")),
//             },
//             SfxWord::default(),
//             Transform::from_xyz(-WINDOW_WIDTH / 2., -WINDOW_HEIGHT / 2., 0.),
//         ))
//         .with_child((
//             Sprite {
//                 image: asset_server.load("textbox/black.png"),
//                 anchor: Anchor::BottomLeft,
//                 ..Default::default()
//             },
//             Transform::from_xyz(0., 0., -1.),
//         ))
//         .with_child((
//             Sprite {
//                 image: asset_server.load("textbox/collision_mask.png"),
//                 anchor: Anchor::BottomLeft,
//                 ..Default::default()
//             },
//             Transform::from_xyz(0., 0., 100.),
//             Continue,
//             Visibility::Hidden,
//         ));
// }
