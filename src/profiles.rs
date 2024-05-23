use std::{
    fmt,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use eyre::Result;
use uzers::{Users, UsersCache};

use crate::user;

/// Represents a Nix profile path.
///
/// This wraps a [std::path::PathBuf] to provide a named type.
pub struct Profile<U>(PathBuf, Arc<Mutex<U>>)
where
    U: Users;

#[cfg(not(tarpaulin_include))]
impl<U> fmt::Debug for Profile<U>
where
    U: Users,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Profile")
            .field(&self.0)
            .field(&format_args!("_"))
            .finish()
    }
}

#[cfg(not(tarpaulin_include))]
impl Profile<UsersCache> {
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
        Self(path.into(), Arc::new(Mutex::new(UsersCache::new())))
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
        let users = Arc::new(Mutex::new(UsersCache::new()));

        Self::all_with_users(users)
    }
}

impl<U> Profile<U>
where
    U: Users,
{
    fn all_with_users(users: Arc<Mutex<U>>) -> Vec<Self> {
        let ctx = |s: &str| context(s, users.clone());

        let mut paths = vec![
            "/nix/var/nix/profiles/per-user/$USER/profile",
            "/home/$USER/.local/state/nix/profiles/home-manager",
        ];

        if user::is_root(users.clone()) {
            paths.push("/nix/var/nix/profiles/system");
        }

        paths
            .iter()
            .map(|p| -> Result<_> { Ok(shellexpand::env_with_context(p, ctx).unwrap()) })
            .map(|p| -> Result<_> { Ok(PathBuf::from(p?.to_string())) })
            .filter_map(|pr| pr.ok())
            // .filter(|p| p.exists())
            .map(|p| Self(p, users.clone()))
            .collect::<Vec<_>>()
    }
}

impl<U> AsRef<Path> for Profile<U>
where
    U: Users,
{
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

fn context<U>(s: &str, users: Arc<Mutex<U>>) -> Result<Option<String>>
where
    U: Users,
{
    match s {
        "USER" => Ok(user::get_real_username(users.clone())),
        v => Err(eyre::eyre!("unknown variable: {v}")),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use proptest::prelude::*;
    use temp_env::with_vars;
    use uzers::mock::MockUsers;

    proptest! {
        #[test]
        fn new(path in "(/[a-z]+)+") {
            let path = PathBuf::from(&path);
            let profile = Profile::new(&path);
            prop_assert_eq!(profile.0, path);
        }

        #[test]
        fn test_being_regular_user(uid in 1u32..20_000, username in "[a-z]+") {
            let users = Arc::new(Mutex::new(MockUsers::with_current_uid(uid)));

            let profiles = with_vars([("USER", Some(&username))], || Profile::all_with_users(users.clone()));

            let paths = profiles.iter().map(|p| PathBuf::from(p.as_ref())).collect::<Vec<_>>();

            let user_profile = PathBuf::from(format!("/nix/var/nix/profiles/per-user/{}/profile", username));
            let home_profile = PathBuf::from(format!("/home/{}/.local/state/nix/profiles/home-manager", username));

            prop_assert_eq!(paths, vec![user_profile, home_profile]);
        }

        #[test]
        fn test_being_sudo_user(username in "[a-z]+") {
            let users = Arc::new(Mutex::new(MockUsers::with_current_uid(0)));

            let profiles = with_vars([("SUDO_USER", Some(&username)), ("USER", Some(&"root".to_owned()))], || Profile::all_with_users(users.clone()));

            let paths = profiles.iter().map(|p| PathBuf::from(p.as_ref())).collect::<Vec<_>>();

            let user_profile = PathBuf::from(format!("/nix/var/nix/profiles/per-user/{}/profile", username));
            let home_profile = PathBuf::from(format!("/home/{}/.local/state/nix/profiles/home-manager", username));

            prop_assert_eq!(paths, vec![user_profile, home_profile, PathBuf::from("/nix/var/nix/profiles/system")]);
        }
    }
}
