use crate::dialogue::DialogueId;
use bevy::prelude::*;
use rand::Rng;
use std::collections::{hash_map::Entry, HashMap};

pub trait Evaluate: sealed::Sealed {
    fn evaluate(&self) -> Evaluation;
}

mod sealed {
    pub trait Sealed {}

    impl Sealed for super::Evaluation {}
    impl<const LEN: usize> Sealed for [bool; LEN] {}
    impl Sealed for Vec<bool> {}
    impl Sealed for bool {}
}

impl Evaluate for bool {
    fn evaluate(&self) -> Evaluation {
        Evaluation {
            result: *self,
            count: 1,
        }
    }
}

impl<const LEN: usize> Evaluate for [bool; LEN] {
    fn evaluate(&self) -> Evaluation {
        Evaluation {
            result: self.iter().all(|e| *e),
            count: self.len(),
        }
    }
}

impl Evaluate for Vec<bool> {
    fn evaluate(&self) -> Evaluation {
        Evaluation {
            result: self.iter().all(|e| *e),
            count: self.len(),
        }
    }
}

impl Evaluate for Evaluation {
    fn evaluate(&self) -> Evaluation {
        *self
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Evaluation {
    pub result: bool,
    pub count: usize,
}

impl Evaluation {
    pub fn merge(&mut self, other: Evaluation) {
        self.result &= other.result;
        self.count += other.count;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DialogueState {
    pub triggered: usize,
    pub active: bool,
}

#[derive(Resource, Debug, Default)]
pub struct DialogueStates {
    pub state: HashMap<DialogueId, DialogueState>,
}

#[derive(Resource, Debug, Default)]
pub struct EvaluatedDialogue {
    pub(super) evaluations: HashMap<DialogueId, Evaluation>,
}

impl EvaluatedDialogue {
    pub fn insert<E: Evaluate>(&mut self, id: DialogueId, evaluation: E) {
        let eval = evaluation.evaluate();
        match self.evaluations.entry(id) {
            Entry::Vacant(e) => {
                e.insert(eval);
            }
            Entry::Occupied(mut e) => e.get_mut().merge(eval),
        }
    }

    pub fn clear(&mut self) {
        self.evaluations.clear();
    }
}

pub fn clear_evaluated_dialogue(mut evaluated_dialogue: ResMut<EvaluatedDialogue>) {
    evaluated_dialogue.clear();
}