use super::Scene;
use crate::gfx::post_processing::PostProcessCommand;
use crate::gfx::zorder::YOrigin;
use crate::physics::prelude::{Collider, StaticBody};
use crate::textbox::frags::IntoBox;
use crate::textbox::prelude::TextBoxPortrait;
use crate::{annual, IntoFlower, IntoIzzy};
use bevy::core_pipeline::bloom::Bloom;
use bevy::prelude::*;
use bevy_light_2d::light::AmbientLight2d;
use bevy_pretty_text::prelude::*;
use bevy_sequence::prelude::FragmentExt;

pub struct SandboxPlugin;

impl Plugin for SandboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                tree_collisions::<annual::ParkTree1>,
                tree_collisions::<annual::ParkTree2>,
                leave_particles,
            )
                .run_if(super::scene_type_exists::<SandboxScene>),
        );
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

        (s!("Lorem ipsum dolor sit amet, consectetur adipiscing elit. `Nullam|green` sed purus.").izzy(),
            s!("Donec faucibus, velit in dictum malesuada, `eros purus|red` sit amet turpis.").flower())
            .once()
            .always()
            .spawn_box_with(&mut world.commands(), ());

        world.commands().post_process(AmbientLight2d {
            brightness: 0.1,
            color: Color::WHITE,
        });
        world.commands().post_process(Bloom::NATURAL);
        let handle = world.load_asset("sounds/music/quiet-night.wav");
        world.commands().spawn(AudioPlayer::new(handle));
    }
}

fn leave_particles(
    mut commands: Commands,
    server: Res<AssetServer>,
    emitter_query: Query<Entity, Added<annual::LeafEmitter>>,
    mut materials: ResMut<Assets<bevy_enoki::prelude::SpriteParticle2dMaterial>>,
) {
    for entity in emitter_query.iter() {
        let sprite_material =
            materials.add(bevy_enoki::prelude::SpriteParticle2dMaterial::from_texture(
                server.load("sprites/leaf1.png"),
            ));

        commands.entity(entity).insert((
            bevy_enoki::ParticleSpawner(sprite_material),
            bevy_enoki::ParticleEffectHandle(server.load("particles/leaves.ron")),
        ));
    }
}

fn tree_collisions<C: Component>(
    mut commands: Commands,
    tree_query: Query<(Entity, &Transform), Added<C>>,
) {
    for (entity, _transform) in tree_query.iter() {
        commands.entity(entity).insert((
            StaticBody,
            Collider::from_circle(Vec2::new(104., -180.), 20.),
            YOrigin(-180.),
        ));
    }
}
