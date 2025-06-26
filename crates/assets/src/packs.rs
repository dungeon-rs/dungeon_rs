//! An asset pack is a single root folder that contains asset and subfolders.

use bevy::prelude::{Component, default};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;
use utils::file_name;

/// An [`AssetPack`] is a single root folder that contains assets and subfolders.
///
/// The asset pack handles the indexing, categorising and loading the assets.
#[derive(Component)]
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
    root: PathBuf,
    /// The directory in which metadata about the [`AssetPack`] is kept.
    /// This ranges from index metadata, scripts to thumbnails, this directory is not guaranteed to
    /// exist between runs and may be cleaned to recover disk space. The operations in this directory
    /// should be ephemeral by design.
    ///
    /// By default, this is embedded in a hidden subfolder of the [`AssetPack::root`] itself.
    meta_dir: PathBuf,
    /// Internal mapping table between asset identifiers and their physical paths.
    ///
    /// Each path value is relative to the [`AssetPack::root`].
    index: HashMap<String, PathBuf>,
    /// A [Rhai](https://rhai.rs/) script that is used during indexing operations to assist in categorising
    /// the assets in the pack.
    script: Option<String>,
}

/// Describes the current state of an [`AssetPack`].
#[derive(Default, Debug)]
pub enum AssetPackState {
    #[default]
    Created,
    Indexing,
    Invalid(String),
    Ready,
}

#[derive(Error, Debug)]
pub enum AssetPackError {}

impl AssetPack {
    /// Generate a new [`AssetPack`] in the [`AssetPackState::Created`] state.
    pub fn new(root: PathBuf, name: Option<String>) -> Result<Self, AssetPackError> {
        let meta_dir = root.join(".metadata");
        let id = blake3::hash(root.as_os_str().as_encoded_bytes()).to_string();
        let name = name
            .or_else(|| file_name(&root))
            .unwrap_or_else(|| id.clone());

        Ok(Self {
            state: default(),
            id: id.clone(),
            name,
            root,
            meta_dir,
            index: HashMap::new(),
            script: None,
        })
    }
}
