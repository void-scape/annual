use super::{End, Fragment, FragmentData, FragmentNode, IntoFragment, Start};
use crate::dialogue::evaluate::{EvaluatedFragments, FragmentStates};
use crate::dialogue::{FragmentEvent, FragmentId};
use bevy::prelude::*;
use bevy::utils::all_tuples;

#[derive(Debug, Component)]
pub struct SequenceItems {
    id: FragmentId,
    children: Vec<FragmentId>,
}

pub fn update_sequence_items(
    q: Query<&SequenceItems>,
    state: Res<FragmentStates>,
    mut evals: ResMut<EvaluatedFragments>,
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
    id: FragmentId,
}

macro_rules! seq_frag {
    ($($ty:ident),*) => {
        #[allow(non_snake_case)]
        impl< $($ty),*> IntoFragment for ($($ty,)*)
        where
            $($ty: IntoFragment),*
        {
            type Fragment<Data> = Sequence<($($ty::Fragment<Data>,)*)>;

            #[allow(unused_mut)]
            fn into_fragment<Data>(self, commands: &mut Commands) -> (Self::Fragment<Data>, FragmentNode) {
                let id = FragmentId::random();
                let mut ids = Vec::new();
                let mut node = FragmentNode::new(id, Vec::new());
                let ($($ty,)*) = self;

                let seq = Sequence {
                    fragments: (
                        $(
                            {
                                let (frag, n) = $ty.into_fragment(commands);
                                ids.push(n.id);
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
        impl<Data, $($ty),*> Fragment<Data> for Sequence<($($ty,)*)>
        where
            Data: FragmentData,
            $($ty: Fragment<Data>),*
        {
            #[allow(unused)]
            fn start(
                &mut self,
                id: FragmentId,
                state: &mut FragmentStates,
                writer: &mut EventWriter<FragmentEvent<Data>>,
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
                id: FragmentId,
                state: &mut FragmentStates,
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

            fn id(&self) -> &FragmentId {
                &self.id
            }
        }
    };
}

all_tuples!(seq_frag, 0, 15, T);

impl<T> IntoFragment for Vec<T>
where
    T: IntoFragment,
{
    type Fragment<Data> = Sequence<Vec<T::Fragment<Data>>>;

    fn into_fragment<Data>(self, commands: &mut Commands) -> (Self::Fragment<Data>, FragmentNode) {
        let id = FragmentId::random();
        let mut ids = Vec::new();
        let mut node = FragmentNode::new(id, Vec::new());

        let fragments = self
            .into_iter()
            .map(|frag| {
                let (frag, n) = frag.into_fragment(commands);
                ids.push(n.id);
                node.push(n);
                frag
            })
            .collect();

        let seq = Sequence { fragments, id };

        commands.spawn(SequenceItems {
            id: seq.id,
            children: ids,
        });

        (seq, node)
    }
}

impl<T, const LEN: usize> IntoFragment for [T; LEN]
where
    T: IntoFragment,
{
    type Fragment<Data> = Sequence<[T::Fragment<Data>; LEN]>;

    fn into_fragment<Data>(self, commands: &mut Commands) -> (Self::Fragment<Data>, FragmentNode) {
        let id = FragmentId::random();
        let mut ids = Vec::new();
        let mut node = FragmentNode::new(id, Vec::new());

        let mut fragments = self.into_iter();
        let fragments = core::array::from_fn(|_| {
            let (frag, n) = fragments.next().unwrap().into_fragment(commands);
            ids.push(n.id);
            node.push(n);
            frag
        });

        let seq = Sequence { fragments, id };

        commands.spawn(SequenceItems {
            id: seq.id,
            children: ids,
        });

        (seq, node)
    }
}

macro_rules! impl_iterable {
    ($ty:ident, $col:ty) => {
        impl<Data, $ty> Fragment<Data> for Sequence<$col>
        where
            Data: FragmentData,
            $ty: Fragment<Data>,
        {
            fn start(
                &mut self,
                id: FragmentId,
                state: &mut FragmentStates,
                writer: &mut EventWriter<FragmentEvent<Data>>,
                commands: &mut Commands,
            ) -> Start {
                let mut start = Start::Unvisited;

                for (i, frag) in self.fragments.iter_mut().enumerate() {
                    let frag_start = frag.start(id, state, writer, commands);

                    if i == 0 && frag_start.entered() {
                        state.update(self.id).triggered += 1;
                        start = Start::Entered;
                    }

                    if frag_start.visited() && start == Start::Unvisited {
                        start = Start::Visited;
                    }
                }

                start
            }

            fn end(
                &mut self,
                id: FragmentId,
                state: &mut FragmentStates,
                commands: &mut Commands,
            ) -> End {
                let mut end = End::Unvisited;
                let len = self.fragments.len();

                for (i, frag) in self.fragments.iter_mut().enumerate() {
                    let frag_end = frag.end(id, state, commands);

                    if i == len - 1 && frag_end.exited() {
                        state.update(self.id).completed += 1;
                        end = End::Exited;
                    }

                    if frag_end.visited() && end == End::Unvisited {
                        end = End::Visited;
                    }
                }

                end
            }

            fn id(&self) -> &FragmentId {
                &self.id
            }
        }
    };
}

impl_iterable!(T, Vec<T>);

// TODO: consolidate this somehow
impl<Data, T, const LEN: usize> Fragment<Data> for Sequence<[T; LEN]>
where
    Data: FragmentData,
    T: Fragment<Data>,
{
    fn start(
        &mut self,
        id: FragmentId,
        state: &mut FragmentStates,
        writer: &mut EventWriter<FragmentEvent<Data>>,
        commands: &mut Commands,
    ) -> Start {
        let mut start = Start::Unvisited;

        for (i, frag) in self.fragments.iter_mut().enumerate() {
            let frag_start = frag.start(id, state, writer, commands);

            if i == 0 && frag_start.entered() {
                state.update(self.id).triggered += 1;
                start = Start::Entered;
            }

            if frag_start.visited() && start == Start::Unvisited {
                start = Start::Visited;
            }
        }

        start
    }

    fn end(&mut self, id: FragmentId, state: &mut FragmentStates, commands: &mut Commands) -> End {
        let mut end = End::Unvisited;
        let len = self.fragments.len();

        for (i, frag) in self.fragments.iter_mut().enumerate() {
            let frag_end = frag.end(id, state, commands);

            if i == len - 1 && frag_end.exited() {
                state.update(self.id).completed += 1;
                end = End::Exited;
            }

            if frag_end.visited() && end == End::Unvisited {
                end = End::Visited;
            }
        }

        end
    }

    fn id(&self) -> &FragmentId {
        &self.id
    }
}
