use std::time::Duration;

use super::{Fragment, IntoFragment, Threaded};
use bevy::{ecs::system::SystemId, prelude::*};

#[derive(Component, Clone)]
pub(crate) struct AfterSystem(SystemId, Timer);

pub(crate) fn manage_delay(
    mut q: Query<(Entity, &mut AfterSystem)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut sys) in q.iter_mut() {
        sys.1.tick(time.delta());
        if sys.1.finished() {
            commands.run_system(sys.0);
            commands.entity(entity).despawn();
        }
    }
}

pub struct Delay<F, S> {
    fragment: F,
    system: S,
    duration: Duration,
}

impl<F, S> Delay<F, S> {
    pub fn new(fragment: F, duration: Duration, system: S) -> Self {
        Self {
            fragment,
            duration,
            system,
        }
    }
}

impl<F, S, D> IntoFragment<D> for Delay<F, S>
where
    D: Threaded,
    F: IntoFragment<D>,
    S: System<In = (), Out = ()> + 'static,
{
    type Fragment = Delay<F::Fragment, SystemId>;

    fn into_fragment(self, commands: &mut Commands) -> (Self::Fragment, super::FragmentNode) {
        let (fragment, n) = self.fragment.into_fragment(commands);

        (
            Delay {
                fragment,
                duration: self.duration,
                system: commands.register_one_shot_system(self.system),
            },
            n,
        )
    }
}

impl<F, D> Fragment<D> for Delay<F, SystemId>
where
    D: Threaded,
    F: Fragment<D>,
{
    fn start(
        &mut self,
        id: crate::dialogue::FragmentId,
        state: &mut crate::dialogue::evaluate::FragmentStates,
        writer: &mut EventWriter<crate::dialogue::FragmentEvent<D>>,
        commands: &mut Commands,
    ) -> super::Start {
        self.fragment.start(id, state, writer, commands)
    }

    fn end(
        &mut self,
        id: crate::dialogue::FragmentId,
        state: &mut crate::dialogue::evaluate::FragmentStates,
        commands: &mut Commands,
    ) -> super::End {
        let end = self.fragment.end(id, state, commands);

        if end.exited() {
            commands.spawn(AfterSystem(
                self.system,
                Timer::new(self.duration, TimerMode::Once),
            ));
        }

        end
    }

    fn id(&self) -> &crate::dialogue::FragmentId {
        self.fragment.id()
    }
}
