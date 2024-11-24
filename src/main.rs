use bevy::prelude::*;

mod dialogue;

// This is just an example.
#[derive(Debug, Resource)]
enum SceneState {
    Start,
    End,
}

fn scene(world: &mut World) {
    use dialogue::fragment::*;

    // NOTE: API example.
    //
    // This doesn't actually do anything yet.
    // I'm not sure if taking `&mut World` is the best,
    // but it does allow `IntoFragment` to schedule arbitrary
    // systems, and we can't really do the same with a plugin
    // since it takes `&self`.
    let fragment = sequence((
        "Hello, world!".on_trigger(|mut state: ResMut<SceneState>| *state = SceneState::Start),
        dynamic(|state: Res<SceneState>| format!("The scene state is {:#?}!", state)),
        "Lorem ipsum",
        "Dolor".on_trigger(|mut state: ResMut<SceneState>| *state = SceneState::End),
    ))
    .into_fragment(world)
    .boxed();

    world.spawn(ErasedFragment(fragment));
}

// fn watch_events(reader: EventReader<DialogueEvent>,)
//     mut evaluated_dialogue: ResMut<EvaluatedDialogue>,
// )

fn main() {
    App::default()
        .add_plugins(DefaultPlugins)
        .add_plugins(dialogue::DialoguePlugin)
        .add_systems(Update, bevy_bits::close_on_escape)
        .run();
}
