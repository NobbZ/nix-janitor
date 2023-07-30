use std::{
    env,
    path::{Path, PathBuf},
};

use eyre::Result;

/// Represents a Nix profile path.
///
/// This wraps a [std::path::PathBuf] to provide a named type.
#[derive(Debug)]
pub struct Profile(PathBuf);

impl Profile {
    /// Creates a new Profile from the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the profile.
    ///
    /// # Examples
    ///
    /// ```
    /// use janitor::Profile;
    ///
    /// let profile = Profile::new("/foo/bar");
    /// ```
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self(path.into())
    }

    /// Returns all default profile paths for the current user.
    ///
    /// This discovers the Nix profile paths by detecting if running as root/sudo,
    /// and expanding environment variables.
    ///
    /// # Examples
    ///
    /// ```
    /// use janitor::Profile;
    /// let profiles = Profile::all();
    /// ```
    pub fn all() -> Vec<Self> {
        let mut paths = vec![
            "/nix/var/nix/profiles/per-user/$USER/profile",
            "/home/$USER/.local/state/nix/profiles/home-manager",
        ];

        if is_root::is_root() {
            paths.push("/nix/var/nix/profiles/system");
        }

        paths
            .iter()
            .map(|p| -> Result<_> { Ok(shellexpand::env_with_context(p, context).unwrap()) })
            .map(|p| -> Result<_> { Ok(PathBuf::from(p?.to_string())) })
            .filter_map(|pr| pr.ok())
            .filter(|p| p.exists())
            .map(Self::new)
            .collect::<Vec<_>>()
    }
}

impl AsRef<Path> for Profile {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

fn context(s: &str) -> Result<Option<String>> {
    match s {
        "USER" => Ok(get_username()),
        v => Err(eyre::eyre!("unknown variable: {v}")),
    }
}

fn get_username() -> Option<String> {
    if is_root::is_root() {
        tracing::debug!("running as root, using SUDO_USER");
        env::var("SUDO_USER").ok()
    } else {
        tracing::debug!("running regular user, using USER");
        env::var("USER").ok()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn new(path in "(/[a-z]+)+") {
            let path = PathBuf::from(&path);
            let profile = Profile::new(&path);
            assert_eq!(profile.0, path);
        }
    }

    // TODO: provide some tests for Profile::all()
}
