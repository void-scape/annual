use bevy::{ecs::system::SystemId, prelude::*, utils::HashMap};
use paste::paste;
use std::marker::PhantomData;

use crate::evaluate::{Evaluate, Evaluator};

macro_rules! dialogue {
    ($scene:ident, $($dlog:expr, $dlog_cond:expr),*) => {
        pub struct $scene<M, C, O> {
            condition: C,
            _marker: PhantomData<fn() -> (M, O)>,
        }

        impl<M, O> $scene<M, (), O>
        where
            O: Condition<M> + Clone,
        {
            pub fn new(condition: O) -> $scene<M, impl Fn() -> O, O> {
                $scene {
                    condition: move || condition.clone(),
                    _marker: PhantomData,
                }
            }
        }

        paste! {
            impl<M: 'static, C: 'static + Send + Sync, O: 'static> Plugin for $scene<M, C, O>
            where
                C: Fn() -> O,
                O: Condition<M>,
            {
                fn build(&self, app: &mut App) {
                    app.add_systems(
                        Update,
                        evaluate_dialogue.in_set([<$scene Set>]),
                    );
                    app.configure_sets(Update, [<$scene Set>].run_if((self.condition)()));
                    app.add_systems(Startup, setup);
                    app.add_systems(Update, run_dialogue);
                }
            }

            #[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
            pub struct [<$scene Set>];
        }
    };
}

fn evaluate_dialogue(mut commands: Commands, mut evaluated_dialogue: ResMut<EvaluatedDialogue>) {
    evaluated_dialogue.insert_evaluation(
        DialogueHash(0),
        commands.run_system(commands.register_one_shot_system(d1_eval)),
    );
}

fn setup(mut commands: Commands, mut evaluated_dialogue: ResMut<EvaluatedDialogue>) {
    evaluated_dialogue.register(DialogueHash(0), commands.register_one_shot_system(d1));
    evaluated_dialogue.register(DialogueHash(1), commands.register_one_shot_system(d2));
}

dialogue!(IntroScene, d1, d1_eval, d2, d2_eval);

//////////////////////////////

#[derive(Component, Debug)]
struct EvaluatorResult(bool, usize);

#[derive(Component, Debug, Hash, PartialEq, Eq)]
struct DialogueHash(usize);

#[derive(Resource, Debug, Default)]
pub struct EvaluatedDialogue {
    evaluations: HashMap<DialogueHash, Evaluation>,
    oneshots: HashMap<DialogueHash, SystemId>,
}

impl EvaluatedDialogue {
    pub fn register(&mut self, hash: DialogueHash, id: SystemId) {
        self.oneshots.insert(hash, id);
    }

    pub fn insert_evaluation(&mut self, hash: DialogueHash, evaluation: Evaluation) {
        self.evaluations.insert(hash, evaluation);
    }

    pub fn clear(&mut self) {
        self.evaluations.clear();
    }
}

fn run_dialogue(mut commands: Commands, mut evaluated_dialogue: ResMut<EvaluatedDialogue>) {
    let evaluations = evaluated_dialogue.evaluations.drain().collect::<Vec<_>>();
    evaluations.sort_by_key(|(_, eval)| eval.count);
    if let Some(hash) = evaluations.find_map(|(hash, eval)| eval.result.then_some(hash)) {
        if let Some(oneshot) = evaluated_dialogue.oneshots.get(hash) {
            commands.run_system(*oneshot);
        }
    }

    evaluated_dialogue.clear();
}

/////////////

#[derive(Resource)]
pub struct DialogStep(pub usize);

#[derive(Component, Debug)]
struct IntroDialogueMarker;

pub fn d1_eval(mut writer: EventWriter<Evaluation>, step: Res<DialogStep>) -> Evaluation {
    writer.send(Evaluator::new([step.0 == 0]).evaluate());
}

pub fn d1(mut step: ResMut<DialogStep>) {
    println!("Hello, Synthia!");
    step.0 += 1;
}

pub fn d2_eval(step: Res<DialogStep>) -> Evaluation {
    step.0 == 1
}

pub fn d2(mut step: ResMut<DialogStep>) {
    println!("Hello, John. How are you doing?");
    step.0 += 1;
}

// static COOL_DIALGOUE: DialogueId = dlg!("Hello, Synthia!");
// static D2: DialogueId = dlg!(precondition = D1, "whatever");
//
// dialogue! {
//     D1 = "Hello",
//     D2 = "whatever",
// }
//
// fn idea() -> impl Dialogue {
//     dlg!("Here's some dialogue", |q: Query<Health>| {
//         q.single().unwrap().0 < 10
//     })
//
//     vec![
//         "Hello",
//         "My name is jeff",
//         "Yo",
//     ];
//
//     vec![
//         dlg!("Oh no!", |q: Query<Health>| { q.single().unwrap() < 10 }),
//         dlg!("Nice!", |q: Query<Health>| { q.single().unwrap() > 10 }),
//     ]
// }
