use bevy::prelude::*;
use rand::Rng;
use std::collections::HashMap;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DialogueId(u64);

impl DialogueId {
    pub fn random() -> Self {
        Self(rand::thread_rng().gen())
    }
}

pub struct DialogueState {
    id: DialogueId,
    triggered: usize,
    active: bool,
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

#[derive(Resource, Debug, Default)]
pub struct EvaluatedDialogue {
    evaluations: HashMap<DialogueId, Evaluation>,
}

impl EvaluatedDialogue {
    pub fn clear(&mut self) {
        self.evaluations.clear();
    }
}

pub fn clear_evaluated_dialogue(mut evaluated_dialogue: ResMut<EvaluatedDialogue>) {
    evaluated_dialogue.clear();
}
