//! A library serves as a device-wide registry of asset packs.

use crate::{AssetPack, AssetPackError};
use bevy::prelude::Resource;
use semver::Version;
use serialization::{Deserialize, SerializationFormat, Serialize, deserialize, serialize_to};
use std::collections::HashMap;
use std::fs::{File, create_dir_all};
use std::io::Read;
use std::path::{Path, PathBuf};
use thiserror::Error;
use utils::{DirectoryError, cache_path, config_path};

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
    registered_packs: HashMap<String, AssetLibraryEntry>,
    /// A map of currently loaded asset packs, keyed by their (public) identifiers.
    ///
    /// Given that this map is only persisted for the runtime of the application,
    /// it's possible that an [`AssetPack`] is 'known' but not loaded.
    #[serde(skip)]
    loaded_packs: HashMap<String, AssetPack>,
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
    /// An error occurred while (de)serialising the library configuration.
    #[error("failed to (de)serialize library configuration")]
    Serialisation(#[from] serialization::SerializationError),
    /// Wrapper for the [`AssetPackError`].
    #[error(transparent)]
    AssetPack(#[from] AssetPackError),
}

impl Default for AssetLibrary {
    fn default() -> Self {
        Self {
            version: utils::version().clone(),
            registered_packs: HashMap::new(),
            loaded_packs: HashMap::new(),
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
            .map_err(AssetLibraryError::Serialisation)
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
        let file =
            File::create(path.join(LIBRARY_FILE_NAME)).map_err(AssetLibraryError::ReadFile)?;
        serialize_to(self, &SerializationFormat::Toml, &file)?;
        Ok(())
    }

    /// Registers a new [`AssetPack`] in the library and returns the [`AssetPack`]'s ID.
    ///
    /// Note that you should only use this to create a *new* pack, not to load an existing one.
    ///
    /// # Errors
    /// - The configuration folder could not be retrieved: [`AssetLibraryError::LocateConfigFolder`]
    /// - An error occurs while trying to read the config file (doesn't exist, permissions, ...):
    ///   [`AssetLibraryError::LocateConfigFolder`]
    /// - The file was found, could be read but failed to deserialize: [`AssetLibraryError::Serialization`].
    pub fn add_pack(
        &mut self,
        root: &Path,
        name: Option<String>,
    ) -> Result<String, AssetLibraryError> {
        let meta_dir = cache_path()?;
        let pack = AssetPack::new(root, meta_dir.as_path(), name)?;
        let pack_id = pack.id.clone();
        let entry = AssetLibraryEntry {
            root: root.to_path_buf(),
            index: meta_dir.clone(),
        };

        self.registered_packs.insert(pack_id.clone(), entry);
        self.loaded_packs.insert(pack_id.clone(), pack);
        Ok(pack_id)
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
    #![allow(clippy::missing_panics_doc)]
    #![allow(clippy::missing_errors_doc)]

    use super::*;

    #[test]
    fn add_pack_creates_asset_pack() -> anyhow::Result<()> {
        let tmp = tempfile::tempdir()?;
        let mut library = AssetLibrary::default();
        let pack_id = library.add_pack(tmp.path(), None)?;

        assert_eq!(library.registered_packs.len(), 1);
        assert!(library.registered_packs.contains_key(&pack_id));
        Ok(())
    }
}
