use std::collections::BTreeSet;

use chrono::prelude::*;

use crate::generation::Generation;

/// Represents a set of [Generation]s.
///
/// The generations are stored in a [BTreeSet] and kept in order by
/// [Generation::id].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationSet {
    generations: BTreeSet<Generation>,
}

impl GenerationSet {
    /// Returns a new [GenerationSet] containing only the `n` most recent
    /// [Generation]s in this set.
    ///
    /// The generations are sorted by [Generation::id] in descending order, so
    /// the highest ids are first.
    ///
    /// If `n` is greater than or equal to the number of generations in this set,
    /// a clone of this entire set is returned.  
    ///
    /// # Arguments
    ///
    /// * `n` - The number of recent generations to return
    ///
    /// # Examples
    ///
    /// ```
    /// use janitor::{Generation, GenerationSet};
    /// use chrono::prelude::*;
    ///
    /// let date = NaiveDateTime::from_timestamp_opt(0, 0).unwrap();
    ///
    /// let generations = vec![
    ///     Generation { id: 1, current: false, date },
    ///     Generation { id: 2, current: false, date },
    ///     Generation { id: 3, current: false, date },
    /// ].into_iter().collect::<GenerationSet>();
    ///
    /// let recent = generations.get_last_n_generations(2);
    /// assert_eq!(recent.len(), 2);
    /// assert_eq!(recent.iter().map(|g| g.id).collect::<Vec<_>>(), vec![2, 3]);
    /// ```
    pub fn get_last_n_generations(&self, n: usize) -> Self {
        let mut generations = self.generations.iter().cloned().collect::<Vec<_>>();

        generations.sort_by(|a, b| a.id.cmp(&b.id));

        if n >= generations.len() {
            return generations.into();
        }

        generations[generations.len() - n..].into()
    }

    /// Returns a new [GenerationSet] containing the active generation on or after
    /// the provided `date`, along with any newer generations.
    ///
    /// The result will include the last generation before `date` as it potentially
    /// has been active on `date`.
    ///
    /// # Arguments
    ///
    /// * `date` - The cutoff date. The returned set will contain any generation
    ///   that has been potentially active on or after this point in time.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::NaiveDateTime;  
    /// use janitor::{Generation, GenerationSet};
    ///
    /// let date1 = NaiveDateTime::parse_from_str("2020-01-01 00:00", "%Y-%m-%d %H:%M").unwrap();
    /// let date2 = NaiveDateTime::parse_from_str("2020-02-01 00:00", "%Y-%m-%d %H:%M").unwrap();
    /// let cutoff = NaiveDateTime::parse_from_str("2020-02-02 00:00", "%Y-%m-%d %H:%M").unwrap();
    ///
    /// let generations = vec![
    ///     Generation { id: 1, date: date1, current: false },
    ///     Generation { id: 2, date: date2, current: false },
    /// ].into_iter().collect::<GenerationSet>();
    ///
    /// let active = generations.get_active_on_or_after(cutoff);
    /// assert_eq!(active.len(), 1);
    /// assert_eq!(active.iter().next().unwrap().id, 2);
    /// ```
    pub fn get_active_on_or_after(&self, date: NaiveDateTime) -> Self {
        let (newer, older): (Vec<_>, _) = self.iter().partition(|g| g.date >= date);

        older
            .iter()
            .last()
            .map_or_else(
                || newer.clone(),
                |last| {
                    let mut result = vec![*last];
                    result.extend_from_slice(&newer);
                    result
                },
            )
            .into()
    }

    /// Returns a new [GenerationSet] containing generations that should be deleted.
    ///
    /// The returned set will contain all generations except:
    ///
    /// - The `keep` most recent generations based on [Generation::id].
    /// - Any generations active on or after `date`.
    ///
    /// # Arguments
    ///
    /// * `keep` - The number of recent generations to keep.
    /// * `date` - The cutoff date. Generations active on or after this will be kept.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::NaiveDateTime;
    /// use janitor::{Generation, GenerationSet};
    ///  
    /// let date1 = NaiveDateTime::parse_from_str("2020-01-01 00:00", "%Y-%m-%d %H:%M").unwrap();
    /// let date2 = NaiveDateTime::parse_from_str("2020-02-01 00:00", "%Y-%m-%d %H:%M").unwrap();
    /// let date3 = NaiveDateTime::parse_from_str("2020-03-01 00:00", "%Y-%m-%d %H:%M").unwrap();
    ///
    /// let threshold = NaiveDateTime::parse_from_str("2020-02-02 00:00", "%Y-%m-%d %H:%M").unwrap();
    ///
    /// let generations = vec![
    ///     Generation { id: 1, date: date1, current: false }, // delete
    ///     Generation { id: 2, date: date2, current: false }, // keep (because of date)
    ///     Generation { id: 3, date: date3, current: false }, // keep (recent)
    /// ].into_iter().collect::<GenerationSet>();
    ///
    /// let to_delete = generations.generations_to_delete(1, threshold);
    /// assert_eq!(to_delete.len(), 1);
    /// assert_eq!(to_delete.iter().next().unwrap().id, 1);
    /// ```
    pub fn generations_to_delete(&self, keep: usize, date: NaiveDateTime) -> Self {
        let by_count = self.get_last_n_generations(keep).generations;

        let by_date = self.get_active_on_or_after(date).generations;

        let to_keep = by_count
            .union(&by_date)
            .cloned()
            .collect::<BTreeSet<Generation>>();

        self.iter()
            .cloned()
            .filter(|g| !to_keep.contains(g))
            .collect()
    }

