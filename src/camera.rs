use bevy::prelude::*;
use bevy_sequence::prelude::*;
use std::time::Duration;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_camera).add_systems(
            PostUpdate,
            (camera_move_to, camera_binded).before(TransformSystem::TransformPropagate),
        );
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
    fn bind_camera<M: Component>(self, marker: M) -> impl IntoFragment<C, D>;

    /// Unbinds the camera and moves to the `marked` entity's position, with an offset, linearly
    /// over duration.
    ///
    /// After the move is complete, the camera binds to the `marked` entity.
    fn move_then_bind_camera<M: Component>(
        self,
        marker: M,
        position: Vec3,
        duration: Duration,
    ) -> impl IntoFragment<C, D>;
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
        let system = move |camera: Single<(Entity, &Transform), With<MainCamera>>,
                           entity_t: Single<&Transform, With<M>>,
                           mut commands: Commands| {
            let (camera, camera_t) = camera.into_inner();
            commands.entity(camera).insert(MoveTo::new(
                camera_t.translation,
                entity_t.translation + offset,
                duration,
            ));
            commands.entity(camera).remove::<Binded>();
        };

        self.on_start(system)
    }

    fn bind_camera<M: Component>(self, _marker: M) -> impl IntoFragment<C, D> {
        self.on_start(bind_camera::<M>)
    }

    fn move_then_bind_camera<M: Component>(
        self,
        _marker: M,
        offset: Vec3,
        duration: Duration,
    ) -> impl IntoFragment<C, D> {
        let mov = move |camera: Single<(Entity, &Transform), With<MainCamera>>,
                        entity_t: Single<&Transform, With<M>>,
                        mut commands: Commands| {
            let (camera, camera_t) = camera.into_inner();
            commands.entity(camera).insert(MoveTo::new(
                camera_t.translation,
                entity_t.translation + offset,
                duration,
            ));
            commands.entity(camera).remove::<Binded>();
        };

        self.on_start(mov).on_end(bind_camera::<M>)
    }
}

pub fn bind_camera<M: Component>(
    entity: Option<Single<Entity, (With<M>, With<Transform>)>>,
    camera: Option<Single<Entity, With<MainCamera>>>,
    mut commands: Commands,
) {
    if let Some(camera) = camera {
        if let Some(entity) = entity {
            commands
                .entity(camera.into_inner())
                .insert(Binded(entity.into_inner()));
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
    let mut proj = OrthographicProjection::default_2d();
    proj.scale = 0.25;
    commands.spawn((MainCamera, Camera2d, proj));
}

fn camera_move_to(
    camera: Option<Single<(Entity, &mut Transform, &mut MoveTo), With<MainCamera>>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    if let Some((entity, mut transform, mut move_to)) = camera.map(|c| c.into_inner()) {
        move_to.tick(time.delta());
        if move_to.complete() {
            commands.entity(entity).remove::<MoveTo>();
        } else {
            transform.translation = move_to.position();
        }
    }
}

fn camera_binded(
    camera: Option<Single<(&mut Transform, &Binded), With<MainCamera>>>,
    transforms: Query<&Transform, Without<MainCamera>>,
) {
    if let Some((mut transform, binded)) = camera.map(|c| c.into_inner()) {
        if let Ok(t) = transforms.get(binded.0) {
            transform.translation = t.translation;
        } else {
            warn!("Camera binded to entity with no transform");
        }
    }
}
