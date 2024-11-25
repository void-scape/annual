use super::{Emitted, Fragment, IntoFragment, StackList, Unregistered};
use crate::dialogue::{DialogueEvent, DialogueId};
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
                    id_path: todo!(),
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

    fn into_fragment(self, commands: &mut Commands) -> Self::Fragment {
        Dynamic {
            id: self.id,
            system: commands.register_one_shot_system(self.system.0),
        }
    }
}

impl Fragment for Dynamic<SystemId> {
    fn emit(
        &mut self,
        selected_id: DialogueId,
        parent: Option<&StackList<DialogueId>>,
        _writer: &mut EventWriter<DialogueEvent>,
        commands: &mut Commands,
    ) -> Emitted {
        let node = StackList::new(parent, self.id());

        // TODO: how to pass in id path??
        if selected_id == self.id {
            commands.run_system(self.system);
            Emitted::Emitted
        } else {
            Emitted::NotEmitted
        }
    }

    fn id(&self) -> &DialogueId {
        &self.id
    }
}
