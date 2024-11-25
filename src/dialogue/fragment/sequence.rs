use super::{Emitted, Fragment, IntoFragment, StackList};
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
    id: DialogueId,
}

macro_rules! seq_frag {
    ($($ty:ident),*) => {
        #[allow(non_snake_case)]
        impl<$($ty),*> IntoFragment for ($($ty,)*)
        where
            $($ty: IntoFragment),*
        {
            type Fragment = Sequence<($($ty::Fragment,)*)>;

            fn into_fragment(self, commands: &mut Commands) -> Self::Fragment {
                #[allow(unused_mut)]
                let mut ids = Vec::new();
                let ($($ty,)*) = self;

                let seq = Sequence {
                    fragments: (
                        $(
                            {
                                let frag = $ty.into_fragment(commands);
                                ids.push(*frag.id());
                                frag
                            },
                        )*
                    ),
                    id: DialogueId::random(),
                };

                // TODO: this is basically a leak. It would be nice if we could
                // remove this when this sequene is no longer active.
                commands.spawn(SequenceItems(ids));

                seq
            }
        }

        #[allow(non_snake_case)]
        impl<$($ty),*> Fragment for Sequence<($($ty,)*)>
        where
            $($ty: Fragment),*
        {
            #[allow(unused)]
            fn emit(
                &mut self,
                selected_id: DialogueId,
                parent: Option<&StackList<DialogueId>>,
                writer: &mut EventWriter<DialogueEvent>,
                commands: &mut Commands,
            ) -> Emitted {
                let id = *self.id();
                let node = StackList::new(parent, &id);
                let mut emitted = Emitted::NotEmitted;

                let ($($ty,)*) = &mut self.fragments;
                $(emitted |= $ty.emit(selected_id, Some(&node), writer, commands);)*
                emitted
            }

            fn id(&self) -> &DialogueId {
                &self.id
            }
        }
    };
}

all_tuples!(seq_frag, 0, 15, T);
