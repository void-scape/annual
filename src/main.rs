use bevy::prelude::*;
use dialogue::DialogueEndEvent;

mod dialogue;

// This is just an example.
#[derive(Debug, Resource)]
enum SceneState {
    None,
    Start,
    End,
}

fn scene(world: &mut World) {
    use dialogue::fragment::*;

    // API example.
    //
    // I'm not sure if taking `&mut World` is the best,
    // but it does allow `IntoFragment` to schedule arbitrary
    // systems, and we can't really do the same with a plugin
    // since it takes `&self`.
    let fragment = sequence((
        "Hello, world!"
            .on_trigger(|mut state: ResMut<SceneState>| *state = SceneState::Start)
            // this ensures we actually start the sequence
            .once(),
        dynamic(|state: Res<SceneState>| format!(r#"The scene state is "{:?}"!"#, *state)),
        "Lorem ipsum".on_trigger(|mut state: ResMut<SceneState>| *state = SceneState::End),
        dynamic(|state: Res<SceneState>| format!(r#"And now the scene state is "{:?}"!"#, *state)),
        "Dolor",
    ))
    .into_fragment(world);

    world.spawn(ErasedFragment(fragment.boxed()));
}

fn ping_pong(
    mut reader: EventReader<dialogue::DialogueEvent>,
    mut writer: EventWriter<DialogueEndEvent>,
) {
    for event in reader.read() {
        // Dialogue printing
        println!("{}", event.dialogue);
        // Indicating we're done with the dialogue
        writer.send(event.end());
    }
}

fn main() {
    App::default()
        .add_plugins(DefaultPlugins)
        .add_plugins(dialogue::DialoguePlugin)
        .insert_resource(SceneState::None)
        .add_systems(Update, bevy_bits::close_on_escape)
        .add_systems(Startup, scene)
        .add_systems(Update, ping_pong)
        .run();
}
