use crate::dialogue::{FragmentEvent, FragmentId};
use bevy::prelude::*;
use std::marker::PhantomData;

mod dynamic;
mod eval;
mod hooks;
mod limit;
mod mapped;
mod sequence;
mod string;

pub use dynamic::dynamic;
pub use eval::Evaluated;
pub use hooks::{OnEnd, OnStart, OnVisit};
pub use limit::Limit;
pub use mapped::Mapped;

pub(crate) use limit::update_limit_items;
pub(crate) use sequence::update_sequence_items;

use super::evaluate::FragmentStates;

/// A wrapper for typestate management.
pub struct Unregistered<T>(T);

/// A type-erased fragment component.
#[derive(Component)]
pub struct ErasedFragment<Data>(pub Box<dyn Fragment<Data> + Send + Sync>);

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
pub struct DialogueTree {
    pub tree: FragmentNode,
    pub fragment: Entity,
}

pub trait FragmentData: Send + Sync + 'static {}

impl<T> FragmentData for T where T: Send + Sync + 'static {}

/// A dialogue fragment.
///
/// Fragments represent nodes in a dialogue tree. Leaf nodes
/// are simply text with associated IDs, but further up the
/// tree you can have organizational nodes like [Sequence]
/// or behavioral nodes like [Trigger] and [Evaluated].
///
/// This is intentionally type-eraseable. We can
/// store top-level fragments as `Box<dyn Fragment>` in entities and
/// call their `emit` method any time a [DialogueId] is selected.
pub trait Fragment<Data: FragmentData> {
    /// React to a leaf node being selected.
    fn start(
        &mut self,
        id: FragmentId,
        state: &mut FragmentStates,
        writer: &mut EventWriter<FragmentEvent<Data>>,
        commands: &mut Commands,
    ) -> Start;

    /// React to a leaf node being selected.
    fn end(&mut self, id: FragmentId, state: &mut FragmentStates, commands: &mut Commands) -> End;

    /// This fragment's ID.
    ///
    /// This should be stable over the lifetime of the application.
    fn id(&self) -> &FragmentId;
}

/// A convenience trait for type-erasing fragments.
pub trait BoxedFragment<Data> {
    fn boxed(self) -> Box<dyn Fragment<Data> + Send + Sync>;
}

impl<T, Data> BoxedFragment<Data> for T
where
    T: Fragment<Data> + Send + Sync + 'static,
    Data: FragmentData,
{
    fn boxed(self) -> Box<dyn Fragment<Data> + Send + Sync> {
        Box::new(self)
    }
}

// pub trait SpawnFragment: Sized {
//     fn spawn(self, commands: &mut Commands);
// }
//
// impl<T> SpawnFragment for T
// where
//     T: IntoFragment,
//     T::Fragment: Send + Sync + 'static,
// {
//     fn spawn(self, commands: &mut Commands) {
//         let (fragment, tree) = self.into_fragment(commands);
//
//         let associated_frag = commands.spawn(ErasedFragment(fragment.boxed())).id();
//         commands.spawn(DialogueTree {
//             tree,
//             fragment: associated_frag,
//         });
//     }
// }

pub trait IntoFragment<Data: FragmentData> {
    type Fragment: Fragment<Data>;

    fn into_fragment(self, commands: &mut Commands) -> (Self::Fragment, FragmentNode);

    /// Run a system any time this fragment is visited.
    fn on_visit<S, M>(self, system: S) -> OnVisit<Self, S::System>
    where
        S: IntoSystem<(), (), M>,
        Self: Sized,
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
        Self: Sized,
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
        Self: Sized,
    {
        OnEnd {
            fragment: self,
            on_trigger: IntoSystem::into_system(system),
        }
    }

    /// Map a dialogue event.
    fn map_event<S, M>(self, event: S) -> Mapped<Self, S, M>
    where
        M: Event,
        S: FnMut(&FragmentEvent<Data>) -> M + Send + Sync + 'static,
        Self: Sized,
    {
        Mapped {
            fragment: self,
            event,
            _marker: PhantomData,
        }
    }

    /// Wrap this fragment in an evaluation.
    fn eval<S, M, O>(self, system: S) -> Evaluated<Self, Unregistered<S::System>, O>
    where
        S: IntoSystem<(), O, M>,
        Self: Sized,
    {
        Evaluated {
            fragment: self,
            evaluation: Unregistered(IntoSystem::into_system(system)),
            _marker: PhantomData,
        }
    }

    /// Limit this fragment to `n` triggers.
    fn limit(self, n: usize) -> Limit<Self>
    where
        Self: Sized,
    {
        Limit::new(self, n)
    }
}

/// A convenience trait for setting a fragment's limit to 1.
pub trait Once<Data: FragmentData>: Sized {
    /// Set this fragment's limit to 1.
    fn once(self) -> Limit<Self>;
}

impl<T, Data> Once<Data> for T
where
    T: IntoFragment<Data>,
    Data: FragmentData,
{
    fn once(self) -> Limit<Self> {
        self.limit(1)
    }
}
