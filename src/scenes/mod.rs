use bevy::ecs::component::StorageType;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_sequence::fragment::DataLeaf;
use bevy_sequence::prelude::*;
use std::any::TypeId;
//use bevy::ecs::world::DeferredWorld;
//use bevy::tasks::IoTaskPool;
//use std::fs::File;
//use std::io::Write;

pub mod home;
pub mod park;
mod point_light;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_ldtk_scene::LdtkScenePlugin)
            .insert_resource(SceneSystemCache::default())
            .add_systems(Update, point_light::init_point_light_tiles);
    }
}

pub trait Scene: 'static + Send + Sync + Clone {
    //const SAVE_PATH: &'static str;

    fn spawn(root: &mut EntityCommands);
    //fn save(_world: &mut DeferredWorld, _root: Entity) -> Option<Vec<u8>> {
    //    None
    //}
}

pub struct SceneTransition<F, T> {
    from: F,
    to: T,
}

impl<F, T> Default for SceneTransition<F, T>
where
    F: Default,
    T: Default,
{
    fn default() -> Self {
        Self {
            from: F::default(),
            to: T::default(),
        }
    }
}

impl<F, T> Clone for SceneTransition<F, T>
where
    F: Scene,
    T: Scene,
{
    fn clone(&self) -> Self {
        Self {
            from: self.from.clone(),
            to: self.to.clone(),
        }
    }
}

pub fn scene_transition<F: Scene, S: Scene>(
    mut commands: Commands,
    mut reader: EventReader<FragmentEvent<SceneTransition<F, S>>>,
    from: Option<Single<Entity, With<SceneRoot<F>>>>,
) {
    for event in reader.read() {
        if let Some(ref from) = from {
            commands.entity(**from).despawn_recursive();
            commands.spawn(SceneRoot::new(event.data.to.clone()));
        }
    }
}

impl<F: Scene + Clone, S: Scene + Clone> IntoFragment<SceneTransition<F, S>, ()>
    for SceneTransition<F, S>
{
    fn into_fragment(self, context: &Context<()>, commands: &mut Commands) -> FragmentId {
        <_ as IntoFragment<SceneTransition<F, S>, ()>>::into_fragment(
            DataLeaf::new(self),
            context,
            commands,
        )
    }
}

pub struct SceneRoot<S: Scene>(S);

impl<S: Scene> SceneRoot<S> {
    pub fn new(root: S) -> Self {
        Self(root)
    }
}

impl<S: Scene> Component for SceneRoot<S> {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks
            .on_add(|mut world, entity, _| {
                S::spawn(&mut world.commands().entity(entity));
            })
            .on_remove(|_world, _entity, _| {
                //if let Some(save) = S::save(&mut world, entity) {
                //    world
                //        .commands()
                //        .write_to_file(S::SAVE_PATH, save);
                //}
            });
    }
}

pub trait SceneCommands {
    fn add_scoped_systems<S, C, M>(&mut self, _scene: S, schedule: impl ScheduleLabel, systems: C)
    where
        S: Scene,
        C: IntoSystemConfigs<M> + Send + 'static;

    //fn write_to_file(&self, path: impl Into<String>, data: Vec<u8>);
}

impl SceneCommands for Commands<'_, '_> {
    fn add_scoped_systems<S, C, M>(&mut self, _scene: S, schedule: impl ScheduleLabel, systems: C)
    where
        S: Scene,
        C: IntoSystemConfigs<M> + Send + 'static,
    {
        self.queue(move |world: &mut World| {
            world.schedule_scope(schedule, |world: &mut World, schedule: &mut Schedule| {
                let mut cache = world.resource_mut::<SceneSystemCache>();
                if cache.0.insert(TypeId::of::<C>()) {
                    schedule.add_systems(systems.run_if(scene_exists::<S>));
                }
            })
        });
    }

    //fn write_to_file(&self, path: impl Into<String>, data: Vec<u8>) {
    //    let path = path.into();
    //
    //    #[cfg(not(target_arch = "wasm32"))]
    //    IoTaskPool::get()
    //        .spawn(async move {
    //            if let Err(e) = File::create(&path).and_then(|mut file| file.write(&data)) {
    //                error!("failed to write to file {:?}: {e}", &path);
    //            }
    //        })
    //        .detach();
    //}
}

#[derive(Default, Resource)]
struct SceneSystemCache(HashSet<TypeId>);

fn scene_exists<S: Scene>(scene_query: Option<Single<&SceneRoot<S>>>) -> bool {
    scene_query.is_some()
}
