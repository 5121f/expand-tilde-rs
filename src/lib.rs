/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Expanding tilde (`~`) into the user’s home directory.
//! Useful for working with shell-like paths such as `~/projects`.
//! Works both as free functions and as a trait [`ExpandTilde`] on [`Path`]
//!
//! ## Example
//!
//! ```rust
//! use zeroten_expand_tilde::ExpandTilde;
//! use std::path::Path;
//!
//! let path = Path::new("~/dir");
//! let expanded = path.expand_tilde().unwrap();
//! println!("{}", expanded.display()); // Something like `/home/user/dir`
//! ```

#![warn(clippy::pedantic)]

use std::borrow::Cow;
use std::fmt;
use std::path::{Path, PathBuf};

use crate::sealed::Sealed;

const TILDE: &str = "~";

/// Expands the leading tilde (`~`) in a path using the provided `home_dir`.
///
/// If the path starts with `~`, it is replaced with the given `home_dir`.
/// Otherwise, the original path is returned unchanged.
///
/// # Example
///
/// ```rust
/// use zeroten_expand_tilde::expand_tilde_with;
/// use std::path::{Path, PathBuf};
///
/// let home = "/home/user";
/// let path = Path::new("~/docs");
/// assert_eq!(
///     expand_tilde_with(path, home),
///     PathBuf::from("/home/user/docs")
/// );
/// ```
pub fn expand_tilde_with<P, H>(path: &P, home_dir: H) -> Cow<'_, Path>
where
    P: AsRef<Path> + ?Sized,
    H: AsRef<Path>,
{
    fn inner<'a>(path: &'a Path, home_dir: &Path) -> Cow<'a, Path> {
        path.strip_prefix(TILDE)
            .map_or_else(|_| path.into(), |stripped| home_dir.join(stripped).into())
    }

    inner(path.as_ref(), home_dir.as_ref())
}

/// Expands the leading tilde (`~`) in a path using the current user’s home directory.
///
/// By default, this uses [`std::env::home_dir()`], which is the recommended
/// approach in modern Rust versions.
///
/// If the `compat` feature is enabled, the [`home`] crate is used instead.
///
/// For a detailed discussion of the differences and platform-specific
/// behavior, see the [`home`] crate:
/// <https://crates.io/crates/home>
///
/// If you need to expand multiple paths, prefer using
/// [`expand_tilde_with`] with a cached home directory to avoid
/// calling [`home_dir`] repeatedly.
///
/// # Errors
///
/// - [`HomeDirError::NotFounded`] if the home directory cannot be determined
/// - [`HomeDirError::Empty`] if the home directory is empty
pub fn expand_tilde<P>(path: &P) -> Result<Cow<'_, Path>, HomeDirError>
where
    P: AsRef<Path> + ?Sized,
{
    let home_dir = home_dir()?;
    Ok(expand_tilde_with(path, home_dir))
}

/// Returns the current user’s home directory.
///
/// By default, this uses [`std::env::home_dir()`], which is the recommended
/// approach in modern Rust versions.
///
/// If the `compat` feature is enabled, the [`home`] crate is used instead.
///
/// For a detailed discussion of the differences and platform-specific
/// behavior, see the [`home`] crate:
/// <https://crates.io/crates/home>
///
/// # Errors
///
/// - [`HomeDirError::NotFounded`] if the home directory cannot be determined
/// - [`HomeDirError::Empty`] if the home directory is empty
pub fn home_dir() -> Result<PathBuf, HomeDirError> {
    #[cfg(feature = "compat")]
    let home_dir = home::home_dir().ok_or(HomeDirError::NotFounded)?;

    #[cfg(not(feature = "compat"))]
    let home_dir = std::env::home_dir().ok_or(HomeDirError::NotFounded)?;

    if home_dir.as_os_str().is_empty() {
        return Err(HomeDirError::Empty);
    }

    Ok(home_dir)
}

mod sealed {
    pub trait Sealed {}
}

/// A trait for expanding tildes.
pub trait ExpandTilde: Sealed {
    /// Expands the leading tilde (`~`) in a path using the provided `home_dir`.
    ///
    /// If the path starts with `~`, it is replaced with the given `home_dir`.
    /// Otherwise, the original path is returned unchanged.
    ///
    /// # Example
    ///
    /// ```rust
    /// use zeroten_expand_tilde::ExpandTilde;
    /// use std::path::{Path, PathBuf};
    ///
    /// let home = "/home/user";
    /// let path = Path::new("~/docs");
    /// assert_eq!(
    ///     path.expand_tilde_with(home),
    ///     PathBuf::from("/home/user/docs")
    /// );
    /// ```
    fn expand_tilde_with<H: AsRef<Path>>(&self, home_dir: H) -> Cow<'_, Path>;

    /// Expands the leading tilde (`~`) in a path using the current user’s home directory.
    ///
    /// By default, this uses [`std::env::home_dir()`], which is the recommended
    /// approach in modern Rust versions.
    ///
    /// If the `compat` feature is enabled, the [`home`] crate is used instead.
    ///
    /// For a detailed discussion of the differences and platform-specific
    /// behavior, see the [`home`] crate:
    /// <https://crates.io/crates/home>
    ///
    /// If you need to expand multiple paths, prefer using
    /// [`ExpandTilde::expand_tilde_with`] with a cached home directory to avoid
    /// calling [`home_dir`] repeatedly.
    ///
    /// # Errors
    ///
    /// - [`HomeDirError::NotFounded`] if the home directory cannot be determined
    /// - [`HomeDirError::Empty`] if the home directory is empty
    fn expand_tilde(&self) -> Result<Cow<'_, Path>, HomeDirError>;
}

impl ExpandTilde for Path {
    fn expand_tilde_with<H: AsRef<Path>>(&self, home_dir: H) -> Cow<'_, Path> {
        expand_tilde_with(self, home_dir)
    }

    fn expand_tilde(&self) -> Result<Cow<'_, Path>, HomeDirError> {
        expand_tilde(self)
    }
}

impl Sealed for Path {}

#[derive(Debug, Clone)]
pub enum HomeDirError {
    /// The home directory is empty
    Empty,
    /// The home directoy not founed
    NotFounded,
}

impl fmt::Display for HomeDirError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HomeDirError::Empty => write!(f, "the home directory is empty"),
            HomeDirError::NotFounded => write!(f, "the home directoy not founed"),
        }
    }
}

impl std::error::Error for HomeDirError {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(
            PathBuf::from("/home/user/some/dir"),
            expand_tilde_with("~/some/dir", "/home/user").into_owned()
        );
        assert_eq!(
            PathBuf::from("some/dir"),
            expand_tilde_with("some/dir", "/home/user").into_owned()
        );
        assert_eq!(
            expand_tilde_with("~", "/home/user"),
            PathBuf::from("/home/user")
        );
    }
}
