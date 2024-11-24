use bevy::prelude::*;
use dialogue::{DialogStep, EvaluatedDialogue};
use fragment::{DialogueEvent, ErasedFragment};

mod dialogue;
mod evaluate;
mod fragment;

// This is just an example.
#[derive(Debug, Resource)]
enum SceneState {
    Start,
    End,
}

fn scene(world: &mut World) {
    use fragment::*;

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

// sometehing like this
fn handle_fragments(
    mut fragments: Query<&mut ErasedFragment>,
    mut writer: EventWriter<DialogueEvent>,
    mut evaluated_dialogue: ResMut<EvaluatedDialogue>,
    mut commands: Commands,
) {
    let mut evaluations = evaluated_dialogue.evaluations.drain().collect::<Vec<_>>();
    evaluations.sort_by_key(|(_, eval)| eval.count);
    if let Some(hash) = evaluations
        .iter()
        .find_map(|(hash, eval)| eval.result.then_some(hash))
    {
        for mut fragment in fragments.iter_mut() {
            fragment.0.as_mut().emit(*hash, &mut writer, &mut commands);
        }
    }

    evaluated_dialogue.clear();
}

fn main() {
    App::default()
        .add_plugins(DefaultPlugins)
        .insert_resource(DialogStep(0))
        .insert_resource(dialogue::EvaluatedDialogue::default())
        .add_systems(Update, bevy_bits::close_on_escape)
        .add_plugins(dialogue::IntroScene::new(|| true))
        .add_event::<DialogueEvent>()
        .add_systems(PostUpdate, handle_fragments)
        .run();
}
