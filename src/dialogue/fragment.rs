use crate::dialogue::{DialogueEvent, DialogueId};
use bevy::prelude::*;
use std::marker::PhantomData;

mod dynamic;
mod eval;
mod limit;
mod mapped;
mod sequence;
mod string;
mod trigger;

pub use dynamic::dynamic;
pub use eval::Evaluated;
pub use limit::Limit;
pub use mapped::Mapped;
pub use trigger::Trigger;

pub(crate) use limit::update_limit_items;
pub(crate) use sequence::update_sequence_items;

/// A wrapper for typestate management.
pub struct Unregistered<T>(T);

/// A type-erased fragment component.
#[derive(Component)]
pub struct ErasedFragment(pub Box<dyn Fragment + Send + Sync>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Emitted {
    /// This node or one of its children emitted an event.
    Emitted,

    /// Neither this node nor any of its children emitted an event.
    NotEmitted,
}

impl Emitted {
    pub fn did_emit(&self) -> bool {
        matches!(self, Self::Emitted)
    }
}

impl core::ops::BitOr for Emitted {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::NotEmitted, Self::NotEmitted) => Self::NotEmitted,
            _ => Self::Emitted,
        }
    }
}

impl core::ops::BitOrAssign for Emitted {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

/// A linked list that can be constructed on the stack.
#[derive(Debug, Clone, Copy)]
pub struct StackList<'a, T> {
    parent: Option<&'a StackList<'a, T>>,
    node: &'a T,
}

impl<'a, T> StackList<'a, T> {
    pub fn new(parent: Option<&'a StackList<'a, T>>, node: &'a T) -> Self {
        Self { parent, node }
    }
}

impl From<StackList<'_, DialogueId>> for super::IdPath {
    fn from(value: StackList<'_, DialogueId>) -> Self {
        let mut node = Some(&value);
        let mut path = Vec::<DialogueId>::new();

        while let Some(n) = node {
            // de-duplicate nested identical IDs
            if !path.last().is_some_and(|l| l == n.node) {
                path.push(*n.node);
            }
            node = n.parent;
        }

        super::IdPath::new(path)
    }
}

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
pub trait Fragment {
    /// Emit events and run triggers for the selected ID.
    ///
    /// If this node or any of its children emitted an event,
    /// this should return [Emitted::Emitted].
    ///
    /// If this fragment and its children do not match the ID,
    /// this should do nothing.
    fn emit(
        &mut self,
        selected_id: DialogueId,
        parent: Option<&StackList<DialogueId>>,
        writer: &mut EventWriter<DialogueEvent>,
        commands: &mut Commands,
    ) -> Emitted;

    /// This fragment's ID.
    ///
    /// This should be stable over the lifetime of the application.
    fn id(&self) -> &DialogueId;
}

/// A convenience trait for type-erasing fragments.
pub trait BoxedFragment {
    fn boxed(self) -> Box<dyn Fragment + Send + Sync>;
}

impl<T> BoxedFragment for T
where
    T: Fragment + Send + Sync + 'static,
{
    fn boxed(self) -> Box<dyn Fragment + Send + Sync> {
        Box::new(self)
    }
}

#[allow(unused)]
pub trait IntoFragment {
    type Fragment: Fragment;

    fn into_fragment(self, commands: &mut Commands) -> Self::Fragment;

    /// Provide a trigger to this fragment.
    fn on_trigger<S, M>(self, system: S) -> Trigger<Self, Unregistered<S::System>>
    where
        S: IntoSystem<(), (), M>,
        Self: Sized,
    {
        Trigger {
            fragment: self,
            on_trigger: Unregistered(IntoSystem::into_system(system)),
        }
    }

    /// Map a dialogue event.
    fn map_event<S, E>(self, event: S) -> Mapped<Self, S, E>
    where
        E: Event,
        S: FnMut(&DialogueEvent) -> E + Send + Sync + 'static,
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
pub trait Once: Sized {
    /// Set this fragment's limit to 1.
    fn once(self) -> Limit<Self>;
}

impl<T> Once for T
where
    T: IntoFragment,
{
    fn once(self) -> Limit<Self> {
        self.limit(1)
    }
}
