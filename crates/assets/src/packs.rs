//! An asset pack is a single root folder that contains asset and subfolders.

use bevy::prelude::Component;
use serialization::{Deserialize, SerializationFormat, Serialize, deserialize, serialize_to};
use std::collections::HashMap;
use std::fs::File;
use std::io::read_to_string;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;
use utils::file_name;

/// The filename of the asset pack manifests.
const MANIFEST_FILE_NAME: &str = "asset_pack.toml";

/// An [`AssetPack`] is a single root folder that contains assets and subfolders.
///
/// The asset pack handles the indexing, categorising and loading the assets.
#[derive(Component, Debug)]
pub struct AssetPack {
    /// The state the pack is currently in.
    ///
    /// This is used to track whether a pack needs to perform operations to be usable, whether some
    /// operations failed and so forth.
    pub state: AssetPackState,
    /// The identifier of this string, usually a hash or short ID defined by the creator of the asset
    /// pack represented.
    ///
    /// This ID is used when referring to files under [`AssetPack::root`].
    pub id: String,
    /// The human-readable name of this [`AssetPack`].
    ///
    /// This is not guaranteed to be unique! If you need to identify this pack, please use [`AssetPack::id`].
    pub name: String,
    /// The "root" directory under which the assets live for this pack.
    ///
    /// This is used internally to generate relative paths (that are portable) from absolute paths
    /// used in the asset loader.
    pub root: PathBuf,
    /// The directory in which metadata about the [`AssetPack`] is kept.
    /// This ranges from index metadata, scripts to thumbnails, this directory is not guaranteed to
    /// exist between runs and may be cleaned to recover disk space. The operations in this directory
    /// should be ephemeral by design.
    pub meta_dir: PathBuf,
    /// Internal mapping table between asset identifiers and their physical paths.
    ///
    /// Each path value is relative to the [`AssetPack::root`].
    index: HashMap<String, PathBuf>,
    /// A [Rhai](https://rhai.rs/) script that is used during indexing operations to assist in categorising
    /// the assets in the pack.
    script: Option<String>,
}

/// Internal "copy" of the [`AssetPack`] struct intended for saving/loading to disk.
#[derive(Serialize, Deserialize, Debug)]
#[allow(
    clippy::missing_docs_in_private_items,
    reason = "Copied from the original struct"
)]
struct _AssetPack {
    pub id: String,
    pub name: String,
    index: HashMap<String, PathBuf>,
    script: Option<String>,
}

/// Describes the current state of an [`AssetPack`].
#[derive(Default, Debug, Serialize, Deserialize)]
pub enum AssetPackState {
    /// The asset pack was just created, no validation or checks to its current state have been made.
    /// Additional processing is required to validate the pack's state before it can be used.
    #[default]
    Created,
    /// The asset pack is currently (re)indexing its contents.
    Indexing,
    /// Something went wrong during processing, leaving this pack in an invalid state.
    Invalid(String),
    /// The pack is ready to use.
    Ready,
}

/// Describes the errors that can occur when working with [`AssetPack`]s
#[derive(Error, Debug)]
pub enum AssetPackError {
    /// Thrown when creating/opening the asset pack manifest fails.
    #[error("An IO error occurred while reading/writing the asset pack manifest")]
    ManifestFile(#[from] std::io::Error),
    /// Thrown when the serialisation of an asset pack manifest fails.
    #[error("An error occurred while serialising the asset pack manifest")]
    Serialisation(#[from] serialization::SerializationError),
}

impl AssetPack {
    /// Generate a new [`AssetPack`] in the [`AssetPackState::Created`] state.
    ///
    /// # Errors
    /// This method may return an error if it fails to [canonicalize](https://doc.rust-lang.org/std/fs/fn.canonicalize.html)
    /// the root path.
    pub fn new(root: &Path, meta_dir: &Path, name: Option<String>) -> Result<Self, AssetPackError> {
        let root = root.canonicalize()?;
        let id = blake3::hash(root.as_os_str().as_encoded_bytes()).to_string();
        let name = name
            .or_else(|| file_name(&root))
            .unwrap_or_else(|| id.clone());

        Ok(Self {
            state: AssetPackState::Created,
            id: id.clone(),
            name,
            root,
            meta_dir: meta_dir.to_path_buf(),
            index: HashMap::new(),
            script: None,
        })
    }

    /// Attempts to save the manifest for this [`AssetPack`] to disk.
    /// The resulting file will be written under [`AssetPack::root`].
    ///
    /// # Errors
    /// - [`AssetPackError::ManifestFile`] when the file/folder for the manifest couldn't be created.
    /// - [`AssetPackError::Serialisation`] when serialising the manifest fails.
    pub fn save_manifest(&self) -> Result<(), AssetPackError> {
        let config = _AssetPack::from(self);
        let manifest = self.root.join(MANIFEST_FILE_NAME);
        let manifest = File::create(manifest).map_err(AssetPackError::ManifestFile)?;

        serialize_to(&config, &SerializationFormat::Toml, manifest)
            .map_err(AssetPackError::Serialisation)
    }

    /// Attempts to load an [`AssetPack`] from its manifest in the `root` folder.
    /// The resulting [`AssetPack`] will always be in [`AssetPackState::Crated`].
    ///
    /// # Errors
    /// - [`AssetPackError::ManifestFile`] when the file/folder for the manifest couldn't be opened.
    /// - [`AssetPackError::Serialisation`] when serialising the manifest fails.
    pub fn load_manifest(root: &Path, meta_dir: &Path) -> Result<Self, AssetPackError> {
        let manifest = root.join(MANIFEST_FILE_NAME);
        let manifest = File::open(manifest).map_err(AssetPackError::ManifestFile)?;
        let manifest = read_to_string(manifest).map_err(AssetPackError::ManifestFile)?;

        let manifest: _AssetPack = deserialize(manifest.as_bytes(), &SerializationFormat::Toml)?;
        Ok(Self {
            state: AssetPackState::Created,
            id: manifest.id,
            name: manifest.name,
            root: root.to_path_buf(),
            meta_dir: meta_dir.to_path_buf(),
            index: manifest.index,
            script: manifest.script,
        })
    }
}

impl From<&AssetPack> for _AssetPack {
    fn from(pack: &AssetPack) -> Self {
        Self {
            id: pack.id.clone(),
            name: pack.name.clone(),
            index: pack.index.clone(),
            script: pack.script.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::missing_panics_doc)]
    #![allow(clippy::missing_errors_doc)]

    use super::*;
    use tempfile::tempdir;

    #[test]
    fn new_asset_pack_id_is_stable() {
        let path = Path::new(".");
        let pack = AssetPack::new(path, path, None).unwrap();
        let pack2 = AssetPack::new(path, path, None).unwrap();

        assert_eq!(pack.id, pack2.id);
    }

    #[test]
    fn new_asset_pack_id_unique() -> anyhow::Result<()> {
        let path1 = tempdir()?;
        let path2 = tempdir()?;
        let pack1 = AssetPack::new(path1.path(), path1.path(), None)?;
        let pack2 = AssetPack::new(path2.path(), path2.path(), None)?;

        assert_ne!(pack1.id, pack2.id);
        Ok(())
    }

    #[test]
    #[should_panic = "Should fail to create asset pack"]
    fn new_asset_error_on_invalid_path() {
        let path = Path::new("./does/not/exist");
        AssetPack::new(path, path, None).expect("Should fail to create asset pack");
    }
}
