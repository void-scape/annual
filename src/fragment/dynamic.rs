use super::{DialogueEvent, Fragment, IntoFragment, Unregistered};
use crate::evaluate::DialogueId;
use bevy::ecs::system::SystemId;
use bevy::prelude::*;

pub struct Dynamic<S> {
    id: DialogueId,
    system: S,
}

/// A dynamic text fragment.
///
/// This takes any system that outputs a string.
pub fn dynamic<S, I, M>(system: S) -> Dynamic<Unregistered<impl System<In = I, Out = ()>>>
where
    S: IntoSystem<I, String, M>,
{
    let id = DialogueId::random();
    Dynamic {
        id,
        system: Unregistered(IntoSystem::into_system(system.pipe(
            move |dialogue: In<String>, mut writer: EventWriter<DialogueEvent>| {
                writer.send(DialogueEvent {
                    dialogue: dialogue.0,
                    id,
                });
            },
        ))),
    }
}

impl<S> IntoFragment for Dynamic<Unregistered<S>>
where
    S: System<In = (), Out = ()>,
{
    type Fragment = Dynamic<SystemId>;

    fn into_fragment(self, world: &mut World) -> Self::Fragment {
        Dynamic {
            id: self.id,
            system: world.register_system(self.system.0),
        }
    }
}

impl IntoFragment for Dynamic<SystemId> {
    type Fragment = Self;

    fn into_fragment(self, world: &mut World) -> Self::Fragment {
        self
    }
}

impl Fragment for Dynamic<SystemId> {
    fn emit(
        &mut self,
        selected_id: DialogueId,
        writer: &mut EventWriter<DialogueEvent>,
        commands: &mut Commands,
    ) {
        if selected_id == self.id {
            commands.run_system(self.system)
        }
    }

    fn id(&self) -> &[crate::evaluate::DialogueId] {
        core::slice::from_ref(&self.id)
    }
}
