#![allow(unused)]
use bevy::{
    input::{
        keyboard::{Key, KeyboardInput},
        ButtonState,
    },
    prelude::*,
    window::PrimaryWindow,
};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "editor")]
        app.add_plugins(bevy_editor_pls::EditorPlugin::default());
        // .add_systems(Startup, (maximize_window, switch_view));
    }
}

fn maximize_window(mut window: Query<&mut Window, With<PrimaryWindow>>) {
    let mut window = window.single_mut();
    window.set_maximized(true);
}

fn switch_view(mut writer: EventWriter<KeyboardInput>, window: Query<Entity, With<PrimaryWindow>>) {
    writer.send(KeyboardInput {
        key_code: KeyCode::KeyE,
        logical_key: Key::Character("e".into()),
        state: ButtonState::Pressed,
        window: window.single(),
    });
}
