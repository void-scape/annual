use super::{End, Fragment, FragmentData, FragmentNode, IntoFragment, Start};
use crate::dialogue::evaluate::FragmentStates;
use crate::dialogue::{FragmentEvent, FragmentId};
use bevy::ecs::system::SystemId;
use bevy::prelude::*;

pub struct Dynamic<S> {
    id: FragmentId,
    system: S,
}

/// A dynamic text fragment.
///
/// This takes any system that outputs a string.
pub fn dynamic<S, M, O>(system: S) -> Dynamic<S::System>
where
    S: IntoSystem<(), O, M>,
{
    let id = FragmentId::random();
    Dynamic {
        id,
        system: IntoSystem::into_system(system),
    }
}

impl<S, Data> IntoFragment<Data> for Dynamic<S>
where
    Data: FragmentData,
    S: System<In = ()>,
    S::Out: Send + Sync + 'static + Into<Data>,
{
    type Fragment = Dynamic<SystemId>;

    fn into_fragment(self, commands: &mut Commands) -> (Self::Fragment, FragmentNode) {
        let id = self.id;
        (
            Dynamic {
                id,
                system: commands.register_one_shot_system(self.system.pipe(
                    move |data: In<S::Out>, mut writer: EventWriter<FragmentEvent<Data>>| {
                        writer.send(FragmentEvent {
                            id,
                            data: data.0.into(),
                        });
                    },
                )),
            },
            FragmentNode::leaf(id),
        )
    }
}

impl<Data> Fragment<Data> for Dynamic<SystemId>
where
    Data: FragmentData,
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
