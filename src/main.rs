use bevy::prelude::*;
use dialogue::{DialogueEvent, DialogueId};

mod dialogue;
mod dialogue_box;

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

    // ("Lorem", "Ipsum", "Dolor")
    //     .once()
    //     .on_start(dialogue_box::show_dialogue_box(
    //         box_id,
    //         Transform::default().with_scale(Vec3::new(3.0, 3.0, 1.0)),
    //         dialogue_box::DialogueBoxDimensions::new(5, 2),
    //     ))
    //     .on_end(dialogue_box::hide_dialogue_box(box_id))
    //     .spawn(&mut commands);

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

    // spawn(
    //     world,
    //     fragment,
    //     box_id,
    //     Transform::default().with_scale(Vec3::new(3.0, 3.0, 1.0)),
    //     dialogue_box::DialogueBoxDimensions::new(5, 2),
    // );

    commands.spawn(Camera2dBundle::default());
}

fn look_for_finished_event(
    mut reader: EventReader<FinishedEvent>,
    mut events: EventReader<DialogueEvent>,
) {
    let mut found_first = false;
    let mut found_second = false;

    for event in events.read() {
        found_first = true;
        info!("dialogue: {event:?}");
    }

    for event in reader.read() {
        found_second = true;
        info!("finished: {event:?}");
    }

    if found_first && found_second {
        info!("Found both at once!!!")
    }
}

#[derive(Event, Debug, Clone)]
struct FinishedEvent(DialogueId);

fn main() {
    App::default()
        .add_event::<FinishedEvent>()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((
            dialogue::DialoguePlugin,
            dialogue_box::DialogueBoxPlugin::new("Wasted-Vindey.ttf"),
        ))
        .insert_resource(SceneState::None)
        .add_systems(Startup, scene)
        .add_systems(Update, bevy_bits::close_on_escape)
        .add_systems(Update, look_for_finished_event)
        .run();
}
