use uzers::Users;

pub fn is_root<U>(users: &mut U) -> bool
where
    U: Users,
{
    users.get_current_uid() == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    use proptest::prelude::*;
    use uzers::mock::MockUsers;

    proptest! {
        #[test]
        fn test_is_root(uid in 1u32..20_000) {
            let mut users = MockUsers::with_current_uid(uid);

            prop_assert!(!is_root(&mut users));
        }
    }

    #[test]
    fn test_is_root_root() {
        let mut users = MockUsers::with_current_uid(0);

        assert!(is_root(&mut users));
    }
}
