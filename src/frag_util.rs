use bevy::prelude::*;
use bevy_sequence::prelude::*;
use rand::Rng;

pub trait FragExt<D, C>
where
    Self: IntoFragment<D, C> + Sized,
    D: Threaded,
    C: Threaded,
{
    fn sound(self, path: &'static str) -> impl IntoFragment<D, C> {
        self.sound_with(path, PlaybackSettings::DESPAWN)
    }

    fn sound_with(self, path: &'static str, settings: PlaybackSettings) -> impl IntoFragment<D, C> {
        let hash = TransientSound(rand::thread_rng().gen());

        self.on_start(
            move |mut commands: Commands, asset_server: Res<AssetServer>| {
                commands.spawn((AudioPlayer::new(asset_server.load(path)), settings, hash));
            },
        )
        //.on_end(
        //    move |mut _commands: Commands, sound_query: Query<(Entity, &TransientSound)>| {
        //        for (_entity, sound) in sound_query.iter() {
        //            if *sound == hash {
        //                // TODO: you don't really ever want to stop a sound abruptly
        //                //commands.entity(entity).despawn();
        //            }
        //        }
        //    },
        //)
    }
}

impl<T, D: Threaded, C: Threaded> FragExt<D, C> for T where T: IntoFragment<D, C> {}

#[derive(Clone, Copy, PartialEq, Eq, Component)]
struct TransientSound(usize);
