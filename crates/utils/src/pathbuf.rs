//! Various helper methods for working with Rust's [`PathBuf`].

use std::path::PathBuf;

/// Retrieves the file name from the given [`PathBuf`].
///
/// This method is lossy in case of invalid UTF-8 characters, see [`OsStr::to_string_lossy`]
#[inline]
pub fn file_name(path: &PathBuf) -> Option<String> {
    let Some(file_name) = path.file_name() else {
        return None;
    };

    Some(file_name.to_string_lossy().to_string())
}