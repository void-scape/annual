use super::{End, Fragment, FragmentData, FragmentNode, IntoFragment, Start, Unregistered};
use crate::dialogue::evaluate::FragmentStates;
use crate::dialogue::{FragmentEvent, FragmentId};
use bevy::ecs::system::SystemId;
use bevy::prelude::*;
use std::marker::PhantomData;

pub struct Dynamic<S, Data> {
    id: FragmentId,
    system: S,
    _marker: PhantomData<Data>,
}

/// A dynamic text fragment.
///
/// This takes any system that outputs a string.
pub fn dynamic<Data, S, I, M>(
    system: S,
) -> Dynamic<Unregistered<impl System<In = I, Out = ()>>, Data>
where
    S: IntoSystem<I, String, M>,
    Data: FragmentData + From<String>,
{
    let id = FragmentId::random();
    Dynamic {
        id,
        system: Unregistered(IntoSystem::into_system(system.pipe(
            move |dialogue: In<String>, mut writer: EventWriter<FragmentEvent<Data>>| {
                writer.send(FragmentEvent {
                    data: dialogue.0.into(),
                    id,
                });
            },
        ))),
        _marker: PhantomData,
    }
}

impl<Data, S> IntoFragment<Data> for Dynamic<Unregistered<S>, Data>
where
    S: System<In = (), Out = ()>,
    Data: FragmentData,
    Data: From<String>,
{
    type Fragment = Dynamic<SystemId, Data>;

    fn into_fragment(self, commands: &mut Commands) -> (Self::Fragment, FragmentNode) {
        (
            Dynamic {
                id: self.id,
                system: commands.register_one_shot_system(self.system.0),
                _marker: self._marker,
            },
            FragmentNode::leaf(self.id),
        )
    }
}

impl<Data> Fragment<Data> for Dynamic<SystemId, Data>
where
    Data: FragmentData + From<String>,
{
    fn start(
        &mut self,
        id: FragmentId,
        state: &mut FragmentStates,
        _writer: &mut EventWriter<FragmentEvent<Data>>,
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

    fn end(&mut self, id: FragmentId, state: &mut FragmentStates, _commands: &mut Commands) -> End {
        if id == self.id {
            let state = state.update(id);
            state.completed += 1;
            state.active = false;

            End::Exited
        } else {
            End::Unvisited
        }
    }

    fn id(&self) -> &FragmentId {
        &self.id
    }
}
