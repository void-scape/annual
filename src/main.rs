use bevy::prelude::*;
use dialogue::fragment::*;
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

fn spawn_box<F>(
    fragment: F,
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlasLayout>,
) where
    F: IntoFragment,
    F::Fragment<bevy_bits::DialogueBoxToken>:
        Fragment<bevy_bits::DialogueBoxToken> + Send + Sync + 'static,
{
    let box_entity = commands.spawn_empty().id();
    fragment
        .once()
        .on_start(dialogue_box::spawn_dialogue_box(
            box_entity,
            dialogue_box::DialogueBoxBundle {
                atlas: dialogue_box::DialogueBoxAtlas::new(
                    asset_server,
                    texture_atlases,
                    "Scalable txt screen x1.png",
                    UVec2::new(16, 16),
                ),
                dimensions: dialogue_box::DialogueBoxDimensions::new(20, 4),
                font: dialogue_box::DialogueBoxFont {
                    font: asset_server.load("joystix monospace.otf"),
                    font_size: 32.0,
                    default_color: bevy::color::Color::WHITE,
                },
                spatial: SpatialBundle::from_transform(
                    Transform::default()
                        .with_scale(Vec3::new(3.0, 3.0, 1.0))
                        .with_translation(Vec3::new(-500.0, 0.0, 0.0)),
                ),
            },
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
        .spawn_fragment::<bevy_bits::DialogueBoxToken>(commands);
}

fn scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let fragment = (
        "Hello...",
        tokens!("[1.0](speed)..."),
        "What are you looking for?",
        tokens!("D-did you... [1.0](pause)I mean, [0.5](pause)are you a..."),
        "Is something wrong?",
        "Are you... talking?",
        "Well, are you?",
        tokens!("But you're a [FLOWER](wave)!"),
        "Oh, I guess so...",
    );

    spawn_box(fragment, &mut commands, &asset_server, &mut texture_atlases);

    commands.spawn(Camera2dBundle::default());
}
