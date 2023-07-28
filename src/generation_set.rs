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

    pub fn iter(&self) -> impl Iterator<Item = &Generation> {
        self.generations.iter()
    }

    pub fn len(&self) -> usize {
        self.generations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.generations.is_empty()
    }
}

impl FromIterator<Generation> for GenerationSet {
    fn from_iter<T: IntoIterator<Item = Generation>>(iter: T) -> Self {
        Self {
            generations: iter.into_iter().collect(),
        }
    }
}

impl From<Vec<Generation>> for GenerationSet {
    fn from(generations: Vec<Generation>) -> Self {
        Self {
            generations: generations.into_iter().collect(),
        }
    }
}

impl From<&[Generation]> for GenerationSet {
    fn from(generations: &[Generation]) -> Self {
        Self {
            generations: generations.iter().cloned().collect(),
        }
    }
}

#[cfg(test)]
mod test {
    use rstest::rstest;

    use eyre::Result;

    use super::*;
    use crate::generation::Generation;

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

    #[rstest]
    #[case(1, 681)]
    #[case(5, 677)]
    #[case(10, 672)]
    #[case(21, 661)]
    #[case(22, 661)]
    #[case(31, 661)]
    fn test_get_last_n_generations(#[case] n: usize, #[case] first_id: u32) -> Result<()> {
        let parsed_vec = Generation::parse_many(INPUT_WITH_CURRENT)?;
        let parsed = Into::<GenerationSet>::into(parsed_vec.as_slice());

        let filtered = parsed.get_last_n_generations(n);
        let mut filtered_vec = filtered.iter().cloned().collect::<Vec<_>>();

        filtered_vec.sort_by(|a, b| a.id.cmp(&b.id));

        assert_eq!(filtered_vec[0].id, first_id);
        assert_eq!(filtered_vec, parsed_vec[parsed.len() - filtered.len()..]);

        Ok(())
    }

    #[rstest]
    #[case(ndt!("2023-06-01 00:00:00"), 661)]
    #[case(ndt!("2023-06-10 00:00:00"), 666)]
    #[case(ndt!("2023-06-20 00:00:00"), 671)]
    #[case(ndt!("2023-07-01 00:00:00"), 672)]
    #[case(ndt!("2023-07-15 12:00:00"), 679)]
    fn test_get_active_on_or_after(#[case] date: NaiveDateTime, #[case] id: u32) -> Result<()> {
        let parsed_vec = Generation::parse_many(INPUT_WITH_CURRENT)?;
        let parsed = Into::<GenerationSet>::into(parsed_vec.as_ref());
        let parsed_ids = parsed
            .iter()
            .filter_map(|g| if g.id >= id { Some(g.id) } else { None })
            .collect::<BTreeSet<u32>>();

        let filtered = parsed.get_active_on_or_after(date);
        let filtered_ids = filtered.iter().map(|g| g.id).collect::<BTreeSet<u32>>();

        assert_eq!(filtered_ids.first(), Some(&id));
        assert_eq!(filtered_ids, parsed_ids);

        Ok(())
    }

    #[rstest]
    #[case(1, ndt!("2023-06-01 00:00:00"), None)]
    #[case(1, ndt!("2023-07-01 00:00:00"), Some(10))]
    #[case(1, ndt!("2023-07-15 12:00:00"), Some(17))]
    #[case(5, ndt!("2023-06-01 00:00:00"), None)]
    #[case(5, ndt!("2023-07-01 00:00:00"), Some(10))]
    #[case(5, ndt!("2023-07-15 12:00:00"), Some(15))]
    #[case(10, ndt!("2023-06-01 00:00:00"), None)]
    #[case(10, ndt!("2023-07-01 00:00:00"), Some(10))]
    #[case(10, ndt!("2023-07-15 12:00:00"), Some(10))]
    #[case(21, ndt!("2023-06-01 00:00:00"), None)]
    #[case(21, ndt!("2023-07-01 00:00:00"), None)]
    #[case(21, ndt!("2023-07-15 12:00:00"), None)]
    #[case(22, ndt!("2023-06-01 00:00:00"), None)]
    #[case(22, ndt!("2023-07-01 00:00:00"), None)]
    #[case(22, ndt!("2023-07-15 12:00:00"), None)]
    #[case(31, ndt!("2023-06-01 00:00:00"), None)]
    #[case(31, ndt!("2023-07-01 00:00:00"), None)]
    #[case(31, ndt!("2023-07-15 12:00:00"), None)]
    fn test_generations_to_delete(
        #[case] keep: usize,
        #[case] date: NaiveDateTime,
        #[case] end: Option<usize>,
    ) -> Result<()> {
        let parsed_vec = Generation::parse_many(INPUT_WITH_CURRENT)?;
        let parsed = Into::<GenerationSet>::into(parsed_vec.clone());

        let mut filtered = parsed
            .generations_to_delete(keep, date)
            .iter()
            .cloned()
            .collect::<Vec<_>>();

        filtered.sort_by(|a, b| a.id.cmp(&b.id));

        assert_eq!(
            filtered,
            end.map_or_else(Vec::new, |end| parsed_vec[0..=end].into())
        );

        Ok(())
    }
}
