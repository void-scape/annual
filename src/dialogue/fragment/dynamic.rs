use super::{End, Fragment, FragmentData, FragmentNode, IntoFragment, Start};
use crate::dialogue::evaluate::FragmentStates;
use crate::dialogue::{FragmentEvent, FragmentId};
use bevy::ecs::system::SystemId;
use bevy::prelude::*;

struct CachedSystem<S> {
    system: Option<S>,
    id: Option<SystemId>,
}

pub struct Dynamic<S> {
    id: FragmentId,
    system: CachedSystem<S>,
}

/// A dynamic text fragment.
///
/// This takes any system that outputs a string.
pub fn dynamic<S, M, O>(system: S) -> Dynamic<impl System<In = (), Out = O>>
where
    S: IntoSystem<(), O, M>,
{
    let id = FragmentId::random();
    Dynamic {
        id,
        system: CachedSystem {
            system: Some(IntoSystem::into_system(system)),
            id: None,
        }, // system: Unregistered(IntoSystem::into_system(system.pipe(
           //     move |dialogue: In<String>, mut writer: EventWriter<FragmentEvent<Data>>| {
           //         writer.send(FragmentEvent {
           //             data: dialogue.0.into(),
           //             id,
           //         });
           //     },
           // ))),
    }
}

impl<S> IntoFragment for Dynamic<S> {
    type Fragment<Data> = Self;

    fn into_fragment<Data>(self, _: &mut Commands) -> (Self::Fragment<Data>, FragmentNode) {
        let id = self.id;
        (
            // Dynamic {
            //     id: self.id,
            //     system: ,
            //     _marker: self._marker,
            // },
            self,
            FragmentNode::leaf(id),
        )
    }
}

impl<S, Data> Fragment<Data> for Dynamic<S>
where
    S: System<In = ()>,
    S::Out: Send + Sync + 'static + Into<Data>,
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
            if self.system.id.is_none() {
                let sys = commands
                    .register_one_shot_system(self.system.system.take().unwrap().pipe(
                    move |dialogue: In<S::Out>, mut writer: EventWriter<FragmentEvent<Data>>| {
                        writer.send(FragmentEvent {
                            data: dialogue.0.into(),
                            id,
                        });
                    },
                ));

                self.system.id = Some(sys);
            }

            commands.run_system(self.system.id.unwrap());

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
