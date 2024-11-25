use super::{End, Fragment, FragmentNode, IntoFragment, Start, Unregistered};
use crate::dialogue::evaluate::DialogueStates;
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

    fn into_fragment(self, commands: &mut Commands) -> (Self::Fragment, FragmentNode) {
        (
            Dynamic {
                id: self.id,
                system: commands.register_one_shot_system(self.system.0),
            },
            FragmentNode::leaf(self.id),
        )
    }
}

impl Fragment for Dynamic<SystemId> {
    fn start(
        &mut self,
        id: DialogueId,
        state: &mut DialogueStates,
        _writer: &mut EventWriter<DialogueEvent>,
        commands: &mut Commands,
    ) -> Start {
        if id == self.id {
            commands.run_system(self.system);

            let state = state.update(id);
            state.triggered += 1;
            state.active = true;

            Start::Entered
        } else {
            Start::Unvisited
        }
    }

    fn end(&mut self, id: DialogueId, state: &mut DialogueStates, _commands: &mut Commands) -> End {
        if id == self.id {
            let state = state.update(id);
            state.completed += 1;
            state.active = false;

            End::Exited
        } else {
            End::Unvisited
        }
    }

    fn id(&self) -> &DialogueId {
        &self.id
    }
}
