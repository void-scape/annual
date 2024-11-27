use bevy::{prelude::*, sprite::Anchor};
use macros::tokens;

mod dialogue;
mod dialogue_box;

fn main() {
    App::default()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((
            dialogue::DialoguePlugin,
            dialogue_box::DialogueBoxPlugin::new(
                "joystix monospace.otf",
                "Scalable txt screen x1.png",
                UVec2::splat(16),
            ),
        ))
        .add_systems(Startup, scene)
        .add_systems(Update, bevy_bits::close_on_escape)
        .run();
}

fn scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    use dialogue::fragment::*;

    let dialogue_box = dialogue_box::DialogueBoxBundle {
        atlas: dialogue_box::DialogueBoxAtlas::new(
            &asset_server,
            &mut texture_atlases,
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
    (
        // TODO: clear token implicit for &'static str and strings
        tokens!("Hello..."),
        tokens!("[15](speed)..."),
        tokens!("[20](speed)What are you looking for?"),
        tokens!("[15](speed)D-did you... [1.0](pause)I mean, [0.5](pause)are you a..."),
        tokens!("[20](speed)Is something wrong?"),
        tokens!("Are you... talking?"),
        tokens!("Well, are you?"),
        tokens!("[15](speed)But you're a [20](speed)[FLOWER](wave)!"),
        tokens!("Oh, I guess so..."),
    )
        .once()
        .on_start(dialogue_box::spawn_dialogue_box(
            box_entity,
            dialogue_box::TypeWriterBundle {
                state: dialogue_box::TypeWriterState::new(20.),
                text_anchor: Anchor::TopLeft,
                font: dialogue_box::DialogueBoxFont {
                    font: asset_server.load("joystix monospace.otf"),
                    font_size: 45.0,
                    default_color: bevy::color::Color::WHITE,
                },
                spatial: SpatialBundle::from_transform(Transform::default().with_scale(Vec3::new(
                    1.0 / 3.0,
                    1.0 / 3.0,
                    1.0,
                ))),
                text_2d_bounds: dialogue_box.text_bounds(),
                ..Default::default()
            },
            dialogue_box,
        ))
        .on_end(dialogue_box::despawn_dialogue_box(box_entity))
        .map_event(
            move |event: &dialogue::FragmentEvent<bevy_bits::DialogueBoxToken>| {
                dialogue_box::DialogueBoxEvent {
                    event: event.clone(),
                    entity: box_entity,
                }
            },
        )
        .spawn_fragment::<bevy_bits::DialogueBoxToken>(&mut commands);

    commands.spawn(Camera2dBundle::default());
}
