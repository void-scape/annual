use super::Scene;
use crate::gfx::post_processing::PostProcessCommand;
use crate::textbox::frags::IntoBox;
use crate::{annual, IntoFlower, IntoIzzy};
use bevy::core_pipeline::bloom::Bloom;
use bevy::prelude::*;
use bevy_light_2d::light::AmbientLight2d;
use bevy_pretty_text::prelude::*;
use bevy_seedling::sample::SamplePlayer;
use bevy_seedling::RepeatMode;
use bevy_sequence::prelude::FragmentExt;

pub struct SandboxPlugin;

impl Plugin for SandboxPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(
        //     Update,
        //     (leaf_particles,).run_if(super::scene_type_exists::<SandboxScene>),
        // );
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SandboxScene;

impl Scene for SandboxScene {
    fn spawn(&self, root: &mut EntityCommands) {
        let entity = root.id();
        root.commands().queue(init(entity));
    }
}

fn init(entity: Entity) -> impl Fn(&mut World) {
    move |world: &mut World| {
        if let Err(e) = world.run_system_cached_with(annual::sandbox::spawn, entity) {
            error!("failed to load level: {e}");
        }

        (
            s!("Donec faucibus, velit in dictum malesuada, `eros purus`[Shake(1.)] sit amet turpis.").flower(),
            s!("Lorem ipsum `dolor`[Wave(8.)] sit amet, consectetur `adipiscing|red`[Wave] elit. `Nullam|green` sed. `iuel|green`[Shake]").izzy(),
        )
            .once()
            .always()
            .spawn_box_with(&mut world.commands(), ());

        world.commands().post_process(AmbientLight2d {
            brightness: 0.1,
            color: Color::WHITE,
        });
        world.commands().post_process(Bloom::NATURAL);

        let handle = world.load_asset("sounds/music/quiet-night.wav");
        world.commands().spawn((
            SamplePlayer::new(handle),
            bevy_seedling::sample::PlaybackSettings {
                mode: RepeatMode::RepeatEndlessly,
                volume: 0.85,
            },
        ));
    }
}
