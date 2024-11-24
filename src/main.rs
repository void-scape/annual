use bevy::prelude::*;

fn main() {
    App::default()
        .add_plugins(DefaultPlugins)
        .add_systems(Update, bevy_bits::close_on_escape)
        .run();
}
