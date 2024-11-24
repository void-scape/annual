use bevy::{prelude::*, utils::all_tuples};
use std::collections::HashMap;

trait Evaluate: sealed::Sealed {
    fn evaluate(&self) -> Evaluation;
}

mod sealed {
    pub trait Sealed {}

    impl<const LEN: usize> Sealed for super::Evaluator<LEN> {}
}

#[derive(Debug, Hash)]
struct DialogueId(u64);

struct DialogueState {
    id: DialogueId,
    triggered: usize,
    active: bool,
}

struct Evaluator<const LEN: usize> {
    conditions: [bool; LEN],
}

impl<const LEN: usize> Evaluator<LEN> {
    pub fn new(conditions: [bool; LEN]) -> Self {
        Self { conditions }
    }
}

impl<const LEN: usize> Evaluate for Evaluator<LEN> {
    fn evaluate(&self) -> Evaluation {
        Evaluation {
            result: self.conditions.iter().all(|c| *c),
            count: LEN,
        }
    }
}

#[derive(Component, Debug)]
struct Evaluation {
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

fn eval_d1(mut q: Query<()>, evals: ResMut<EvaluatedDialogue>) -> Evaluation {
    Evaluator::new([q.single_mut().is_dynamic(), q.single_mut().is_dynamic()]).evaluate()
}
