use crate::{HEIGHT, WIDTH};

use self::frags::SectionFrag;
use self::render_layer::RenderLayerPlugin;
use bevy::render::view::RenderLayers;
use bevy::window::WindowResized;
use bevy::{prelude::*, sprite::Anchor, text::TextBounds};
use bevy_pretty_text::text::TypeWriterCommand;
use bevy_pretty_text::type_writer::TypeWriterSets;
use bevy_pretty_text::{prelude::*, type_writer::scroll::OnScrollEnd};
use bevy_sequence::prelude::*;
use render_layer::PropagateRenderLayers;

pub mod frags;
pub mod render_layer;

#[allow(unused)]
pub mod prelude {
    pub use super::frags::{portrait::TextBoxPortrait, sfx::TextBoxSfx, IntoBox, TextBoxContext};
    pub use super::{TextBox, TextBoxPlugin};
}

pub struct TextBoxPlugin;

impl Plugin for TextBoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PrettyTextPlugin, RenderLayerPlugin))
            .add_systems(Startup, init_camera)
            .add_systems(
                Update,
                (
                    frags::portrait::spawn_portrait,
                    frags::portrait::update_portrait,
                    update_continue_visibility,
                    spawn_section_frags,
                    resize_textbox,
                )
                    .chain()
                    .after(TypeWriterSets::Update),
            );
    }
}

fn init_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            // Render after the main camera
            order: 1,
            clear_color: Color::NONE.into(),
            // All cameras must be hdr
            hdr: true,
            ..default()
        },
        TextBox::RENDER_LAYER,
        Msaa::Off,
    ));
}

#[derive(Default, Component)]
#[require(RenderLayers(|| TextBox::RENDER_LAYER), PropagateRenderLayers)]
pub struct TextBox {
    pub text_transform: Transform,
    pub text_bounds: TextBounds,
    pub text_anchor: Option<Anchor>,
    pub font_size: f32,
    pub font: Option<Handle<Font>>,
}

impl TextBox {
    pub const RENDER_LAYER: RenderLayers = RenderLayers::layer(2);

    pub fn text_bundle(&self) -> impl Bundle {
        (
            self.text_bounds,
            self.text_anchor.unwrap_or_default(),
            self.text_transform,
            TextFont {
                font_size: self.font_size,
                font: self.font.clone().unwrap_or_default(),
                ..Default::default()
            },
        )
    }
}

#[derive(Component)]
pub struct Continue;

fn update_continue_visibility(
    textbox_query: Query<&Children, With<TextBox>>,
    mut continue_query: Query<&mut Visibility, With<Continue>>,
    scroll_query: Query<&Scroll, With<AwaitClear>>,
) {
    for children in textbox_query.iter() {
        if children.iter().any(|c| scroll_query.get(*c).is_ok()) {
            for child in children.iter() {
                if let Ok(mut vis) = continue_query.get_mut(*child) {
                    *vis = Visibility::Visible;
                }
            }
        } else {
            for child in children.iter() {
                if let Ok(mut vis) = continue_query.get_mut(*child) {
                    *vis = Visibility::Hidden;
                }
            }
        }
    }
}

// TODO: this leaks memory (SystemId)
fn spawn_section_frags(
    mut commands: Commands,
    mut reader: EventReader<FragmentEvent<SectionFrag>>,
    textbox_query: Query<(&Children, &TextBox)>,
    mut text_query: Query<&mut TypeWriterSection>,
) {
    for event in reader.read() {
        let textbox = event.data.textbox;
        let end = event.end();

        if let Ok((children, tb)) = textbox_query.get(textbox) {
            if children.iter().any(|c| text_query.get(*c).is_ok()) {
                for child in children.iter() {
                    if let Ok(mut section) = text_query.get_mut(*child) {
                        section.join(&event.data.section);

                        let entity = *child;
                        match event.data.section.end {
                            Some(_) => {
                                let on_clear = commands.register_system(
                                move |mut commands: Commands, mut writer: EventWriter<FragmentEndEvent>| {
                                    commands.entity(entity).despawn_recursive();
                                    writer.send(end);
                                },
                            );
                                commands.entity(entity).insert(OnClear(on_clear));
                            }
                            None => {
                                let on_end = commands.register_system(
                                    move |mut writer: EventWriter<FragmentEndEvent>| {
                                        writer.send(end);
                                    },
                                );
                                commands.entity(entity).insert(OnScrollEnd(on_end));
                            }
                        }

                        break;
                    }
                }
            } else {
                let entity = commands.spawn_empty().id();

                let on_clear = commands.register_system(
                    move |mut commands: Commands, mut writer: EventWriter<FragmentEndEvent>| {
                        commands.entity(entity).despawn_recursive();
                        writer.send(end);
                    },
                );

                let id = commands
                    .entity(entity)
                    .insert((
                        event.data.section.clone(),
                        Scroll::default(),
                        OnClear(on_clear),
                        tb.text_bundle(),
                    ))
                    .id();

                if event
                    .data
                    .section
                    .end
                    .is_none_or(|c| TypeWriterCommand::AwaitClear != c)
                {
                    let on_end = commands.register_system(
                        move |mut writer: EventWriter<FragmentEndEvent>| {
                            writer.send(end);
                        },
                    );
                    commands.entity(id).insert(OnScrollEnd(on_end));
                }

                commands.entity(textbox).add_child(id);
            }
        }
    }
}

fn resize_textbox(
    mut resize_events: EventReader<WindowResized>,
    mut textbox_query: Query<&mut Transform, With<TextBox>>,
) {
    for event in resize_events.read() {
        let h_scale = event.width / WIDTH;
        let v_scale = event.height / HEIGHT;
        for mut t in textbox_query.iter_mut() {
            t.scale = Vec2::new(h_scale, v_scale).extend(1.);
        }
    }
}
