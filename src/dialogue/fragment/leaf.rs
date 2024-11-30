use super::{End, Fragment, FragmentNode, FragmentStates, Start, Threaded};
use crate::dialogue::{FragmentEvent, FragmentId};
use bevy::prelude::*;

pub struct Leaf<T> {
    leaf: T,
    id: FragmentId,
}

impl<T> Leaf<T> {
    pub fn new(value: T) -> (Self, FragmentNode) {
        let id = FragmentId::random();

        (Leaf { leaf: value, id }, FragmentNode::leaf(id))
    }
}

impl<T, Data> Fragment<Data> for Leaf<T>
where
    Data: Threaded + From<T>,
    T: Clone,
{
    fn start(
        &mut self,
        id: FragmentId,
        state: &mut FragmentStates,
        writer: &mut EventWriter<FragmentEvent<Data>>,
        _commands: &mut Commands,
    ) -> Start {
        if id == self.id {
            writer.send(FragmentEvent {
                data: self.leaf.clone().into(),
                id: self.id,
            });

            let state = state.update(id);
            state.triggered += 1;
            state.active = true;

            Start::Entered
        } else {
            Start::Unvisited
        }
    }

    fn end(&mut self, id: FragmentId, state: &mut FragmentStates, _commands: &mut Commands) -> End {
        if id == self.id {
            let state = state.update(id);
            state.completed += 1;
            state.active = false;

            End::Exited
        } else {
            End::Unvisited
        }
    }

    fn id(&self) -> &FragmentId {
        &self.id
    }
}
