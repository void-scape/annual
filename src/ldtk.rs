use crate::asset_loading::AssetState;
use assets::LdtkProject;
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_ecs_ldtk::*;

#[derive(AssetCollection, Resource)]
pub struct LdtkAssets {
    #[asset(path = "ldtk/annual.ldtk")]
    pub annual: Handle<LdtkProject>,
}

pub struct LdtkPlugin;

impl Plugin for LdtkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_ecs_ldtk::LdtkPlugin)
            // main level should be `level_0`
            .insert_resource(LevelSelection::index(0))
            .insert_resource(LdtkSettings {
                // level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                //     load_level_neighbors: true,
                // },
                ..default()
            })
            .add_systems(OnEnter(AssetState::Loaded), startup);
    }
}

fn startup(mut commands: Commands, assets: Res<LdtkAssets>) {
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: assets.annual.clone(),
        ..Default::default()
    });
}
