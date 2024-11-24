use bevy::prelude::*;
use dialogue::DialogStep;

mod dialogue;
mod evaluate;

fn main() {
    App::default()
        .add_plugins(DefaultPlugins)
        .insert_resource(DialogStep(0))
        .insert_resource(dialogue::EvaluatedDialogue::default())
        .add_systems(Update, bevy_bits::close_on_escape)
        .add_plugins(dialogue::IntroScene::new(|| true))
        .run();
}
