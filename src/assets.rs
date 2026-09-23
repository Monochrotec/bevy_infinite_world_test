use bevy::prelude::*;
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{LoadingState, LoadingStateAppExt, config::ConfigureLoadingState},
};

pub struct MyAssetPlugin;

impl Plugin for MyAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AssetLoadingState>().add_loading_state(
            LoadingState::new(AssetLoadingState::Loading)
                .continue_to_state(AssetLoadingState::Next)
                .load_collection::<TileAssets>(),
        );
    }
}

#[derive(AssetCollection, Resource)]
pub struct TileAssets {
    #[asset(texture_atlas_layout(tile_size_x = 16, tile_size_y = 16, columns = 2, rows = 2))]
    pub atlas_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "tileset.png")]
    pub tileset_image: Handle<Image>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum AssetLoadingState {
    #[default]
    Loading,
    Next,
}
