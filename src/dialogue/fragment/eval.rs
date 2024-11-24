use super::{Fragment, IntoFragment, Unregistered};
use crate::dialogue::evaluate::{Evaluate, EvaluatedDialogue, Evaluation};
use crate::dialogue::{DialogueEvent, DialogueId};
use bevy::ecs::system::SystemId;
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
    type Fragment = Evaluated<F::Fragment, (), O>;

    fn into_fragment(self, world: &mut World) -> Self::Fragment {
        let fragment = self.fragment.into_fragment(world);
        let id = fragment.id().to_owned();

        let mut schedules = world.resource_mut::<Schedules>();
        schedules.add_systems(
            Update,
            self.evaluation.0.pipe(
                move |eval: In<O>, mut evaluated_dialogue: ResMut<EvaluatedDialogue>| {
                    // TODO: clearly these evaluations should be additive, not simply clear each
                    // other out.
                    let eval = eval.0.evaluate();
                    for id in id.iter() {
                        evaluated_dialogue.insert(*id, eval);
                    }
                },
            ),
        );

        Evaluated {
            fragment,
            evaluation: (),
            _marker: self._marker,
        }
    }
}

impl<F, O> Fragment for Evaluated<F, (), O>
where
    F: Fragment,
{
    fn emit(
        &mut self,
        selected_id: DialogueId,
        writer: &mut EventWriter<DialogueEvent>,
        commands: &mut Commands,
    ) {
        self.fragment.emit(selected_id, writer, commands);
    }

    fn id(&self) -> &[DialogueId] {
        self.fragment.id()
    }
}
