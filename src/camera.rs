use bevy::prelude::*;
use bevy::utils::hashbrown::HashSet;
use bevy_sequence::prelude::*;
use std::any::TypeId;
use std::marker::PhantomData;
use std::time::Duration;

use crate::curves::IntoCurve;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraSystemCache::default())
            .add_systems(Startup, init_camera)
            .add_systems(
                PostUpdate,
                camera_binded.before(TransformSystem::TransformPropagate),
            );
    }
}

fn init_camera(mut commands: Commands) {
    let mut proj = OrthographicProjection::default_2d();
    proj.scale = 0.25;
    commands.spawn((MainCamera, Camera2d, proj));
}

#[allow(unused)]
pub trait CameraCurveFragment<D, C>: Sized
where
    D: Threaded,
    C: Clone,
{
    /// Unbinds the camera and moves to the `marked` entity's position, with an offset, linearly over duration.
    fn move_camera_to<M: Component>(
        self,
        marker: M,
        offset: Vec3,
        duration: Duration,
    ) -> impl IntoFragment<D, C>
    where
        Self: IntoFragment<D, C>,
        D: Threaded;

    fn move_camera_curve<M: Component, I: Curve<Vec3>>(
        self,
        _marker: M,
        offset: Vec3,
        duration: Duration,
        curve: impl IntoCurve<I> + Send + Sync + 'static,
    ) -> CameraCurveFrag<impl IntoFragment<D, C>, I>;

    /// Bind the camera to an entity's position.
    fn bind_camera<M: Component>(self, marker: M) -> impl IntoFragment<D, C>;

    /// Unbinds the camera and moves to the `marked` entity's position, with an offset, linearly
    /// over duration.
    ///
    /// After the move is complete, the camera binds to the `marked` entity.
    fn move_then_bind_camera<M: Component>(
        self,
        marker: M,
        offset: Vec3,
        duration: Duration,
    ) -> impl IntoFragment<D, C>;

    fn move_curve_then_bind_camera<M: Component, I: Curve<Vec3>>(
        self,
        _marker: M,
        offset: Vec3,
        duration: Duration,
        curve: impl IntoCurve<I> + Send + Sync + 'static,
    ) -> CameraCurveFrag<impl IntoFragment<D, C>, I>;
}

