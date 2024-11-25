use super::{Fragment, IntoFragment};
use crate::dialogue::{DialogueEvent, DialogueId};
use bevy::prelude::*;

pub struct Binding<F, E> {
    pub(super) fragment: F,
    pub(super) event: Box<dyn Fn(DialogueId) -> E + Send + Sync>,
}

impl<F, E> IntoFragment for Binding<F, E>
where
    F: IntoFragment,
    E: Event + Clone,
{
    type Fragment = Binding<F::Fragment, E>;

    fn into_fragment(self, world: &mut World) -> Self::Fragment {
        Binding {
            fragment: self.fragment.into_fragment(world),
            event: self.event,
        }
    }
}

impl<F, E> Fragment for Binding<F, E>
where
    F: Fragment,
    E: Event + Clone,
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
            let event = (self.event)(selected_id);
            commands.add(|world: &mut World| {
                world.send_event(event);
            });
        }
    }

    fn id(&self) -> &[DialogueId] {
        self.fragment.id()
    }
}
