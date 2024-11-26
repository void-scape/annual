use bevy::prelude::*;
use dialogue_box::DialogueBoxEvent;
use macros::tokens;
use text::*;

mod dialogue;
mod dialogue_box;
mod dialogue_parser;
mod text;

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
        // .add_systems(Startup, test)
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

    let val = tokens!("Hello, World! My name is [Nic](red). How are [you](wave) doing?");
    println!("{val:#?}");

    let box_id = dialogue_box::DialogueBoxId::random();
    (
        "[10.0](speed)Absence of light. [15.0](speed)[2.0](pause)Notions of shapes[0.5](pause) - both big and small[0.25](pause), dense and fluid.[0.5](pause) \
         There is no separation of self and ship. [1.0](pause)[10.0](speed)Time itself slips through your [5.0](speed)\
         [inanimate structure](red)..."
            .on_visit(|mut state: ResMut<SceneState>| {
                *state = SceneState::Start;
            }),
        dynamic(|state: Res<SceneState>| format!(r#"The scene state is "{:?}"!"#, *state)),
        "Lorem ipsum".on_visit(|mut state: ResMut<SceneState>| *state = SceneState::End),
        dynamic(|state: Res<SceneState>| format!(r#"And now the scene state is "{:?}"!"#, *state)),
        "Dolor",
    )
        .once()
        .map_event(move |event| DialogueBoxEvent(event.clone(), box_id))
        .on_start(dialogue_box::show_dialogue_box(
            box_id,
            Transform::default()
                .with_scale(Vec3::new(3.0, 3.0, 1.0))
                .with_translation(Vec3::new(-500.0, 0.0, 0.0)),
            dialogue_box::DialogueBoxDimensions::new(20, 4),
        ))
        .on_end(dialogue_box::hide_dialogue_box(box_id))
        .spawn(&mut commands);

    commands.spawn(Camera2dBundle::default());
}
