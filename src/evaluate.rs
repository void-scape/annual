use bevy::{prelude::*, utils::all_tuples};
use std::collections::HashMap;

trait Evaluate: sealed::Sealed + Sized {
    fn evaluate(self) -> Evaluation;
}

mod sealed {
    pub trait Sealed {}

    impl<T> Sealed for super::Evaluator<T> {}
}

#[derive(Debug, Hash)]
struct DialogueId(u64);

struct DialogueState {
    id: DialogueId,
    triggered: usize,
    active: bool,
}

struct Evaluator<C> {
    count: usize,
    conditions: C,
}

impl Evaluator<()> {
    pub fn new() -> Self {
        Self {
            count: 0,
            conditions: (),
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

macro_rules! impl_eval {
    ($($condition:ident),*) => {
        #[allow(non_snake_case)]
        impl<$($condition),*> Evaluator<($($condition,)*)> {
            fn fact<F>(self, fact: F) -> Evaluator<($($condition,)* F,)>
            where
                F: FnOnce() -> bool
            {
                let ($($condition,)*) = self.conditions;

                Evaluator {
                    count: self.count + 1,
                    conditions: ($($condition,)* fact,)
                }
            }
        }

        #[allow(non_snake_case)]
        impl<F, $($condition),*> Evaluator<($($condition,)* F,)>
        where $($condition: FnOnce() -> bool,)*
            F: FnOnce() -> bool,
        {
            fn evaluate(self) -> Evaluation {
                let ($($condition,)* F,) = self.conditions;

                Evaluation {
                    result: $($condition()||)* F(),
                    count: self.count,
                }
            }
        }
    };
}

all_tuples!(impl_eval, 0, 16, T);

fn eval_d1(mut q: Query<()>, evals: ResMut<EvaluatedDialogue>) -> Evaluation {
    Evaluator::new()
        .fact(q.single_mut().is_dynamic())
        .fact(q.single_mut().is_dynamic())
        .evaluate()
}