    pub fn get(&self, id: u32) -> Option<&Generation> {
        self.generations.iter().find(|g| g.id == id)
    }

    pub fn contains(&self, id: u32) -> bool {
        self.get(id).is_some()
    }

    pub fn len(&self) -> usize {
        self.generations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.generations.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Generation> {
        self.generations.iter()
    }
}

impl IntoIterator for GenerationSet {
    type Item = Generation;
    type IntoIter = std::collections::btree_set::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.generations.into_iter()
    }
}

impl<'a> IntoIterator for &'a GenerationSet {
    type Item = &'a Generation;
    type IntoIter = std::collections::btree_set::Iter<'a, Generation>;

    fn into_iter(self) -> Self::IntoIter {
        self.generations.iter()
    }
}

impl FromIterator<Generation> for GenerationSet {
    fn from_iter<T: IntoIterator<Item = Generation>>(iter: T) -> Self {
        Self {
            generations: iter.into_iter().collect(),
        }
    }
}

impl From<GenerationSet> for Vec<Generation> {
    fn from(val: GenerationSet) -> Self {
        val.generations.into_iter().collect()
    }
}

impl From<GenerationSet> for BTreeSet<u32> {
    fn from(val: GenerationSet) -> Self {
        val.generations.into_iter().map(|g| g.id).collect()
    }
}

impl<S> From<S> for GenerationSet
where
    S: AsRef<[Generation]>,
{
    fn from(iter: S) -> Self {
        Self {
            generations: iter.as_ref().iter().cloned().collect(),
        }
    }
}

#[cfg(test)]
mod test {
    use std::ops::RangeBounds;

    use eyre::Result;
    use rstest::{fixture, rstest};

    use crate::generation::Generation;

    use super::*;

    macro_rules! ndt {
        ( $date:expr ) => {
            NaiveDateTime::parse_from_str($date, "%Y-%m-%d %H:%M:%S").unwrap()
        };
    }

    const INPUT_WITH_CURRENT: &str = r#" 661   2023-06-01 08:10:47   
    662   2023-06-05 21:35:55   
    663   2023-06-06 13:17:20   
    664   2023-06-06 18:29:49   
    665   2023-06-07 07:57:08   
    666   2023-06-08 07:42:25   
    667   2023-06-13 22:13:13   
    668   2023-06-14 09:03:01   
    669   2023-06-15 12:21:00   
    670   2023-06-16 09:59:25   
    671   2023-06-19 18:54:32   
    672   2023-06-20 07:09:24   
    673   2023-07-03 08:56:50   
    674   2023-07-05 18:26:11   
    675   2023-07-10 08:56:27   
    676   2023-07-12 23:32:24   
    677   2023-07-13 12:55:34   
    678   2023-07-14 11:46:59   
    679   2023-07-15 10:32:58   
    680   2023-07-15 22:40:41   
    681   2023-07-16 11:35:46   (current)"#;

    #[fixture]
    fn parsed() -> Result<GenerationSet> {
        Ok(Generation::parse_many(INPUT_WITH_CURRENT)?.into())
    }

    #[rstest]
    #[case( 1, 681..=681)]
    #[case( 5, 677..=681)]
    #[case(10, 672..=681)]
    #[case(21, 661..=681)]
    #[case(22, 661..=681)]
    #[case(31, 661..=681)]
    fn test_get_last_n_generations<R>(
        parsed: Result<GenerationSet>,
        #[case] n: usize,
        #[case] ids: R,
    ) -> Result<()>
    where
        R: RangeBounds<u32> + IntoIterator<Item = u32>,
    {
        let filtered: BTreeSet<u32> = parsed?.get_last_n_generations(n).into();

        assert_eq!(filtered, ids.into_iter().collect());

        Ok(())
    }

