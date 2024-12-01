use crate::{dialogue_box::IntoBox, OnEnd, OnStart};
use bevy::prelude::*;
use std::time::Duration;

pub struct CutscenePlugin;

impl Plugin for CutscenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, apply_movements);
    }
}

/// This is applied to entities whose movement should be handled
/// purely by cutscene directives.
#[derive(Debug, Clone, Copy, Component)]
pub struct CutsceneMovement;

#[derive(Debug, Clone, Component)]
struct MovementClip {
    start: Vec3,
    end: Vec3,
    timer: Timer,
    // TODO: in 0.15, use the curves API
    // curve:
}

/// The instantaneous velocity resulting from cutscene movements.
#[derive(Debug, Component)]
pub struct CutsceneVelocity(pub Vec3);

impl MovementClip {
    pub fn position(&self) -> Vec3 {
        self.start.lerp(self.end, self.timer.fraction())
    }

    /// Advance the clip by the given time.
    pub fn tick(&mut self, duration: Duration) {
        self.timer.tick(duration);
    }

    pub fn complete(&self) -> bool {
        self.timer.finished()
    }
}

pub trait CutsceneFragment: Sized {
    fn move_to<M: Component, C: Component>(
        self,
        marker: M,
        position: Vec3,
        duration: Duration,
    ) -> impl IntoBox<C>
    where
        Self: IntoBox<C>;

    /// Lock entity movement during a cutscene.
    fn lock<M: Component>(
        self,
        marker: M,
    ) -> OnStart<OnEnd<Self, impl System<In = (), Out = ()>>, impl System<In = (), Out = ()>>;
}

impl<T> CutsceneFragment for T {
    fn move_to<M: Component, C: Component>(
        self,
        _marker: M,
        position: Vec3,
        duration: Duration,
    ) -> impl IntoBox<C>
    where
        Self: IntoBox<C>,
    {
        let system = move |q: Query<(Entity, &Transform), With<M>>,
                           root: Query<&Transform, With<C>>,
                           mut commands: Commands| {
            let root = root
                .get_single()
                .ok()
                .map(|t| t.translation)
                .unwrap_or_default();

            for (entity, transform) in q.iter() {
                commands.entity(entity).insert((
                    CutsceneMovement,
                    CutsceneVelocity(Vec3::ZERO),
                    MovementClip {
                        start: transform.translation,
                        end: root - position,
                        timer: Timer::new(duration, TimerMode::Once),
                    },
                ));
            }
        };

        OnStart::new(self, IntoSystem::into_system(system))
    }

    fn lock<M: Component>(
        self,
        _marker: M,
    ) -> OnStart<OnEnd<Self, impl System<In = (), Out = ()>>, impl System<In = (), Out = ()>> {
        let start = |q: Query<Entity, With<M>>, mut commands: Commands| {
            for entity in q.iter() {
                commands
                    .entity(entity)
                    .insert((CutsceneMovement, CutsceneVelocity(Vec3::ZERO)));
            }
        };

        let end = |q: Query<Entity, (With<M>, With<CutsceneMovement>)>, mut commands: Commands| {
            for entity in q.iter() {
                commands
                    .entity(entity)
                    .remove::<(CutsceneMovement, CutsceneVelocity)>();
            }
        };

        OnStart::new(
            OnEnd::new(self, IntoSystem::into_system(end)),
            IntoSystem::into_system(start),
        )
    }
}

fn apply_movements(
    mut q: Query<
        (
            Entity,
            &mut Transform,
            &mut MovementClip,
            &mut CutsceneVelocity,
        ),
        With<CutsceneMovement>,
    >,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut transform, mut clip, mut velocity) in q.iter_mut() {
        clip.tick(time.delta());

        let new_position = clip.position();
        let difference = new_position - transform.translation;
        transform.translation = new_position;
        velocity.0 = difference;

        if clip.complete() {
            // Not sure if this is totally ideal
            velocity.0 = Vec3::ZERO;
            commands.entity(entity).remove::<MovementClip>();
        }
    }
}
