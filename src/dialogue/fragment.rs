use crate::dialogue::{FragmentEvent, FragmentId, FragmentUpdate};
use bevy::{ecs::event::EventRegistry, prelude::*};
use std::marker::PhantomData;

mod delay;
mod dynamic;
mod eval;
mod hooks;
mod leaf;
mod limit;
mod mapped;
mod primitives;
mod sequence;

pub use delay::Delay;
pub use eval::Evaluated;
pub use hooks::{OnEnd, OnStart, OnVisit};
pub use leaf::Leaf;
pub use limit::Limit;
pub use mapped::Mapped;

pub(crate) use delay::manage_delay;
pub(crate) use limit::update_limit_items;
pub(crate) use sequence::update_sequence_items;

use super::{evaluate::FragmentStates, EvaluateSet};

/// A wrapper for typestate management.
pub struct Unregistered<T>(T);

/// A type-erased fragment component.
#[derive(Component)]
pub struct ErasedFragment<Context, Data>(pub Box<dyn Fragment<Context, Data> + Send + Sync>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Start {
    Entered,
    Visited,
    Unvisited,
}

impl Start {
    pub fn entered(&self) -> bool {
        matches!(self, Self::Entered)
    }

    /// Either Visited or Entered
    pub fn visited(&self) -> bool {
        matches!(self, Self::Visited | Self::Entered)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum End {
    Exited,
    Visited,
    Unvisited,
}

impl End {
    pub fn exited(&self) -> bool {
        matches!(self, Self::Exited)
    }

    /// Either Visited or Exited
    pub fn visited(&self) -> bool {
        matches!(self, Self::Visited | Self::Exited)
    }
}

#[derive(Debug, Clone)]
pub struct FragmentNode {
    pub id: FragmentId,
    pub children: Vec<FragmentNode>,
}

impl FragmentNode {
    pub fn new(id: FragmentId, children: Vec<FragmentNode>) -> Self {
        Self { id, children }
    }

    pub fn leaf(id: FragmentId) -> Self {
        Self {
            id,
            children: Vec::new(),
        }
    }

    pub fn push(&mut self, node: FragmentNode) {
        self.children.push(node);
    }

    /// Return all the leaves starting from this node.
    ///
    /// If this node has no children, its ID is returned.
    /// Otherwise, we descend this node's children to find all the leaves.
    ///
    /// The traversal is depth-first.
    pub fn leaves(&self) -> Vec<FragmentId> {
        let mut leaves = Vec::new();
        self.leaves_recursive(&mut leaves);

        leaves
    }

    fn leaves_recursive(&self, leaves: &mut Vec<FragmentId>) {
        if self.children.is_empty() {
            leaves.push(self.id);
        } else {
            for child in self.children.iter() {
                child.leaves_recursive(leaves);
            }
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct FragmentTree {
    pub tree: FragmentNode,
    pub fragment: Entity,
}

#[derive(Component)]
pub struct FragmentContext<T>(T);

pub trait Threaded: Send + Sync + 'static {}

impl<T> Threaded for T where T: Send + Sync + 'static {}

/// A dialogue fragment.
///
/// Fragments represent nodes in a dialogue tree. Leaf nodes
/// are simply text with associated IDs, but further up the
/// tree you can have organizational nodes like [Sequence]
/// or behavioral nodes like [Trigger] and [Evaluated].
///
/// This is intentionally type-eraseable. We can
/// store top-level fragments as `Box<dyn Fragment>` in entities and
/// call their `emit` method any time a [FragmentId] is selected.
pub trait Fragment<Context, Data: Threaded> {
    /// React to a leaf node being selected.
    fn start(
        &mut self,
        ctx: &Context,
        id: FragmentId,
        state: &mut FragmentStates,
        writer: &mut EventWriter<FragmentEvent<Data>>,
        commands: &mut Commands,
    ) -> Start;

    /// React to a leaf node being selected.
    fn end(
        &mut self,
        ctx: &Context,
        id: FragmentId,
        state: &mut FragmentStates,
        commands: &mut Commands,
    ) -> End;

    /// This fragment's ID.
    ///
    /// This should be stable over the lifetime of the application.
    fn id(&self) -> &FragmentId;
}

/// A convenience trait for type-erasing fragments.
pub trait BoxedFragment<Context, Data> {
    fn boxed(self) -> Box<dyn Fragment<Context, Data> + Send + Sync>;
}

impl<T, Context, Data> BoxedFragment<Context, Data> for T
where
    T: Fragment<Context, Data> + Send + Sync + 'static,
    Data: Threaded,
{
    fn boxed(self) -> Box<dyn Fragment<Context, Data> + Send + Sync> {
        Box::new(self)
    }
}

/// Spawn a fragment with its associated ID tree.
pub fn spawn_fragment<Context, Data>(
    fragment: impl Fragment<Context, Data> + Send + Sync + 'static,
    tree: FragmentNode,
    commands: &mut Commands,
) where
    Data: Threaded,
    Context: Threaded,
{
    commands.add(move |world: &mut World| {
        if !world.contains_resource::<Events<FragmentEvent<Data>>>() {
            EventRegistry::register_event::<FragmentEvent<Data>>(world);
        }

        // TODO: make sure these can't overlap
        let mut schedules = world.resource_mut::<Schedules>();
        schedules.add_systems(
            FragmentUpdate,
            (
                evaluated_fragments::<Context, Data>,
                watch_events::<Context, Data>,
            )
                .chain()
                .after(EvaluateSet),
        );
    });

    let associated_frag = commands.spawn(ErasedFragment(fragment.boxed())).id();
    commands.spawn(FragmentTree {
        tree,
        fragment: associated_frag,
    });
}

pub trait SpawnFragment: Sized {
    /// A convenience method for spawning fragments.
    ///
    /// Equivalent to
    /// ```
    ///
    /// # fn spawn(context: u32, mut commands: Commands) {
    /// let (fragment, tree) = self.into_fragment(context, &mut commands);
    /// spawn_fragment(fragment, tree, &mut commands);
    /// # }
    /// ```
    fn spawn_fragment<Context, Data>(self, context: Context, commands: &mut Commands)
    where
        Data: Threaded,
        Context: Threaded,
        Self: IntoFragment<Context, Data>,
        <Self as IntoFragment<Context, Data>>::Fragment:
            Fragment<Context, Data> + Send + Sync + 'static;
}

impl<T> SpawnFragment for T {
    fn spawn_fragment<Context, Data>(self, context: Context, commands: &mut Commands)
    where
        Data: Threaded,
        Context: Threaded,
        Self: IntoFragment<Context, Data>,
        <Self as IntoFragment<Context, Data>>::Fragment:
            Fragment<Context, Data> + Send + Sync + 'static,
    {
        let (fragment, tree) = self.into_fragment(&context, commands);
        spawn_fragment(fragment, tree, commands);
    }
}

pub trait IntoFragment<Context, Data: Threaded> {
    type Fragment: Fragment<Context, Data> + Send + Sync + 'static;

    fn into_fragment(
        self,
        context: &Context,
        commands: &mut Commands,
    ) -> (Self::Fragment, FragmentNode);
}

impl<T> FragmentExt for T {}

pub trait FragmentExt: Sized {
    /// Run a system any time this fragment is visited.
    fn on_visit<S, M>(self, system: S) -> OnVisit<Self, S::System>
    where
        S: IntoSystem<(), (), M>,
    {
        OnVisit {
            fragment: self,
            on_trigger: IntoSystem::into_system(system),
        }
    }

    /// Run a system when this fragment is initially triggered.
    fn on_start<S, M>(self, system: S) -> OnStart<Self, S::System>
    where
        S: IntoSystem<(), (), M>,
    {
        OnStart {
            fragment: self,
            on_trigger: IntoSystem::into_system(system),
        }
    }

    /// Run a system when this fragment is considered complete.
    fn on_end<S, M>(self, system: S) -> OnEnd<Self, S::System>
    where
        S: IntoSystem<(), (), M>,
    {
        OnEnd {
            fragment: self,
            on_trigger: IntoSystem::into_system(system),
        }
    }

    /// Run a system when this fragment is considered complete after the given delay.
    fn delay<S, M>(self, duration: std::time::Duration, system: S) -> Delay<Self, S::System>
    where
        S: IntoSystem<(), (), M>,
    {
        Delay::new(self, duration, IntoSystem::into_system(system))
    }

    /// Map a dialogue event.
    fn map_event<Data, S, M>(self, event: S) -> Mapped<Self, S, M, Data>
    where
        M: Event,
        S: FnMut(&FragmentEvent<Data>) -> M + Send + Sync + 'static,
    {
        Mapped {
            fragment: self,
            event,
            _marker: PhantomData,
        }
    }
    //
    /// Wrap this fragment in an evaluation.
    fn eval<S, M, O>(self, system: S) -> Evaluated<Self, Unregistered<S::System>, O>
    where
        S: IntoSystem<(), O, M>,
    {
        Evaluated {
            fragment: self,
            evaluation: Unregistered(IntoSystem::into_system(system)),
            _marker: PhantomData,
        }
    }

    /// Limit this fragment to `n` triggers.
    fn limit(self, n: usize) -> Limit<Self> {
        Limit::new(self, n)
    }

    /// Set this fragment's limit to 1.
    fn once(self) -> Limit<Self> {
        self.limit(1)
    }

    /// Set a resource of type `T` with the provided value on the start of a fragment.
    ///
    /// This is similar to:
    /// ```
    /// #[derive(Resource)]
    /// struct Resource(usize);
    ///
    /// "fragment".on_start(|mut resource: ResMut<Resource>| *resource = Resource(1));
    /// ```
    /// Except the resource is automatically inserted if it doesn't already exist.
    fn set_resource<T>(self, value: T) -> OnStart<Self, impl System<In = (), Out = ()>>
    where
        T: Resource + Clone,
    {
        let system = move |world: &mut World| {
            if !world.contains_resource::<T>() {
                world.insert_resource(value.clone());
            } else {
                world.set_resource(value.clone());
            }
        };

        OnStart {
            fragment: self,
            on_trigger: IntoSystem::into_system(system),
        }
    }
}

fn descend_tree(
    node: &FragmentNode,
    fragment: Entity,
    evaluations: &mut super::evaluate::EvaluatedFragments,
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
fn evaluated_fragments<Context, Data: Threaded>(
    mut fragments: Query<(
        &mut ErasedFragment<Context, Data>,
        &FragmentContext<Context>,
    )>,
    trees: Query<&FragmentTree>,
    mut writer: EventWriter<FragmentEvent<Data>>,
    mut evaluated_dialogue: ResMut<super::evaluate::EvaluatedFragments>,
    mut state: ResMut<FragmentStates>,
    mut commands: Commands,
) where
    Context: Sync + Send + 'static,
{
    // traverse trees to build up full evaluatinos
    let mut leaves = Vec::new();
    for FragmentTree { tree, fragment } in trees.iter() {
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
        if let Ok((mut fragment, ctx)) = fragments.get_mut(*fragment) {
            fragment
                .0
                .as_mut()
                .start(&ctx.0, **id, &mut state, &mut writer, &mut commands);
        }
    }

    evaluated_dialogue.clear();
}

fn watch_events<Context, Data: Threaded>(
    mut fragments: Query<(
        &mut ErasedFragment<Context, Data>,
        &FragmentContext<Context>,
    )>,
    mut end: EventReader<super::FragmentEndEvent>,
    mut state: ResMut<FragmentStates>,
    mut commands: Commands,
) where
    Context: Sync + Send + 'static,
{
    for end in end.read() {
        for (mut fragment, ctx) in fragments.iter_mut() {
            fragment
                .0
                .as_mut()
                .end(&ctx.0, end.id, &mut state, &mut commands);
        }
    }
}