impl<D, C, T> CameraCurveFragment<D, C> for T
where
    Self: IntoFragment<D, C>,
    D: Threaded,
    C: Threaded + Clone,
{
    fn move_camera_to<M: Component>(
        self,
        _marker: M,
        offset: Vec3,
        duration: Duration,
    ) -> impl IntoFragment<D, C> {
        let system = move |camera: Single<(Entity, &Transform), With<MainCamera>>,
                           entity_t: Single<&Transform, With<M>>,
                           mut commands: Commands| {
            let (camera, camera_t) = camera.into_inner();
            commands.entity(camera).insert(MoveTo::new(
                duration,
                EasingCurve::new(
                    camera_t.translation,
                    entity_t.translation + offset,
                    EaseFunction::Linear,
                ),
            ));
            commands.entity(camera).remove::<Binded>();
        };

        self.on_start(system)
    }

    fn move_camera_curve<M: Component, I: Curve<Vec3>>(
        self,
        _marker: M,
        offset: Vec3,
        duration: Duration,
        curve: impl IntoCurve<I> + Send + Sync + 'static,
    ) -> CameraCurveFrag<impl IntoFragment<D, C>, I> {
        let system = move |camera: Single<(Entity, &Transform), With<MainCamera>>,
                           entity_t: Single<&Transform, With<M>>,
                           mut commands: Commands| {
            let (camera, camera_t) = camera.into_inner();
            commands.entity(camera).insert(MoveTo::new(
                duration,
                curve.into_curve(camera_t.translation, entity_t.translation + offset),
            ));
            commands.entity(camera).remove::<Binded>();
        };

        CameraCurveFrag {
            fragment: self.on_start(system),
            _marker: PhantomData,
        }
    }

    fn bind_camera<M: Component>(self, _marker: M) -> impl IntoFragment<D, C> {
        self.on_start(bind_camera::<M>)
    }

    fn move_then_bind_camera<M: Component>(
        self,
        _marker: M,
        offset: Vec3,
        duration: Duration,
    ) -> impl IntoFragment<D, C> {
        let mov = move |camera: Single<(Entity, &Transform), With<MainCamera>>,
                        entity_t: Single<&Transform, With<M>>,
                        mut commands: Commands| {
            let (camera, camera_t) = camera.into_inner();
            commands.entity(camera).insert(MoveTo::new(
                duration,
                EasingCurve::new(
                    camera_t.translation,
                    entity_t.translation + offset,
                    EaseFunction::Linear,
                ),
            ));
            commands.entity(camera).remove::<Binded>();
        };

        self.on_start(mov).on_end(bind_camera::<M>)
    }

    fn move_curve_then_bind_camera<M: Component, I: Curve<Vec3>>(
        self,
        _marker: M,
        offset: Vec3,
        duration: Duration,
        curve: impl IntoCurve<I> + Send + Sync + 'static,
    ) -> CameraCurveFrag<impl IntoFragment<D, C>, I> {
        let system = move |camera: Single<(Entity, &Transform), With<MainCamera>>,
                           entity_t: Single<&Transform, With<M>>,
                           mut commands: Commands| {
            let (camera, camera_t) = camera.into_inner();
            commands.entity(camera).insert(MoveTo::new(
                duration,
                curve.into_curve(camera_t.translation, entity_t.translation + offset),
            ));
            commands.entity(camera).remove::<Binded>();
        };

        CameraCurveFrag {
            fragment: self.on_start(system).on_end(bind_camera::<M>),
            _marker: PhantomData,
        }
    }
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
struct MoveTo<C> {
    timer: Timer,
    curve: C,
}

impl<C> MoveTo<C>
where
    C: Curve<Vec3> + Threaded,
{
    pub fn new(duration: Duration, curve: C) -> Self {
        Self {
            timer: Timer::new(duration, TimerMode::Once),
            curve,
        }
    }

    pub fn position(&self) -> Option<Vec3> {
        self.curve.sample(self.timer.fraction())
    }

    pub fn tick(&mut self, duration: Duration) {
        self.timer.tick(duration);
    }

    pub fn complete(&self) -> bool {
        self.timer.finished()
    }
}

pub struct CameraCurveFrag<F, M> {
    fragment: F,
    _marker: PhantomData<fn() -> M>,
}

#[derive(Default, Resource)]
pub struct CameraSystemCache(HashSet<TypeId>);

impl<D, C, F, M> IntoFragment<D, C> for CameraCurveFrag<F, M>
where
    F: IntoFragment<D, C>,
    D: Threaded,
    M: Curve<Vec3> + Threaded,
{
    fn into_fragment(self, context: &Context<C>, commands: &mut Commands) -> FragmentId {
        let id = self.fragment.into_fragment(context, commands);

        commands.queue(|world: &mut World| {
            world.schedule_scope(PostUpdate, |world: &mut World, schedule: &mut Schedule| {
                let mut cache = world.resource_mut::<CameraSystemCache>();
                if cache.0.insert(std::any::TypeId::of::<M>()) {
                    schedule.add_systems(
                        camera_move_to::<M>.before(TransformSystem::TransformPropagate),
                    );
                }
            })
        });

        id
    }
}

#[derive(Component)]
pub struct Binded(pub Entity);

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

fn camera_move_to<C: Curve<Vec3> + Threaded>(
    camera: Option<Single<(Entity, &mut Transform, &mut MoveTo<C>), With<MainCamera>>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    if let Some((entity, mut transform, mut move_to)) = camera.map(|c| c.into_inner()) {
        move_to.tick(time.delta());
        if move_to.complete() {
            commands.entity(entity).remove::<MoveTo<C>>();
        } else {
            if let Some(position) = move_to.position() {
                transform.translation = position;
            }
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
