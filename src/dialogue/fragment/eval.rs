use super::{FragmentNode, IntoFragment, Unregistered};
use crate::dialogue::evaluate::{Evaluate, EvaluatedFragments};
use crate::dialogue::{EvaluateSet, FragmentUpdate};
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
    type Fragment<Data> = F::Fragment<Data>;

    fn into_fragment<Data>(self, commands: &mut Commands) -> (Self::Fragment<Data>, FragmentNode) {
        let (fragment, node) = self.fragment.into_fragment(commands);
        let id = node.id;

        commands.add(move |world: &mut World| {
            let mut schedules = world.resource_mut::<Schedules>();
            schedules.add_systems(
                FragmentUpdate,
                self.evaluation
                    .0
                    .pipe(
                        move |eval: In<O>, mut evaluated_dialogue: ResMut<EvaluatedFragments>| {
                            let eval = eval.0.evaluate();
                            evaluated_dialogue.insert(id, eval);
                        },
                    )
                    .in_set(EvaluateSet),
            );
        });

        (fragment, node)
    }
}
