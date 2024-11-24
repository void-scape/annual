use super::evaluate::EvaluatedDialogue;
use crate::dialogue::{DialogueEvent, DialogueId};
use bevy::{ecs::system::SystemId, prelude::*};
use std::marker::PhantomData;

mod dynamic;
mod eval;
mod sequence;
mod string;
mod trigger;

pub use dynamic::{dynamic, Dynamic};
pub use eval::Evaluated;
pub use sequence::{sequence, Sequence};
pub use string::StringFragment;
pub use trigger::Trigger;

/// A wrapper for typestate management.
pub struct Unregistered<T>(T);

/// A type-erased fragment component.
#[derive(Component)]
pub struct ErasedFragment(pub Box<dyn Fragment + Send + Sync>);

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
    /// If this fragment and its children do not match the ID,
    /// this should do nothing.
    fn emit(
        &mut self,
        selected_id: DialogueId,
        writer: &mut EventWriter<DialogueEvent>,
        commands: &mut Commands,
    );

    /// This fragment's ID.
    ///
    /// This should be stable over the lifetime of the application.
    fn id(&self) -> &[DialogueId];
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

pub trait IntoFragment {
    type Fragment: Fragment;

    fn into_fragment(self, world: &mut World) -> Self::Fragment;

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
}