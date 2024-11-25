use bevy::prelude::*;
use dialogue_box::WhichBox;

mod dialogue;
mod dialogue_box;
mod dialogue_parser;

// This is just an example.
#[derive(Debug, Resource, PartialEq, Eq)]
enum SceneState {
    None,
    Start,
    End,
}

fn scene(world: &mut World) {
    use dialogue::fragment::*;

    let box_id = dialogue_box::DialogueBoxId::random();
    #[allow(clippy::redundant_closure)]
    let fragment = sequence((
        "Hello, [world!](wave)"
            .on_trigger(|mut state: ResMut<SceneState>| {
                *state = SceneState::Start;
            })
            .on_trigger(dialogue_box::show_dialogue_box(
                box_id,
                Transform::default().with_scale(Vec3::new(3.0, 3.0, 1.0)),
                dialogue_box::DialogueBoxDimensions::new(5, 2),
            ))
            .once(),
        dynamic(|state: Res<SceneState>| format!(r#"The scene state is "{:?}"!"#, *state)),
        "Lorem ipsum".on_trigger(|mut state: ResMut<SceneState>| *state = SceneState::End),
        dynamic(|state: Res<SceneState>| format!(r#"And now the scene state is "{:?}"!"#, *state)),
        "Dollor".on_trigger(dialogue_box::hide_dialogue_box(box_id)),
    ))
    .bind(move |id| WhichBox(id, box_id))
    .into_fragment(world);

    let font = world.load_asset("joystix monospace.otf");

    world.commands().spawn((Text2dBundle {
        text: Text::from_sections([
            TextSection::new(
                "Hello, ",
                TextStyle {
                    font: font.clone(),
                    font_size: 32.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::new(
                "World!",
                TextStyle {
                    font: font.clone(),
                    font_size: 32.0,
                    color: Color::BLACK,
                },
            ),
        ]),
        transform: Transform::default().with_translation(Vec3::splat(-200.0)),
        ..Default::default()
    },));

    world.spawn(ErasedFragment(fragment.boxed()));
    world.spawn(Camera2dBundle::default());
}

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
        .insert_resource(SceneState::None)
        .add_systems(Startup, scene)
        .add_systems(Update, bevy_bits::close_on_escape)
        .run();
}
