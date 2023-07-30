use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

use chrono::prelude::*;

/// Represents a Janitor job.
///
/// This bundles together the data needed to execute a janitor job for a
/// particular profile path.
#[derive(Debug, PartialEq, Eq)]
pub struct Job<T> {
    path: PathBuf,
    keep_since: NaiveDateTime,
    keep_at_least: usize,
    data: T,
}

impl<T> Job<T> {
    /// Creates a new Job instance.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the profile to clean up  
    /// * `keep_since` - The cutoff date for keeping generations
    /// * `keep_at_least` - The minimum number of generations to keep  
    /// * `data` - The data for this job
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use chrono::NaiveDateTime;
    /// use janitor::Job;
    ///
    /// let job = Job::new(
    ///     PathBuf::from("/some/path"),
    ///     NaiveDateTime::from_timestamp(0, 0),
    ///     5,
    ///     "data".to_string(),
    /// );
    /// ```
    pub fn new<P: AsRef<Path>>(
        path: P,
        keep_since: NaiveDateTime,
        keep_at_least: usize,
        data: T,
    ) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            keep_since,
            keep_at_least,
            data,
        }
    }

    /// Returns a reference to the path field.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use janitor::Job;
    ///
    /// let job = Job::new(PathBuf::new(), Default::default(), 0, ());  
    /// assert_eq!(job.path(), &PathBuf::new());
    /// ```
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Returns the keep_since date for this job.
    ///
    /// This is the cutoff date for keeping generations - any generations
    /// active on or after this date will be kept.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::NaiveDateTime;
    /// use janitor::Job;
    ///
    /// let job = Job::new("/", NaiveDateTime::from_timestamp(0, 0), 0, ());
    /// assert_eq!(job.keep_since(), NaiveDateTime::from_timestamp(0, 0));
    /// ```
    pub fn keep_since(&self) -> NaiveDateTime {
        self.keep_since
    }

    /// Returns the minimum number of generations to keep.
    ///
    /// This is the lower bound for how many recent generations should be
    /// retained during cleanup.
    ///
    /// # Examples
    ///
    /// ```
    /// use janitor::Job;
    ///
    /// let job = Job::new("/", Default::default(), 5, ());
    /// let min = job.keep_at_least();
    /// assert_eq!(min, 5);
    /// ```
    pub fn keep_at_least(&self) -> usize {
        self.keep_at_least
    }

    /// Returns a reference to the data field.
    ///
    /// The data can be any generic type T.
    ///
    /// # Examples
    ///
    /// ```
    /// use janitor::Job;
    ///
    /// let job = Job::new("/", Default::default(), 0, "data".to_string());
    /// let data = job.data();
    /// assert_eq!(data, &"data".to_string());
    /// ```
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Replaces the data in the Job with new data of type `U` and returns a new Job instance.
    ///
    /// # Arguments
    ///
    /// * `data` - The new data to assign to the Job. This can be any type `U`.
    ///
    /// # Returns
    ///  
    /// A new `Job<U>` instance with the same configuration but the new `data` value.
    ///
    /// # Examples
    ///
    /// ```
    /// use janitor::Job;
    ///
    /// let original = Job::new("/", Default::default(), 0, 1);
    /// let updated = original.set_data("new data");
    ///
    /// assert_eq!(updated.data(), &"new data");
    /// ```
    pub fn set_data<U>(&self, data: U) -> Job<U> {
        Job {
            path: self.path.clone(),
            keep_since: self.keep_since,
            keep_at_least: self.keep_at_least,
            data,
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use chrono::prelude::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn path_remains_unchanged(path in "(/[a-z]+)+") {
            let job = super::Job::new(&path, Default::default(), 0, ());
            prop_assert_eq!(job.path().as_path(), Path::new(&path));
        }

        #[test]
        fn keep_since_remains_unchanged(timestamp in 0..100_000_000i64) {
            let date = NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap();
            let job = super::Job::new("/", date, 0, ());
            prop_assert_eq!(job.keep_since(), date);
        }

        #[test]
        fn keep_at_least_remains_unchanged(min in 0..100usize) {
            let job = super::Job::new("/", Default::default(), min, ());
            prop_assert_eq!(job.keep_at_least(), min);
        }

        #[test]
        fn data_remains_unchanged(data in "[a-z]+") {
            let job = super::Job::new("/", Default::default(), 0, data.clone());
            prop_assert_eq!(job.data(), &data);
        }

        #[test]
        fn after_data_update_anything_else_remains_intact_but_data_changed(
            path in "(/[a-z]+)+",
            timestamp in 0..100_000_000i64,
            min in 0..100usize,
            init_data in "[a-z]+",
            new_data in 0..100_000_000usize,
        ) {
            let date = NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap();
            let job = super::Job::new(path, date, min, init_data.clone());
            let updated = job.set_data(new_data);
            prop_assert_eq!(updated.path(), job.path());
            prop_assert_eq!(updated.keep_since(), job.keep_since());
            prop_assert_eq!(updated.keep_at_least(), job.keep_at_least());
            prop_assert_eq!(job.data(), &init_data);
            prop_assert_eq!(updated.data(), &new_data);
        }
    }
}
