use crate::evaluate::{DialogueId, Evaluate, Evaluation};
use bevy::{ecs::system::SystemId, prelude::*, utils::HashMap};
use paste::paste;
use std::marker::PhantomData;

macro_rules! dialogue {
    ($scene:ident, $($dlog:expr, $dlog_cond:expr, $dlog_id:expr),*) => {
        pub struct $scene<C, M> {
            condition: C,
            _marker: PhantomData<fn() -> M>,
        }

        impl $scene<(), ()> {
            pub fn new<C, M>(condition: C) -> $scene<C, M>
            where
                C: Condition<M> + Clone,
            {
                $scene {
                    condition,
                    _marker: PhantomData,
                }
            }
        }

        paste! {
            impl<M: 'static, C: 'static + Send + Sync> Plugin for $scene<C, M>
            where
                C: Condition<M> + Clone,
            {
                fn build(&self, app: &mut App) {
                    app
                        .add_systems(
                            Startup,
                            |mut commands: Commands, mut evaluated: ResMut<EvaluatedDialogue>| {
                                $(
                                    evaluated.register($dlog_id, commands.register_one_shot_system($dlog));
                                )*
                            }
                        )
                        .add_systems(
                            Update,
                            (
                                $(map_eval($dlog_cond, $dlog_id),)*
                            )
                            .in_set([<$scene Set>]),
                        )
                        .configure_sets(Update, [<$scene Set>].run_if(self.condition.clone()))
                        .add_systems(Update, run_dialogue);
                }
            }

            #[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
            pub struct [<$scene Set>];
        }
    };
}

/// Pipe an evaluation system into one that updates the eval hash map.
///
/// The evaluation system can return anything that implements [Evaluate].
#[inline(always)]
fn map_eval<S, I, O, M>(eval_system: S, id: DialogueId) -> impl IntoSystem<I, (), ()>
where
    S: IntoSystem<I, O, M>,
    O: Evaluate + 'static,
{
    eval_system.pipe(
        move |eval: In<O>, mut evaluated_dialogue: ResMut<EvaluatedDialogue>| {
            evaluated_dialogue.insert_evaluation(id, eval.0.evaluate());
        },
    )
}

dialogue!(
    IntroScene,
    d1,
    d1_eval,
    DialogueId(0),
    d2,
    d2_eval,
    DialogueId(1)
);

//////////////////////////////

#[derive(Resource, Debug, Default)]
pub struct EvaluatedDialogue {
    evaluations: HashMap<DialogueId, Evaluation>,
    oneshots: HashMap<DialogueId, SystemId>,
}

impl EvaluatedDialogue {
    pub fn register(&mut self, hash: DialogueId, id: SystemId) {
        self.oneshots.insert(hash, id);
    }

    pub fn insert_evaluation(&mut self, hash: DialogueId, evaluation: Evaluation) {
        self.evaluations.insert(hash, evaluation);
    }

    pub fn clear(&mut self) {
        self.evaluations.clear();
    }
}

fn run_dialogue(mut commands: Commands, mut evaluated_dialogue: ResMut<EvaluatedDialogue>) {
    let mut evaluations = evaluated_dialogue.evaluations.drain().collect::<Vec<_>>();
    evaluations.sort_by_key(|(_, eval)| eval.count);
    if let Some(hash) = evaluations
        .iter()
        .find_map(|(hash, eval)| eval.result.then_some(hash))
    {
        if let Some(oneshot) = evaluated_dialogue.oneshots.get(hash) {
            commands.run_system(*oneshot);
        }
    }

    evaluated_dialogue.clear();
}

/////////////

#[derive(Resource)]
pub struct DialogStep(pub usize);

pub fn d1_eval(step: Res<DialogStep>) -> impl Evaluate {
    step.0 == 0
}

pub fn d1(mut step: ResMut<DialogStep>) {
    println!("Hello, Synthia!");
    step.0 += 1;
}

pub fn d2_eval(step: Res<DialogStep>) -> impl Evaluate {
    step.0 == 1
}

pub fn d2(mut step: ResMut<DialogStep>) {
    println!("Hello, John. How are you doing?");
    step.0 += 1;
}

// // Dialogue with a globally-scoped ID.
// static TAGGED_DIALOGUE: DialogueId = dialogue!("Hey John...");
//
// fn scene(mut commands: Commands) {
//     // A sequence takes a tuple of dialogue items where
//     // each item has an implicit evaluation on whether the
//     // previous item has finished.
//     let scene = sequence((
//         // Most dialogue won't need a global ID -- it can just be dynamically generated.
//         "Hello, Synthia!",
//         TAGGED_DIALOGUE,
//         // `any` will advance as soon as at least one item has finished.
//         any((
//             // `eval` allows us to pass an explicit evaulation.
//             eval(
//                 demon_slayer_eval,
//                 sequence((
//                     "Did you slay the demon?",
//                     |slain: Res<DemonsSlain>| format!("Yes, I've slain {} in fact.", slain.0),
//                     "Wow!",
//                 )),
//             ),
//             sequence((
//                 // We can add `on_trigger` hooks (which are just systems) to any bit of dialogue.
//                 "How's the weather?".on_trigger(|mut weather_question: ResMut<Weather>| *dq = true),
//                 "Hm.. seems okay...",
//             )),
//         )),
//         "Anyway, let's get going, shall we?",
//     ));
//
//     scene.register(commands);
// }
//
// fn demon_slayer_eval(slain: Res<DemonsSlain>) -> impl Evalate {
//     slain.0 != 0
// }
