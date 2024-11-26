use crate::dialogue::FragmentId;
use bevy::prelude::*;
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FragmentState {
    pub triggered: usize,
    pub completed: usize,
    pub active: bool,
}

#[derive(Resource, Debug, Default)]
pub struct FragmentStates {
    pub state: HashMap<FragmentId, FragmentState>,
}

#[allow(unused)]
impl FragmentStates {
    pub fn update(&mut self, id: FragmentId) -> &mut FragmentState {
        self.state.entry(id).or_default()
    }

    pub fn is_done(&self, id: FragmentId) -> bool {
        self.state
            .get(&id)
            .is_some_and(|s| s.completed >= 1 && !s.active)
    }

    pub fn is_active(&self, id: FragmentId) -> bool {
        self.state.get(&id).is_some_and(|s| s.active)
    }

    pub fn has_triggered(&self, id: FragmentId) -> bool {
        self.state.get(&id).is_some_and(|s| s.triggered > 0)
    }
}

#[derive(Resource, Debug, Default)]
pub struct EvaluatedFragments {
    pub(super) evaluations: HashMap<FragmentId, Evaluation>,
}

#[allow(unused)]
impl EvaluatedFragments {
    pub fn insert<E: Evaluate>(&mut self, id: FragmentId, evaluation: E) {
        let eval = evaluation.evaluate();
        match self.evaluations.entry(id) {
            Entry::Vacant(e) => {
                e.insert(eval);
            }
            Entry::Occupied(mut e) => e.get_mut().merge(eval),
        }
    }

    pub fn get(&self, id: FragmentId) -> Option<Evaluation> {
        self.evaluations.get(&id).copied()
    }

    /// Returns whether the provided ID should be further evaulated.
    ///
    /// An ID not in the set will always return false.
    pub fn is_candidate(&self, id: FragmentId) -> bool {
        self.evaluations
            .get(&id)
            .map(|e| e.result)
            .unwrap_or_default()
    }

    pub fn clear(&mut self) {
        self.evaluations.clear();
    }
}
