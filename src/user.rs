use std::env;
use std::sync::{Arc, Mutex};

use uzers::Users;

pub fn is_root<U>(users: Arc<Mutex<U>>) -> bool
where
    U: Users,
{
    users.lock().unwrap().get_current_uid() == 0
}

pub fn get_real_username<U>(users: Arc<Mutex<U>>) -> Option<String>
where
    U: Users,
{
    if is_root(users) {
        tracing::debug!("running as root, using SUDO_USER");
        env::var("SUDO_USER").ok()
    } else {
        tracing::debug!("running regular user, using USER");
        env::var("USER").ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use proptest::prelude::*;
    use temp_env::with_vars;
    use uzers::mock::MockUsers;

    proptest! {
        #[test]
        fn test_is_root(uid in 1u32..20_000) {
            let users = Arc::new(Mutex::new(MockUsers::with_current_uid(uid)));

            prop_assert!(!is_root(users));
        }

        #[test]
        fn test_get_real_username(uid in 1u32..20_000, username in "[a-z]+") {
            let users = Arc::new(Mutex::new(MockUsers::with_current_uid(uid)));

            let real_username = with_vars([("USER", Some(&username))], || get_real_username(users));

            prop_assert_eq!(real_username, Some(username.to_string()));
        }

        #[test]
        fn test_get_real_username_root(username in "[a-z]+") {
            let users = Arc::new(Mutex::new(MockUsers::with_current_uid(0)));

            let real_username = with_vars([("SUDO_USER", Some(&username)), ("USER", Some(&"root".to_owned()))], || get_real_username(users));

            assert_eq!(real_username, Some(username.to_string()));
        }
    }

    #[test]
    fn test_is_root_root() {
        let users = Arc::new(Mutex::new(MockUsers::with_current_uid(0)));

        assert!(is_root(users));
    }
}
