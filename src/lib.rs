#![warn(clippy::pedantic)]

use std::borrow::Cow;
use std::fmt;
use std::path::{Path, PathBuf};

use crate::sealed::Sealed;

const TILDE: &str = "~";

pub fn expand_tilde_with<P, H>(path: &P, home_dir: H) -> Cow<'_, Path>
where
    P: AsRef<Path> + ?Sized,
    H: AsRef<Path>,
{
    fn inner<'a>(path: &'a Path, home_dir: &Path) -> Cow<'a, Path> {
        path.strip_prefix(TILDE).map_or_else(
            |_| Cow::Borrowed(path),
            |stripped| Cow::Owned(home_dir.join(stripped)),
        )
    }

    inner(path.as_ref(), home_dir.as_ref())
}

pub fn expand_tilde<P>(path: &P) -> Result<Cow<'_, Path>, HomeDirError>
where
    P: AsRef<Path> + ?Sized,
{
    let home_dir = home_dir()?;
    Ok(expand_tilde_with(path, home_dir))
}

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

pub trait ExpandTilde: Sealed {
    fn expand_tilde_with<H: AsRef<Path>>(&self, home_dir: H) -> Cow<'_, Path>;
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
    Empty,
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
    }
}
