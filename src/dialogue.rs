use bevy::{app::MainScheduleOrder, ecs::schedule::ScheduleLabel, prelude::*};
use evaluate::DialogueStates;
use rand::Rng;

pub mod evaluate;
pub mod fragment;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Component)]
pub struct DialogueId(u64);

impl DialogueId {
    pub fn random() -> Self {
        Self(rand::thread_rng().gen())
    }
}

/// The full path of IDs for a particular event.
///
/// The path cannot be constructed without at least one
/// ID, so the path can always produce a leaf ID.
///
/// Since fragments are organized in a tree structure,
/// this path provides information for the entire branch.
#[derive(Debug, Clone, Component)]
pub struct IdPath(Vec<DialogueId>);

impl AsRef<[DialogueId]> for IdPath {
    fn as_ref(&self) -> &[DialogueId] {
        &self.0
    }
}

impl core::ops::Deref for IdPath {
    type Target = [DialogueId];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IdPath {
    pub fn new(path: Vec<DialogueId>) -> Self {
        assert!(!path.is_empty(), "An ID path must have at least one node.");

        Self(path)
    }

    pub fn leaf(&self) -> &DialogueId {
        self.first().unwrap()
    }

    pub fn end(&self) -> DialogueEndEvent {
        DialogueEndEvent {
            id_path: self.clone(),
        }
    }
}

#[derive(Debug, Event, Clone)]
pub struct DialogueEvent {
    pub dialogue: String,
    pub id_path: IdPath,
}

#[allow(unused)]
impl DialogueEvent {
    pub fn end(&self) -> DialogueEndEvent {
        DialogueEndEvent {
            id_path: self.id_path.clone(),
        }
    }
}

#[derive(Debug, Event)]
pub struct DialogueEndEvent {
    pub id_path: IdPath,
}

pub struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(FragmentUpdate);
        app.world_mut()
            .resource_mut::<MainScheduleOrder>()
            .insert_before(PreUpdate, FragmentUpdate);

        app.insert_resource(evaluate::EvaluatedDialogue::default())
            .insert_resource(evaluate::DialogueStates::default())
            .add_event::<DialogueEvent>()
            .add_event::<DialogueEndEvent>()
            .add_systems(
                FragmentUpdate,
                (
                    (
                        fragment::update_sequence_items,
                        fragment::update_limit_items,
                    ),
                    handle_fragments,
                    watch_events,
                )
                    .chain(),
            );
    }
}

/// Schedule to emit events from fragments to ensure visibility in a bevy update schedules.
#[derive(ScheduleLabel, Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct FragmentUpdate;

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
            fragment
                .0
                .as_mut()
                .emit(*hash, None, &mut writer, &mut commands);
        }
    }

    evaluated_dialogue.clear();
}

pub fn watch_events(
    mut start: EventReader<DialogueEvent>,
    mut end: EventReader<DialogueEndEvent>,
    mut state: ResMut<DialogueStates>,
) {
    for start in start.read() {
        let entry = state.state.entry(*start.id_path.leaf()).or_default();

        entry.triggered += 1;
        entry.active = true;
    }

    for end in end.read() {
        let entry = state.state.entry(*end.id_path.leaf()).or_default();

        entry.active = false;
    }
}
