use bevy::prelude::*;

macro_rules! dlg {
    ($stuff:tt) => {};
}

trait Dialogue {}

impl<T> Dialogue for T {}

struct DialogueId(u64);
struct DialogueState {
    id: DialogueId,
    triggered: usize,
    active: bool,
}

#[derive(Resource)]
pub struct DialogStep(pub usize);

pub fn dog(mut step: ResMut<DialogStep>) {
    match step.0 {
        0 => {
            println!("Hello, Synthia!");
            step.0 += 1;
        }
        1 => {
            println!("Hello, John. How are you doing?");
            step.0 += 1;
        }
        _ => {}
    }
}

fn d1_eval(step: Mut<DialogStep>) {
    let result = Evaluator::new().condition(|| step.0 == 0).evaluate();
}

fn d1(mut step: ResMut<DialogStep>) {
    if trigger {
        println!("Hello, Synthia!");
        step.0 += 1;
    }
}

pub fn d1(mut step: ResMut<DialogStep>) {
    if step.0 == 0 {
        println!("Hello, Synthia!");
        step.0 += 1;
    }
}

pub fn d2(mut step: ResMut<DialogStep>) {
    if step.0 == 1 {
        println!("Hello, John. How are you doing?");
        step.0 += 1;
    }
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
