use bevy::prelude::*;
use rand::Rng;

pub mod evaluate;
pub mod fragment;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DialogueId(u64);

impl DialogueId {
    pub fn random() -> Self {
        Self(rand::thread_rng().gen())
    }
}

#[derive(Debug, Event)]
pub struct DialogueEvent {
    pub dialogue: String,
    pub id: DialogueId,
}

pub struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(evaluate::EvaluatedDialogue::default())
            .add_event::<DialogueEvent>()
            .add_systems(PostUpdate, handle_fragments);
    }
}

// sometehing like this
pub fn handle_fragments(
    mut fragments: Query<&mut fragment::ErasedFragment>,
    mut writer: EventWriter<DialogueEvent>,
    mut evaluated_dialogue: ResMut<evaluate::EvaluatedDialogue>,
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
