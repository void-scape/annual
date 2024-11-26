use bevy::{app::MainScheduleOrder, ecs::schedule::ScheduleLabel, prelude::*};
use evaluate::FragmentStates;
use fragment::{DialogueTree, FragmentData};
use rand::Rng;

pub mod evaluate;
pub mod fragment;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Component)]
pub struct FragmentId(u64);

impl FragmentId {
    pub fn random() -> Self {
        Self(rand::thread_rng().gen())
    }

    pub fn end(&self) -> FragmentEndEvent {
        FragmentEndEvent { id: *self }
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
pub struct IdPath(Vec<FragmentId>);

impl AsRef<[FragmentId]> for IdPath {
    fn as_ref(&self) -> &[FragmentId] {
        &self.0
    }
}

impl core::ops::Deref for IdPath {
    type Target = [FragmentId];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[allow(dead_code)]
impl IdPath {
    pub fn new(path: Vec<FragmentId>) -> Self {
        assert!(!path.is_empty(), "An ID path must have at least one node.");

        Self(path)
    }

    pub fn leaf(&self) -> &FragmentId {
        self.first().unwrap()
    }
}

#[derive(Debug, Event, Clone)]
pub struct FragmentEvent<Data> {
    pub id: FragmentId,
    pub data: Data,
}

#[allow(unused)]
impl<E> FragmentEvent<E> {
    pub fn end(&self) -> FragmentEndEvent {
        FragmentEndEvent { id: self.id }
    }
}

#[derive(Debug, Event)]
pub struct FragmentEndEvent {
    pub id: FragmentId,
}

pub struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(FragmentUpdate);
        app.world_mut()
            .resource_mut::<MainScheduleOrder>()
            .insert_before(PreUpdate, FragmentUpdate);

        app.insert_resource(evaluate::EvaluatedFragments::default())
            .insert_resource(evaluate::FragmentStates::default())
            // .add_event::<FragmentEvent>()
            .add_event::<FragmentEndEvent>()
            .add_systems(
                FragmentUpdate,
                (
                    (
                        fragment::update_sequence_items,
                        fragment::update_limit_items,
                    ),
                    // TODO: add these when a new FragmentEvent is introduced
                    // evaluated_fragments,
                    // watch_events,
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
    evaluations: &mut evaluate::EvaluatedFragments,
    leaves: &mut Vec<(FragmentId, Entity)>,
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

// TODO: update so this is inserted for every unique event type.
fn evaluated_fragments<Data: FragmentData>(
    mut fragments: Query<&mut fragment::ErasedFragment<Data>>,
    trees: Query<&DialogueTree>,
    mut writer: EventWriter<FragmentEvent<Data>>,
    mut evaluated_dialogue: ResMut<evaluate::EvaluatedFragments>,
    mut state: ResMut<FragmentStates>,
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

fn watch_events<Data: FragmentData>(
    mut fragments: Query<&mut fragment::ErasedFragment<Data>>,
    mut end: EventReader<FragmentEndEvent>,
    mut state: ResMut<FragmentStates>,
    mut commands: Commands,
) {
    for end in end.read() {
        for mut fragment in fragments.iter_mut() {
            fragment.0.as_mut().end(end.id, &mut state, &mut commands);
        }
    }
}
