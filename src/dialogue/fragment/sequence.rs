use super::{Fragment, IntoFragment};
use crate::dialogue::evaluate::{DialogueStates, EvaluatedDialogue};
use crate::dialogue::{DialogueEvent, DialogueId};
use bevy::prelude::*;
use bevy::utils::all_tuples;

#[derive(Debug, Component)]
pub struct SequenceItems(Vec<DialogueId>);

pub fn update_sequence_items(
    q: Query<&SequenceItems>,
    state: Res<DialogueStates>,
    mut evals: ResMut<EvaluatedDialogue>,
) {
    for items in q.iter() {
        for window in items.0.windows(2) {
            let eval = state.is_done(window[0]) && !state.has_triggered(window[1]);
            evals.insert(window[1], eval);
        }
    }
}

pub struct Sequence<F> {
    fragments: F,
    ids: Vec<DialogueId>,
}

macro_rules! seq_frag {
    ($($ty:ident),*) => {
        #[allow(non_snake_case)]
        impl<$($ty),*> IntoFragment for Sequence<($($ty,)*)>
        where
            $($ty: IntoFragment),*
        {
            type Fragment = Sequence<($($ty::Fragment,)*)>;

            fn into_fragment(self, world: &mut World) -> Self::Fragment {
                let mut ids = self.ids;
                let ($($ty,)*) = self.fragments;

                let seq = Sequence {
                    fragments: (
                        $({
                            let frag = $ty.into_fragment(world);
                            ids.extend(frag.id());
                            frag
                        },)*
                    ),
                    ids,
                };

                // TODO: this is basically a leak. It would be nice if we could
                // remove this when this sequene is no longer active.
                world.spawn(SequenceItems(seq.ids.clone()));

                seq
            }
        }

        #[allow(non_snake_case)]
        impl<$($ty),*> Fragment for Sequence<($($ty,)*)>
        where
            $($ty: Fragment),*
        {
            fn emit(
                &mut self,
                selected_id: DialogueId,
                writer: &mut EventWriter<DialogueEvent>,
                commands: &mut Commands,
            ) {
                let ($($ty,)*) = &mut self.fragments;
                $($ty.emit(selected_id, writer, commands);)*
            }

            fn id(&self) -> &[DialogueId] {
                &self.ids
            }
        }
    };
}

all_tuples!(seq_frag, 0, 15, T);

pub fn sequence<F>(fragments: F) -> Sequence<F> {
    Sequence {
        fragments,
        ids: Vec::new(),
    }
}
