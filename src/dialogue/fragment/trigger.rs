use super::{Emitted, Fragment, IntoFragment, StackList, Unregistered};
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

    fn into_fragment(self, commands: &mut Commands) -> Self::Fragment {
        Trigger {
            fragment: self.fragment.into_fragment(commands),
            on_trigger: commands.register_one_shot_system(self.on_trigger.0),
        }
    }
}

impl<F> IntoFragment for Trigger<F, SystemId>
where
    F: Fragment,
{
    type Fragment = Trigger<F, SystemId>;

    fn into_fragment(self, _world: &mut Commands) -> Self::Fragment {
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
        parent: Option<&StackList<DialogueId>>,
        writer: &mut EventWriter<DialogueEvent>,
        commands: &mut Commands,
    ) -> Emitted {
        let id = *self.id();
        let node = StackList::new(parent, &id);
        let emitted = self
            .fragment
            .emit(selected_id, Some(&node), writer, commands);

        // Run triggers whenever any children are selected.
        if emitted.did_emit() {
            commands.run_system(self.on_trigger);
        }

        emitted
    }

    fn id(&self) -> &DialogueId {
        self.fragment.id()
    }
}
