pub fn optional<T>(condition: bool, value: T) -> Option<T> {
    match condition {
        true => Some(value),
        false => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn true_gets_value_wrapped(v in -10_000i32..10_000) {
            let option = optional(true, v);

            prop_assert!(option.is_some());
            prop_assert_eq!(option, Some(v));
        }

        #[test]
        fn false_gets_none(v in -10_000i32..10_000) {
            let option = optional(false, v);

            prop_assert!(option.is_none());
        }
    }
}
