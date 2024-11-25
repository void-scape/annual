use super::{End, Fragment, FragmentNode, IntoFragment, Start};
use crate::dialogue::evaluate::{DialogueStates, EvaluatedDialogue};
use crate::dialogue::{DialogueEvent, DialogueId};
use bevy::prelude::*;
use bevy::utils::all_tuples;

#[derive(Debug, Component)]
pub struct SequenceItems {
    id: DialogueId,
    children: Vec<DialogueId>,
}

pub fn update_sequence_items(
    q: Query<&SequenceItems>,
    state: Res<DialogueStates>,
    mut evals: ResMut<EvaluatedDialogue>,
) {
    for SequenceItems { id, children } in q.iter() {
        let outer_finished = state.state.get(id).map(|s| s.completed).unwrap_or_default();

        // look for the first item that has finished equal to the container
        let mut first_selected = false;
        for child in children.iter() {
            if !first_selected {
                let inner_finished = state
                    .state
                    .get(child)
                    .map(|s| s.completed)
                    .unwrap_or_default();

                if inner_finished <= outer_finished {
                    first_selected = true;
                    evals.insert(*child, true);

                    continue;
                }
            }

            evals.insert(*child, false);
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

            #[allow(unused_mut)]
            fn into_fragment(self, commands: &mut Commands) -> (Self::Fragment, FragmentNode) {
                let id = DialogueId::random();
                let mut ids = Vec::new();
                let mut node = FragmentNode::new(id, Vec::new());
                let ($($ty,)*) = self;

                let seq = Sequence {
                    fragments: (
                        $(
                            {
                                let (frag, n) = $ty.into_fragment(commands);
                                ids.push(*frag.id());
                                node.push(n);
                                frag
                            },
                        )*
                    ),
                    id,
                };

                // TODO: this is basically a leak. It would be nice if we could
                // remove this when this sequene is no longer active.
                commands.spawn(SequenceItems { id: seq.id, children: ids });

                (seq, node)
            }
        }

        #[allow(non_snake_case)]
        impl<$($ty),*> Fragment for Sequence<($($ty,)*)>
        where
            $($ty: Fragment),*
        {
            #[allow(unused)]
            fn start(
                &mut self,
                id: DialogueId,
                state: &mut DialogueStates,
                writer: &mut EventWriter<DialogueEvent>,
                commands: &mut Commands,
            ) -> Start {
                let mut states = Vec::<Start>::new();

                let ($($ty,)*) = &mut self.fragments;
                $(states.push($ty.start(id, state, writer, commands));)*

                if states.first().is_some_and(|f| f.entered()) {
                    state.update(self.id).triggered += 1;
                    Start::Entered
                } else if states.iter().any(|f| f.visited()) {
                    Start::Visited
                } else {
                    Start::Unvisited
                }
            }

            #[allow(unused)]
            fn end(
                &mut self,
                id: DialogueId,
                state: &mut DialogueStates,
                commands: &mut Commands
            ) -> End {
                let mut states = Vec::<End>::new();

                let ($($ty,)*) = &mut self.fragments;
                $(states.push($ty.end(id, state, commands));)*

                if states.last().is_some_and(|f| f.exited()) {
                    state.update(self.id).completed += 1;
                    End::Exited
                } else if states.iter().any(|f| f.visited()) {
                    End::Visited
                } else {
                    End::Unvisited
                }
            }

            fn id(&self) -> &DialogueId {
                &self.id
            }
        }
    };
}

all_tuples!(seq_frag, 0, 15, T);
