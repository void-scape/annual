use bevy::prelude::*;

mod dialogue;
mod dialogue_box;
mod dialogue_parser;

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

// This is just an example.
#[derive(Debug, Resource, PartialEq, Eq)]
enum SceneState {
    None,
    Start,
    End,
}

fn scene(mut commands: Commands) {
    use dialogue::fragment::*;

    let box_id = dialogue_box::DialogueBoxId::random();
    (
        "Hello, world!".on_visit(|mut state: ResMut<SceneState>| {
            *state = SceneState::Start;
        }),
        dynamic(|state: Res<SceneState>| format!(r#"The scene state is "{:?}"!"#, *state)),
        "Lorem ipsum".on_visit(|mut state: ResMut<SceneState>| *state = SceneState::End),
        dynamic(|state: Res<SceneState>| format!(r#"And now the scene state is "{:?}"!"#, *state)),
        "Dolor",
    )
        .once()
        .on_start(dialogue_box::show_dialogue_box(
            box_id,
            Transform::default().with_scale(Vec3::new(3.0, 3.0, 1.0)),
            dialogue_box::DialogueBoxDimensions::new(5, 2),
        ))
        .on_end(dialogue_box::hide_dialogue_box(box_id))
        .spawn(&mut commands);

    commands.spawn(Camera2dBundle::default());
}
