use bevy::{ecs::schedule::ScheduleLabel, prelude::*, utils::HashMap};
use std::marker::PhantomData;

macro_rules! dlg {
    ($stuff:tt) => {};
}

pub struct IntroDialogue<M, C, O> {
    cond: C,
    _marker: PhantomData<fn() -> (M, O)>,
}

impl<M, O> IntroDialogue<M, (), O>
where
    O: Condition<M> + Clone,
{
    pub fn new(condition: O) -> IntroDialogue<M, impl Fn() -> O, O> {
        IntroDialogue {
            cond: move || condition.clone(),
            _marker: PhantomData,
        }
    }
}

impl<M: 'static, C: 'static + Send + Sync, O: 'static> Plugin for IntroDialogue<M, C, O>
where
    C: Fn() -> O,
    O: Condition<M>,
{
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (d1.run_if(d1_eval), d2.run_if(d2_eval)).in_set(IntroSet),
        )
        .configure_sets(Update, IntroSet.run_if((self.cond)()));
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct IntroSet;

//////////////////////////////

/////////////

#[derive(Resource)]
pub struct DialogStep(pub usize);

#[derive(Component, Debug)]
struct IntroDialogueMarker;

pub fn d1_eval(step: Res<DialogStep>) -> bool {
    step.0 == 0
}

pub fn d1(mut step: ResMut<DialogStep>) {
    println!("Hello, Synthia!");
    step.0 += 1;
}

pub fn d2_eval(step: Res<DialogStep>) -> bool {
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