    #[rstest]
    #[case(ndt!("2023-06-01 00:00:00"), 661..=681)]
    #[case(ndt!("2023-06-10 00:00:00"), 666..=681)]
    #[case(ndt!("2023-06-20 00:00:00"), 671..=681)]
    #[case(ndt!("2023-07-01 00:00:00"), 672..=681)]
    #[case(ndt!("2023-07-15 12:00:00"), 679..=681)]
    fn test_get_active_on_or_after<R>(
        parsed: Result<GenerationSet>,
        #[case] date: NaiveDateTime,
        #[case] ids: R,
    ) -> Result<()>
    where
        R: RangeBounds<u32> + IntoIterator<Item = u32>,
    {
        let filtered: BTreeSet<u32> = parsed?.get_active_on_or_after(date).into();

        assert_eq!(filtered, ids.into_iter().collect());

        Ok(())
    }

    #[rstest]
    #[case( 1, ndt!("2023-06-01 00:00:00"),   0..   0)]
    #[case( 1, ndt!("2023-07-01 00:00:00"), 661..=671)]
    #[case( 1, ndt!("2023-07-15 12:00:00"), 661..=678)]
    #[case( 5, ndt!("2023-06-01 00:00:00"),   0..   0)]
    #[case( 5, ndt!("2023-07-01 00:00:00"), 661..=671)]
    #[case( 5, ndt!("2023-07-15 12:00:00"), 661..=676)]
    #[case(10, ndt!("2023-06-01 00:00:00"),   0..   0)]
    #[case(10, ndt!("2023-07-01 00:00:00"), 661..=671)]
    #[case(10, ndt!("2023-07-15 12:00:00"), 661..=671)]
    #[case(21, ndt!("2023-06-01 00:00:00"),   0..   0)]
    #[case(21, ndt!("2023-07-01 00:00:00"),   0..   0)]
    #[case(21, ndt!("2023-07-15 12:00:00"),   0..   0)]
    #[case(22, ndt!("2023-06-01 00:00:00"),   0..   0)]
    #[case(22, ndt!("2023-07-01 00:00:00"),   0..   0)]
    #[case(22, ndt!("2023-07-15 12:00:00"),   0..   0)]
    #[case(31, ndt!("2023-06-01 00:00:00"),   0..   0)]
    #[case(31, ndt!("2023-07-01 00:00:00"),   0..   0)]
    #[case(31, ndt!("2023-07-15 12:00:00"),   0..   0)]
    fn test_generations_to_delete<R>(
        parsed: Result<GenerationSet>,
        #[case] keep: usize,
        #[case] date: NaiveDateTime,
        #[case] ids: R,
    ) -> Result<()>
    where
        R: RangeBounds<u32> + IntoIterator<Item = u32>,
    {
        let filtered: BTreeSet<u32> = parsed?.generations_to_delete(keep, date).into();

        assert_eq!(filtered, ids.into_iter().collect());

        Ok(())
    }

    #[rstest]
    #[case(661, ndt!("2023-06-01 08:10:47"), false)]
    #[case(666, ndt!("2023-06-08 07:42:25"), false)]
    #[case(671, ndt!("2023-06-19 18:54:32"), false)]
    #[case(678, ndt!("2023-07-14 11:46:59"), false)]
    #[case(681, ndt!("2023-07-16 11:35:46"), true)]
    fn test_get(
        parsed: Result<GenerationSet>,
        #[case] id: u32,
        #[case] date: NaiveDateTime,
        #[case] current: bool,
    ) -> Result<()> {
        assert_eq!(parsed?.get(id), Some(&Generation { id, date, current }));

        Ok(())
    }

    #[rstest]
    #[case(661, true)]
    #[case(666, true)]
    #[case(671, true)]
    #[case(678, true)]
    #[case(681, true)]
    #[case(650, false)]
    #[case(100, false)]
    #[case(800, false)]
    #[case(1000, false)]
    fn test_contains(
        parsed: Result<GenerationSet>,
        #[case] id: u32,
        #[case] exists: bool,
    ) -> Result<()> {
        assert_eq!(parsed?.contains(id), exists);

        Ok(())
    }

    #[rstest]
    #[case::empty(vec![].into(), 0)]
    #[case::one(vec![Generation{id: 1, date: ndt!("2020-01-01 00:00:00"), current: false}].into(), 1)]
    #[case::twenty_one(Generation::parse_many(INPUT_WITH_CURRENT).unwrap().into(), 21)]
    fn test_len(#[case] set: GenerationSet, #[case] len: usize) {
        assert_eq!(set.len(), len);
    }

    #[rstest]
    #[case::empty(vec![].into(), true)]
    #[case::one(vec![Generation{id: 1, date: ndt!("2020-01-01 00:00:00"), current: false}].into(), false)]
    #[case::twenty_one(Generation::parse_many(INPUT_WITH_CURRENT).unwrap().into(), false)]
    fn test_empty(#[case] set: GenerationSet, #[case] empty: bool) {
        assert_eq!(set.is_empty(), empty);
    }
}
