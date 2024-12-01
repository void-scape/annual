use crate::ldtk::LdtkAssets;
use bevy::prelude::*;
use bevy_asset_loader::loading_state::*;
use config::ConfigureLoadingState;

/// The main game state will be run during the [`AssetState::Loaded`] state.
///
/// Use [`crate::asset_loading::loaded`] to conditionally run systems.
/// ```
/// # use crate::asset_loading::loaded;
/// # fn run(app: &mut App, func: impl FnMut(Commands)) {
///     app.add_systems(Update, func.run_if(loaded()))
/// # }
/// ```
pub struct AssetLoadingPlugin;

impl Plugin for AssetLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AssetState>().add_loading_state(
            LoadingState::new(AssetState::Loading)
                .continue_to_state(AssetState::Loaded)
                .load_collection::<LdtkAssets>(),
            // .load_collection::<AudioAssets>(),
        );
    }
}

/// Determines if the game assets are loaded or not.
pub fn loaded() -> impl FnMut(Option<Res<State<AssetState>>>) -> bool + Clone {
    in_state(AssetState::Loaded)
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum AssetState {
    #[default]
    Loading,
    Loaded,
}
