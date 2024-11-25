use super::{Fragment, FragmentData, FragmentNode, IntoFragment, Unregistered};
use crate::dialogue::evaluate::{Evaluate, EvaluatedFragments};
use crate::dialogue::FragmentUpdate;
use bevy::prelude::*;
use std::marker::PhantomData;

pub struct Evaluated<F, T, O> {
    pub(super) fragment: F,
    pub(super) evaluation: T,
    pub(super) _marker: PhantomData<fn() -> O>,
}

impl<Data, F, T, O> IntoFragment<Data> for Evaluated<F, Unregistered<T>, O>
where
    F: IntoFragment<Data>,
    T: System<In = (), Out = O>,
    O: Evaluate + Send + 'static,
    Data: FragmentData,
{
    type Fragment = F::Fragment;

    fn into_fragment(self, commands: &mut Commands) -> (Self::Fragment, FragmentNode) {
        let (fragment, node) = self.fragment.into_fragment(commands);
        let id = *fragment.id();

        commands.add(move |world: &mut World| {
            let mut schedules = world.resource_mut::<Schedules>();
            schedules.add_systems(
                FragmentUpdate,
                self.evaluation.0.pipe(
                    move |eval: In<O>, mut evaluated_dialogue: ResMut<EvaluatedFragments>| {
                        let eval = eval.0.evaluate();
                        evaluated_dialogue.insert(id, eval);
                    },
                ),
            );
        });

        (fragment, node)
    }
}
