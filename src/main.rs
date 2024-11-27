use bevy::{prelude::*, sprite::Anchor};
use dialogue::fragment::*;
use dialogue_box::TypeWriterState;
use macros::tokens;

mod dialogue;
mod dialogue_box;

fn main() {
    App::default()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((dialogue::DialoguePlugin, dialogue_box::DialogueBoxPlugin))
        .add_systems(Startup, scene)
        .add_systems(Update, bevy_bits::close_on_escape)
        .run();
}

fn spawn_box<F>(
    fragment: F,
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlasLayout>,
) where
    F: IntoFragment<bevy_bits::DialogueBoxToken>,
{
    let dialogue_box = dialogue_box::DialogueBoxBundle {
        atlas: dialogue_box::DialogueBoxAtlas::new(
            asset_server,
            texture_atlases,
            "Scalable txt screen x1.png",
            UVec2::new(16, 16),
        ),
        dimensions: dialogue_box::DialogueBoxDimensions::new(20, 4),
        spatial: SpatialBundle::from_transform(
            Transform::default()
                .with_scale(Vec3::new(3.0, 3.0, 1.0))
                .with_translation(Vec3::new(-500.0, 0.0, 0.0)),
        ),
        ..Default::default()
    };

    let box_entity = commands.spawn_empty().id();
    fragment
        .once()
        .on_start(dialogue_box::spawn_dialogue_box(
            box_entity,
            dialogue_box::TypeWriterBundle {
                font: dialogue_box::DialogueBoxFont {
                    font_size: 32.0,
                    default_color: bevy::color::Color::WHITE,
                    font: asset_server.load("joystix monospace.otf"),
                },
                state: TypeWriterState::new(20.0),
                text_anchor: Anchor::TopLeft,
                spatial: SpatialBundle::from_transform(Transform::default().with_scale(Vec3::new(
                    1.0 / 3.0,
                    1.0 / 3.0,
                    1.0,
                ))),
                ..Default::default()
            },
            dialogue_box,
        ))
        .on_end(dialogue_box::despawn_dialogue_box(box_entity))
        .map_event(move |event| dialogue_box::DialogueBoxEvent {
            event: event.clone(),
            entity: box_entity,
        })
        .spawn_fragment::<bevy_bits::DialogueBoxToken>(commands);
}

fn inner_seq() -> impl IntoFragment<bevy_bits::DialogueBoxToken> {
    (tokens!("Hello..."), tokens!("[15](speed)..."))
}

fn scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    // TODO: clear tokens are implicit for strings
    let fragment = (
        inner_seq(),
        tokens!("[20](speed)What are you looking for?"),
        tokens!("[15](speed)D-did you... [1.0](pause)I mean, [0.5](pause)are you a..."),
        tokens!("[20](speed)Is something wrong?"),
        tokens!("Are you... talking?"),
        tokens!("Well, are you?"),
        tokens!("[12](speed)But you're a [0.25](pause)[20](speed)[FLOWER](wave)!"),
        tokens!("Oh, I guess so..."),
    );

    spawn_box(fragment, &mut commands, &asset_server, &mut texture_atlases);
    commands.spawn(Camera2dBundle::default());
}
