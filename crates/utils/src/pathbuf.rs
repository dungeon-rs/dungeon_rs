//! Various helper methods for working with Rust's [`PathBuf`].

use std::path::Path;

/// Retrieves the file name from the given [`PathBuf`].
///
/// This method is lossy in case of invalid UTF-8 characters, see [`OsStr::to_string_lossy`]
#[inline]
#[must_use]
pub fn file_name(path: &Path) -> Option<String> {
    Some(path.file_name()?.to_string_lossy().to_string())
}
