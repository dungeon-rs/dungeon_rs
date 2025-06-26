//! The [`AssetPlugin`] is responsible for loading the required information into Bevy.

use crate::library::AssetLibrary;
use bevy::app::App;
use bevy::prelude::Plugin;

/// Handles registering the required resources and functionality for the asset system.
#[derive(Default)]
pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        let library = AssetLibrary::load_or_default(None).expect("Failed to load asset library");

        app.insert_resource(library);
    }
}
