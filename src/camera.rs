use crate::{IntoFragment, OnEnd, OnStart, Threaded};
use bevy::prelude::*;
use std::time::Duration;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_camera)
            .add_systems(Update, (camera_move_to, camera_binded));
    }
}

// TODO: curves, curves, curves
#[allow(unused)]
pub trait CameraFragment<C, D>: Sized
where
    C: Threaded + Clone,
{
    /// Unbinds the camera and moves to the `marked` entity's position, with an offset, linearly over duration.
    fn move_camera_to<M: Component>(
        self,
        marker: M,
        offset: Vec3,
        duration: Duration,
    ) -> impl IntoFragment<C, D>
    where
        Self: IntoFragment<C, D>,
        D: Threaded;

    /// Bind the camera to an entity's position.
    fn bind_camera<M: Component>(self, marker: M) -> OnStart<Self, impl System<In = (), Out = ()>>;

    /// Unbinds the camera and moves to the `marked` entity's position, with an offset, linearly
    /// over duration.
    ///
    /// After the move is complete, the camera binds to the `marked` entity.
    fn move_then_bind_camera<M: Component>(
        self,
        marker: M,
        position: Vec3,
        duration: Duration,
    ) -> OnStart<OnEnd<Self, impl System<In = (), Out = ()>>, impl System<In = (), Out = ()>>;
}

impl<C, D, T> CameraFragment<C, D> for T
where
    Self: IntoFragment<C, D>,
    D: Threaded,
    C: Threaded + Clone,
{
    fn move_camera_to<M: Component>(
        self,
        _marker: M,
        offset: Vec3,
        duration: Duration,
    ) -> impl IntoFragment<C, D>
    where
        D: Threaded,
    {
        let system = move |camera: Query<(Entity, &Transform), With<MainCamera>>,
                           entities: Query<&Transform, With<M>>,
                           mut commands: Commands| {
            if let Ok((camera, camera_t)) = camera.get_single() {
                if let Ok(entity_t) = entities.get_single() {
                    commands.entity(camera).insert(MoveTo::new(
                        camera_t.translation,
                        entity_t.translation + offset,
                        duration,
                    ));
                    commands.entity(camera).remove::<Binded>();
                }
            }
        };

        OnStart::new(self, IntoSystem::into_system(system))
    }

    fn bind_camera<M: Component>(
        self,
        _marker: M,
    ) -> OnStart<Self, impl System<In = (), Out = ()>> {
        OnStart::new(self, IntoSystem::into_system(bind_camera::<M>))
    }

    fn move_then_bind_camera<M: Component>(
        self,
        _marker: M,
        offset: Vec3,
        duration: Duration,
    ) -> OnStart<OnEnd<Self, impl System<In = (), Out = ()>>, impl System<In = (), Out = ()>> {
        let mov = move |camera: Query<(Entity, &Transform), With<MainCamera>>,
                        entities: Query<&Transform, With<M>>,
                        mut commands: Commands| {
            if let Ok((camera, camera_t)) = camera.get_single() {
                if let Ok(entity_t) = entities.get_single() {
                    commands.entity(camera).insert(MoveTo::new(
                        camera_t.translation,
                        entity_t.translation + offset,
                        duration,
                    ));
                    commands.entity(camera).remove::<Binded>();
                }
            }
        };

        OnStart::new(
            OnEnd::new(self, IntoSystem::into_system(bind_camera::<M>)),
            IntoSystem::into_system(mov),
        )
    }
}

pub fn bind_camera<M: Component>(
    entity: Query<Entity, (With<M>, With<Transform>)>,
    camera: Query<Entity, With<MainCamera>>,
    mut commands: Commands,
) {
    if let Ok(camera) = camera.get_single() {
        if let Ok(entity) = entity.get_single() {
            commands.entity(camera).insert(Binded(entity));
        } else {
            error!("Could not bind camera to entity: Entity not found");
        }
    } else {
        error!("Could not bind camera to entity: Camera not found");
    }
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
struct MoveTo {
    start: Vec3,
    end: Vec3,
    timer: Timer,
}

impl MoveTo {
    pub fn new(start: Vec3, end: Vec3, duration: Duration) -> Self {
        Self {
            timer: Timer::new(duration, TimerMode::Once),
            start,
            end,
        }
    }

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

#[derive(Component)]
pub struct Binded(pub Entity);

fn init_camera(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scale = 0.25;
    commands.spawn((MainCamera, camera));
}

fn camera_move_to(
    mut camera: Query<(Entity, &mut Transform, &mut MoveTo), With<MainCamera>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    if let Ok((entity, mut transform, mut move_to)) = camera.get_single_mut() {
        move_to.tick(time.delta());
        if move_to.complete() {
            commands.entity(entity).remove::<MoveTo>();
        } else {
            transform.translation = move_to.position();
        }
    }
}

fn camera_binded(
    mut camera: Query<(&mut Transform, &Binded), With<MainCamera>>,
    transforms: Query<&Transform, Without<MainCamera>>,
) {
    if let Ok((mut transform, binded)) = camera.get_single_mut() {
        if let Ok(t) = transforms.get(binded.0) {
            transform.translation = t.translation;
        } else {
            warn!("Camera binded to entity with no transform");
        }
    }
}
