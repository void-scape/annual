use crate::color::srgb_from_hex;
use crate::physics::prelude::*;
use crate::scenes::{Scene, SceneRoot};
use bevy::prelude::*;
use bevy_light_2d::prelude::*;
use rand::Rng;

const SPEED: f32 = 20.;

#[derive(Default, Component)]
#[require(Transform, Visibility, FireflySpawnerState)]
pub struct FireflySpawner {
    pub max: usize,
    pub rate: f32,
    pub lifetime: f32,
}

#[derive(Default, Component)]
pub struct FireflySpawnerState {
    active: usize,
}

#[derive(Component)]
pub struct FireflyParent(Entity);

#[derive(Default, Component)]
pub struct Firefly;

#[derive(Component)]
pub struct Lifetime {
    timer: Timer,
    light_intensity_curve: UnevenSampleAutoCurve<f32>,
}

/// Spawn new fireflies and bind them to the given scene.
pub fn spawn_fireflies<S: Scene>(
    mut commands: Commands,
    mut spawner_query: Query<(
        Entity,
        &FireflySpawner,
        &mut FireflySpawnerState,
        &GlobalTransform,
    )>,
    scene_root: Single<Entity, With<SceneRoot<S>>>,
) {
    let scene_root = scene_root.into_inner();

    for (entity, spawner, mut state, transform) in spawner_query.iter_mut() {
        if spawner.max > state.active {
            let rate = spawner.rate / 60.;
            (0..spawner.max - state.active).for_each(|_| {
                if rand::thread_rng().gen_range(0.0..1.0) < rate {
                    commands.entity(scene_root).with_child((
                        Firefly,
                        FireflyParent(entity),
                        Velocity(Vec2::new(
                            rand::thread_rng().gen_range(-1.0..1.0) * SPEED,
                            rand::thread_rng().gen_range(-1.0..1.0) * SPEED,
                        )),
                        Lifetime {
                            timer: Timer::from_seconds(spawner.lifetime, TimerMode::Once),
                            light_intensity_curve: UnevenSampleAutoCurve::new([
                                (0.0, 0.0),
                                (0.5, 2.0),
                                (1.0, 0.0),
                            ])
                            .unwrap(),
                        },
                        Transform::from_translation(
                            (transform.translation().xy()
                                + Vec2::new(
                                    rand::thread_rng().gen_range(-100..100) as f32,
                                    rand::thread_rng().gen_range(-100..100) as f32,
                                ))
                            .extend(0.),
                        ),
                        PointLight2d {
                            color: srgb_from_hex(0xffeb57),
                            intensity: 0.,
                            radius: 20.,
                            falloff: 100.,
                            ..default()
                        },
                    ));
                    state.active += 1;
                }
            });
        }
    }
}

pub fn update_lifetime(
    mut commands: Commands,
    mut spawner_query: Query<&mut FireflySpawnerState>,
    mut lifetime_query: Query<(Entity, &mut Lifetime, &mut PointLight2d, &FireflyParent)>,
    time: Res<Time>,
) {
    for (entity, mut lifetime, mut light, parent) in lifetime_query.iter_mut() {
        lifetime.timer.tick(time.delta());
        light.intensity = lifetime
            .light_intensity_curve
            .sample(lifetime.timer.fraction())
            .unwrap();
        if lifetime.timer.finished() {
            commands.entity(entity).despawn();
            let mut state = spawner_query.get_mut(parent.0).unwrap();
            state.active = state.active.saturating_sub(1);
        }
    }
}
