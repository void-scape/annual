use bevy::prelude::*;
use dialogue::DialogStep;

mod dialogue;

use dialogue::*;

fn main() {
    App::default()
        .add_plugins(DefaultPlugins)
        .insert_resource(DialogStep(0))
        .add_systems(Update, bevy_bits::close_on_escape)
        .add_systems(Update, (d1, d2))
        .run();
}
