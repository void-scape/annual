use bevy::{app::MainScheduleOrder, ecs::schedule::ScheduleLabel, prelude::*};
use evaluate::DialogueStates;
use fragment::DialogueTree;
use rand::Rng;

pub mod evaluate;
pub mod fragment;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Component)]
pub struct DialogueId(u64);

impl DialogueId {
    pub fn random() -> Self {
        Self(rand::thread_rng().gen())
    }

    pub fn end(&self) -> DialogueEndEvent {
        DialogueEndEvent { id: *self }
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

#[allow(dead_code)]
impl IdPath {
    pub fn new(path: Vec<DialogueId>) -> Self {
        assert!(!path.is_empty(), "An ID path must have at least one node.");

        Self(path)
    }

    pub fn leaf(&self) -> &DialogueId {
        self.first().unwrap()
    }
}

#[derive(Debug, Event, Clone)]
pub struct DialogueEvent {
    pub dialogue: String,
    pub id: DialogueId,
}

#[allow(unused)]
impl DialogueEvent {
    pub fn end(&self) -> DialogueEndEvent {
        DialogueEndEvent { id: self.id }
    }
}

#[derive(Debug, Event)]
pub struct DialogueEndEvent {
    pub id: DialogueId,
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
                    evaluated_fragments,
                    watch_events,
                )
                    .chain(),
            );
    }
}

/// Schedule to emit events from fragments to ensure visibility in a bevy update schedules.
#[derive(ScheduleLabel, Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct FragmentUpdate;

fn descend_tree(
    node: &fragment::FragmentNode,
    fragment: Entity,
    evaluations: &mut evaluate::EvaluatedDialogue,
    leaves: &mut Vec<(DialogueId, Entity)>,
) {
    if node.children.is_empty() {
        leaves.push((node.id, fragment));
    } else {
        for child in node.children.iter() {
            // push the parent eval, if any
            if let Some(eval) = evaluations.evaluations.get(&node.id).copied() {
                evaluations.insert(child.id, eval);
            }

            if evaluations.is_candidate(child.id) {
                descend_tree(child, fragment, evaluations, leaves);
            }
        }
    }
}

// sometehing like this
fn evaluated_fragments(
    mut fragments: Query<&mut fragment::ErasedFragment>,
    trees: Query<&DialogueTree>,
    mut writer: EventWriter<DialogueEvent>,
    mut evaluated_dialogue: ResMut<evaluate::EvaluatedDialogue>,
    mut state: ResMut<DialogueStates>,
    mut commands: Commands,
) {
    // traverse trees to build up full evaluatinos
    let mut leaves = Vec::new();
    for DialogueTree { tree, fragment } in trees.iter() {
        // info!("tree: {tree:#?}");
        descend_tree(tree, *fragment, &mut evaluated_dialogue, &mut leaves);
    }

    // info!("leaves: {leaves:#?}");
    // info!("evals: {evaluated_dialogue:#?}");

    let mut evaluations: Vec<_> = leaves
        .iter()
        .flat_map(|(id, frag)| {
            evaluated_dialogue
                .evaluations
                .get(id)
                .map(|e| (id, *frag, e))
        })
        .filter(|(id, _, e)| e.result && !state.is_active(**id))
        .collect();
    evaluations.sort_by_key(|(_, _, e)| e.count);

    if let Some((id, fragment, _)) = evaluations.first() {
        if let Ok(mut fragment) = fragments.get_mut(*fragment) {
            fragment
                .0
                .as_mut()
                .start(**id, &mut state, &mut writer, &mut commands);
        }
    }

    evaluated_dialogue.clear();
}

fn watch_events(
    mut fragments: Query<&mut fragment::ErasedFragment>,
    mut end: EventReader<DialogueEndEvent>,
    mut state: ResMut<DialogueStates>,
    mut commands: Commands,
) {
    for end in end.read() {
        for mut fragment in fragments.iter_mut() {
            fragment.0.as_mut().end(end.id, &mut state, &mut commands);
        }
    }
}
