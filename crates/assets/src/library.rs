//! A library serves as a device-wide registry of asset packs.

use bevy::prelude::Resource;
use semver::Version;
use serialization::{Deserialize, SerializationFormat, Serialize, deserialize, serialize_to};
use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::Read;
use std::path::PathBuf;
use thiserror::Error;
use utils::{DirectoryError, config_path};

/// The name of the library configuration file.
const LIBRARY_FILE_NAME: &str = "library.toml";

/// An [`AssetLibrary`] is a device-wide registry of packs that save files can refer to.
/// It handles as the bridge between relative paths within an asset pack and the actual paths on
/// a user's device.
#[derive(Resource, Debug, Serialize, Deserialize)]
pub struct AssetLibrary {
    /// The version of the software that last touched the library, used to help with future migrations.
    version: Version,
    /// A map of asset packs, keyed by their (public) identifiers.
    asset_packs: HashMap<String, AssetLibraryEntry>,
}

/// Represents an entry in the library, containing additional metadata about an asset pack.
#[derive(Default, Debug, Serialize, Deserialize)]
struct AssetLibraryEntry {
    /// Location of the asset pack on disk.
    ///
    /// Currently only filesystem packs are supported (e.g., no network-backed protocols like HTTP, FTP, ...)
    root: PathBuf,
    /// The location of the asset pack's index, this allows storing the index in a different location than the pack itself.
    index: PathBuf,
}

/// The errors that can occur when loading or saving the [`AssetLibrary`].
#[derive(Error, Debug)]
pub enum AssetLibraryError {
    /// An error occurred while locating the library configuration folder.
    #[error("failed to locate library configuration")]
    LocateConfigFolder(#[from] DirectoryError),
    /// An error occurred reading the configuration file itself.
    #[error("failed to read library configuration")]
    ReadFile(#[from] std::io::Error),
    /// An error occurred while (de)serializing the library configuration.
    #[error("failed to (de)serialize library configuration")]
    Serialization(#[from] serialization::SerializationError),
}

impl Default for AssetLibrary {
    fn default() -> Self {
        Self {
            version: utils::version().clone(),
            asset_packs: HashMap::new(),
        }
    }
}

impl AssetLibrary {
    /// Attempts to [`AssetLibrary::load`] the library from `path`, and returns the default value on [`IOError`].
    ///
    /// Any other error will be propagated by this method.
    ///
    /// # Errors
    /// See [`AssetLibrary::load`].
    pub fn load_or_default(path: Option<PathBuf>) -> Result<Self, AssetLibraryError> {
        match Self::load(path) {
            Err(AssetLibraryError::ReadFile(_)) => Ok(Self::default()),
            result => result,
        }
    }

    /// Attempts to load the asset library from `path`, where `path` is the configuration directory.
    /// If `None` is passed, the [`utils::config_path`] method is used instead.
    ///
    /// # Errors
    /// An error can be returned for the following situations:
    /// - The configuration folder could not be retrieved: [`AssetLibraryError::LocateConfigFolder`]
    /// - An error occurs while trying to read the config file (doesn't exist, permissions, ...):
    ///   [`AssetLibraryError::LocateConfigFolder`]
    /// - The file was found, could be read but failed to deserialize: [`AssetLibraryError::Serialization`].
    pub fn load(path: Option<PathBuf>) -> Result<Self, AssetLibraryError> {
        let path = Self::get_path(path)?.join(LIBRARY_FILE_NAME);

        let mut file = File::open(path).map_err(AssetLibraryError::ReadFile)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(AssetLibraryError::ReadFile)?;

        deserialize(contents.as_bytes(), &SerializationFormat::Toml)
            .map_err(AssetLibraryError::Serialization)
    }

    /// Saves the asset library.
    ///
    /// # Errors
    /// An error can be returned for the following situations:
    /// - The configuration folder could not be retrieved: [`AssetLibraryError::LocateConfigFolder`]
    /// - An error occurs while trying to read the config file (doesn't exist, permissions, ...):
    ///   [`AssetLibraryError::LocateConfigFolder`]
    /// - The file was found, could be read but failed to deserialize: [`AssetLibraryError::Serialization`].
    pub fn save(&self, path: Option<PathBuf>) -> Result<(), AssetLibraryError> {
        let path = Self::get_path(path)?;

        create_dir_all(&path)?; // Ensure the directory exists.
        let file = File::create(path.join(LIBRARY_FILE_NAME)).map_err(AssetLibraryError::ReadFile)?;
        serialize_to(self, &SerializationFormat::Toml, &file)?;
        Ok(())
    }

    /// Either returns `path` or `config_path()` if `path` is `None`.
    ///
    /// # Errors
    /// Returns an error if the configuration folder cannot be found.
    fn get_path(path: Option<PathBuf>) -> Result<PathBuf, AssetLibraryError> {
        let path = if let Some(path) = path {
            path
        } else {
            config_path().map_err(AssetLibraryError::LocateConfigFolder)?
        };

        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    // TODO: add unit tests.
}
