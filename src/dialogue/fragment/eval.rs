use super::{Fragment, IntoFragment, Unregistered};
use crate::dialogue::evaluate::{Evaluate, EvaluatedDialogue};
use crate::dialogue::FragmentUpdate;
use bevy::prelude::*;
use std::marker::PhantomData;

pub struct Evaluated<F, T, O> {
    pub(super) fragment: F,
    pub(super) evaluation: T,
    pub(super) _marker: PhantomData<fn() -> O>,
}

impl<F, T, O> IntoFragment for Evaluated<F, Unregistered<T>, O>
where
    F: IntoFragment,
    T: System<In = (), Out = O>,
    O: Evaluate + Send + 'static,
{
    type Fragment = F::Fragment;

    fn into_fragment(self, commands: &mut Commands) -> Self::Fragment {
        let fragment = self.fragment.into_fragment(commands);
        let id = fragment.id().to_owned();

        commands.add(move |world: &mut World| {
            let mut schedules = world.resource_mut::<Schedules>();
            schedules.add_systems(
                FragmentUpdate,
                self.evaluation.0.pipe(
                    move |eval: In<O>, mut evaluated_dialogue: ResMut<EvaluatedDialogue>| {
                        let eval = eval.0.evaluate();
                        evaluated_dialogue.insert(id, eval);
                    },
                ),
            );
        });

        fragment
    }
}
