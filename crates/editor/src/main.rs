#![doc = include_str!("../README.md")]

mod panic;
use assets::AssetPlugin;

use bevy::prelude::*;
use config::Configuration;
use i18n::I18nPlugin;
use io::IOPlugin;
use logging::log_plugin;
use ui::UIPlugin;

/// Main entry point for the editor.
///
/// # Panics
/// The application will panic when a configuration error occurs, or when Bevy panics, specific
/// circumstances for when Bevy panics can be found in Bevy's documentation.
fn main() -> AppExit {
    panic::register_panic_handler();
    let config = match Configuration::load() {
        Ok(cfg) => cfg,
        Err(err) => panic!("Failed to load configuration: {err:?}"),
    };

    App::new()
        .add_plugins((
            DefaultPlugins.set(log_plugin(&config.logging)),
            I18nPlugin::new(&config.language),
            IOPlugin,
            UIPlugin,
            AssetPlugin,
        ))
        .insert_resource(config)
        .run()
}
