use super::{Fragment, IntoFragment, Unregistered};
use crate::dialogue::{DialogueEvent, DialogueId};
use bevy::ecs::system::SystemId;
use bevy::prelude::*;

pub struct Trigger<F, T> {
    pub(super) fragment: F,
    pub(super) on_trigger: T,
}

impl<F, T> IntoFragment for Trigger<F, Unregistered<T>>
where
    F: IntoFragment,
    T: System<In = (), Out = ()>,
{
    type Fragment = Trigger<F::Fragment, SystemId>;

    fn into_fragment(self, world: &mut World) -> Self::Fragment {
        Trigger {
            fragment: self.fragment.into_fragment(world),
            on_trigger: world.register_system(self.on_trigger.0),
        }
    }
}

impl<F> IntoFragment for Trigger<F, SystemId>
where
    F: Fragment,
{
    type Fragment = Trigger<F, SystemId>;

    fn into_fragment(self, world: &mut World) -> Self::Fragment {
        self
    }
}

impl<F> Fragment for Trigger<F, SystemId>
where
    F: Fragment,
{
    fn emit(
        &mut self,
        selected_id: DialogueId,
        writer: &mut EventWriter<DialogueEvent>,
        commands: &mut Commands,
    ) {
        self.fragment.emit(selected_id, writer, commands);

        // Run triggers whenever any children are selected.
        if self.id().iter().any(|id| *id == selected_id) {
            commands.run_system(self.on_trigger);
        }
    }

    fn id(&self) -> &[DialogueId] {
        self.fragment.id()
    }
}
