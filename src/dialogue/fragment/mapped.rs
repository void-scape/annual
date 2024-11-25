use super::{Fragment, IntoFragment};
use crate::dialogue::{DialogueEvent, FragmentUpdate};
use bevy::{ecs::event::EventRegistry, prelude::*};
use std::marker::PhantomData;

/// Maps a dialogue event.
pub struct Mapped<F, S, E> {
    pub(super) fragment: F,
    pub(super) event: S,
    pub(super) _marker: PhantomData<fn() -> (S, E)>,
}

impl<F, S, E> IntoFragment for Mapped<F, S, E>
where
    F: IntoFragment,
    S: FnMut(&DialogueEvent) -> E + Send + Sync + 'static,
    E: Event + Clone,
{
    type Fragment = F::Fragment;

    fn into_fragment(self, commands: &mut Commands) -> Self::Fragment {
        let fragment = self.fragment.into_fragment(commands);

        let id = *fragment.id();
        let mut map = self.event;
        commands.add(move |world: &mut World| {
            if !world.contains_resource::<Events<E>>() {
                EventRegistry::register_event::<E>(world);
            }

            let mut schedules = world.resource_mut::<Schedules>();
            schedules.add_systems(
                FragmentUpdate,
                move |mut raw_events: EventReader<DialogueEvent>, mut wrapped: EventWriter<E>| {
                    for event in raw_events.read() {
                        if event.id_path.contains(&id) {
                            wrapped.send(map(event));
                        }
                    }
                },
            );
        });

        fragment
    }
}
