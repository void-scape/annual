use app::LdtkEntityAppExt;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_ecs_ldtk::*;

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
            .register_ldtk_entity::<PlayerBundle>("Player")
            .add_systems(Startup, startup);
    }
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window.single();
    let (width, height) = (window.width(), window.height());

    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("ldtk/annual.ldtk"),
        transform: Transform::default().with_translation(Vec3::new(
            -width / 2.0,
            -height / 2.0,
            -100.0,
        )),
        ..Default::default()
    });
}

#[derive(Default, Component)]
struct Player;

impl Player {
    pub fn new(_: &EntityInstance) -> Self {
        Self {}
    }
}

#[derive(Bundle, LdtkEntity)]
struct PlayerBundle {
    #[with(Player::new)]
    player: Player,
    #[ldtk_entity]
    sprite: AnimatedSpriteSheetBundle,
}

#[derive(Bundle, LdtkEntity)]
struct AnimatedSpriteSheetBundle {
    #[sprite_sheet_bundle]
    sprite_sheet: LdtkSpriteSheetBundle,
}
