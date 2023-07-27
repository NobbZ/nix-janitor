use std::collections::BTreeSet;

use chrono::prelude::*;
use eyre::Result;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Generation {
    id: u32,
    date: NaiveDateTime,
    current: bool,
}

impl PartialOrd for Generation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for Generation {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl Generation {
    pub fn parse(input: &str) -> Result<Self> {
        let mut parts = input.split_whitespace();

        let id = parts.next().unwrap().parse::<u32>().unwrap();
        let date = NaiveDateTime::parse_from_str(
            &format!("{} {}", parts.next().unwrap(), parts.next().unwrap()),
            "%Y-%m-%d %H:%M:%S",
        )?;

        let current = parts.next() == Some("(current)");

        Ok(Self { id, date, current })
    }

    pub fn parse_many(input: &str) -> Result<Vec<Self>> {
        input
            .lines()
            .map(Self::parse)
            .collect::<Result<Vec<Self>>>()
    }
}

pub struct GenerationSet {
    generations: BTreeSet<Generation>,
}

impl GenerationSet {
    pub fn get_last_n_generations(&self, n: usize) -> Self {
        let mut generations = self.generations.iter().cloned().collect::<Vec<_>>();

        generations.sort_by(|a, b| a.id.cmp(&b.id));

        if n >= generations.len() {
            return generations.into();
        }

        generations[generations.len() - n..].into()
    }

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
    use super::*;

    use lazy_static::lazy_static;
    use rstest::rstest;

    macro_rules! ndt {
        ( $date:expr ) => {
            NaiveDateTime::parse_from_str($date, "%Y-%m-%d %H:%M:%S").unwrap()
        };
    }

    macro_rules! generation {
        ($id:expr, $date:expr) => {
            Generation {
                id: $id,
                date: NaiveDateTime::parse_from_str($date, "%Y-%m-%d %H:%M:%S").unwrap(),
                current: false,
            }
        };

        ($id:expr, $date:expr, $current:expr) => {
            Generation {
                id: $id,
                date: NaiveDateTime::parse_from_str($date, "%Y-%m-%d %H:%M:%S").unwrap(),
                current: $current,
            }
        };
    }

    lazy_static! {
        static ref GENERATIONS_WITHOUT_CURRENT: Vec<Generation> = vec![
            generation!(661, "2023-06-01 08:10:47"),
            generation!(662, "2023-06-05 21:35:55"),
            generation!(663, "2023-06-06 13:17:20"),
            generation!(664, "2023-06-06 18:29:49"),
            generation!(665, "2023-06-07 07:57:08"),
            generation!(666, "2023-06-08 07:42:25"),
            generation!(667, "2023-06-13 22:13:13"),
            generation!(668, "2023-06-14 09:03:01"),
            generation!(669, "2023-06-15 12:21:00"),
            generation!(670, "2023-06-16 09:59:25"),
            generation!(671, "2023-06-19 18:54:32"),
            generation!(672, "2023-06-20 07:09:24"),
            generation!(673, "2023-07-03 08:56:50"),
            generation!(674, "2023-07-05 18:26:11"),
            generation!(675, "2023-07-10 08:56:27"),
            generation!(676, "2023-07-12 23:32:24"),
            generation!(677, "2023-07-13 12:55:34"),
            generation!(678, "2023-07-14 11:46:59"),
            generation!(679, "2023-07-15 10:32:58"),
            generation!(680, "2023-07-15 22:40:41"),
            generation!(681, "2023-07-16 11:35:46"),
        ];
        static ref GENERATIONS_WITH_CURRENT: Vec<Generation> = vec![
            generation!(661, "2023-06-01 08:10:47"),
            generation!(662, "2023-06-05 21:35:55"),
            generation!(663, "2023-06-06 13:17:20"),
            generation!(664, "2023-06-06 18:29:49"),
            generation!(665, "2023-06-07 07:57:08"),
            generation!(666, "2023-06-08 07:42:25"),
            generation!(667, "2023-06-13 22:13:13"),
            generation!(668, "2023-06-14 09:03:01"),
            generation!(669, "2023-06-15 12:21:00"),
            generation!(670, "2023-06-16 09:59:25"),
            generation!(671, "2023-06-19 18:54:32"),
            generation!(672, "2023-06-20 07:09:24"),
            generation!(673, "2023-07-03 08:56:50"),
            generation!(674, "2023-07-05 18:26:11"),
            generation!(675, "2023-07-10 08:56:27"),
            generation!(676, "2023-07-12 23:32:24"),
            generation!(677, "2023-07-13 12:55:34"),
            generation!(678, "2023-07-14 11:46:59"),
            generation!(679, "2023-07-15 10:32:58"),
            generation!(680, "2023-07-15 22:40:41"),
            generation!(681, "2023-07-16 11:35:46", true),
        ];
        static ref GENERATIONS_WITH_CURRENT_IN_THE_MIDDLE: Vec<Generation> = vec![
            generation!(661, "2023-06-01 08:10:47"),
            generation!(662, "2023-06-05 21:35:55"),
            generation!(663, "2023-06-06 13:17:20"),
            generation!(664, "2023-06-06 18:29:49"),
            generation!(665, "2023-06-07 07:57:08"),
            generation!(666, "2023-06-08 07:42:25"),
            generation!(667, "2023-06-13 22:13:13"),
            generation!(668, "2023-06-14 09:03:01"),
            generation!(669, "2023-06-15 12:21:00"),
            generation!(670, "2023-06-16 09:59:25"),
            generation!(671, "2023-06-19 18:54:32", true),
            generation!(672, "2023-06-20 07:09:24"),
            generation!(673, "2023-07-03 08:56:50"),
            generation!(674, "2023-07-05 18:26:11"),
            generation!(675, "2023-07-10 08:56:27"),
            generation!(676, "2023-07-12 23:32:24"),
            generation!(677, "2023-07-13 12:55:34"),
            generation!(678, "2023-07-14 11:46:59"),
            generation!(679, "2023-07-15 10:32:58"),
            generation!(680, "2023-07-15 22:40:41"),
            generation!(681, "2023-07-16 11:35:46"),
        ];
    }

    const INPUT_WITHOUT_CURRENT: &str = r#" 661   2023-06-01 08:10:47   
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
     681   2023-07-16 11:35:46"#;

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

    const INPUT_WITH_CURRENT_IN_THE_MIDDLE: &str = r#" 661   2023-06-01 08:10:47   
     662   2023-06-05 21:35:55   
     663   2023-06-06 13:17:20   
     664   2023-06-06 18:29:49   
     665   2023-06-07 07:57:08   
     666   2023-06-08 07:42:25   
     667   2023-06-13 22:13:13   
     668   2023-06-14 09:03:01   
     669   2023-06-15 12:21:00   
     670   2023-06-16 09:59:25   
     671   2023-06-19 18:54:32   (current)
     672   2023-06-20 07:09:24   
     673   2023-07-03 08:56:50   
     674   2023-07-05 18:26:11   
     675   2023-07-10 08:56:27   
     676   2023-07-12 23:32:24   
     677   2023-07-13 12:55:34   
     678   2023-07-14 11:46:59   
     679   2023-07-15 10:32:58   
     680   2023-07-15 22:40:41   
     681   2023-07-16 11:35:46"#;

    #[test]
    fn parse_single_generation() -> Result<()> {
        let parsed = Generation::parse("681   2023-07-16 11:35:46")?;

        assert_eq!(parsed, generation!(681, "2023-07-16 11:35:46"));

        Ok(())
    }

    #[test]
    fn parse_single_current_generation() -> Result<()> {
        let parsed = Generation::parse("681   2023-07-16 11:35:46    (current)")?;

        assert_eq!(parsed, generation!(681, "2023-07-16 11:35:46", true));

        Ok(())
    }

    #[test]
    fn parse_many_without_current() -> Result<()> {
        let parsed = Generation::parse_many(INPUT_WITHOUT_CURRENT)?;

        assert_eq!(parsed, *GENERATIONS_WITHOUT_CURRENT);

        Ok(())
    }

    #[test]
    fn parse_many_with_current() -> Result<()> {
        let parsed = Generation::parse_many(INPUT_WITH_CURRENT)?;

        assert_eq!(parsed, *GENERATIONS_WITH_CURRENT);

        Ok(())
    }

    #[test]
    fn parse_many_with_current_in_the_middle() -> Result<()> {
        let parsed = Generation::parse_many(INPUT_WITH_CURRENT_IN_THE_MIDDLE)?;

        assert_eq!(parsed, *GENERATIONS_WITH_CURRENT_IN_THE_MIDDLE);

        Ok(())
    }

    #[rstest]
    #[case(1, 681)]
    #[case(5, 677)]
    #[case(10, 672)]
    #[case(21, 661)]
    #[case(22, 661)]
    #[case(31, 661)]
    fn test_get_last_n_generations(#[case] n: usize, #[case] first_id: u32) -> Result<()> {
        let parsed_vec = Generation::parse_many(INPUT_WITHOUT_CURRENT)?;
        let parsed = Into::<GenerationSet>::into(parsed_vec.clone());

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
        let parsed_vec = Generation::parse_many(INPUT_WITHOUT_CURRENT)?;
        let parsed = Into::<GenerationSet>::into(parsed_vec.clone());
        let parsed_ids = parsed
            .iter()
            .filter_map(|g| if g.id >= id { Some(g.id) } else { None })
            .collect::<BTreeSet<u32>>();

        let filtered = parsed.get_active_on_or_after(date);
        let filtered_ids = filtered.iter().map(|g| g.id).collect::<BTreeSet<u32>>();

        let lowest_id = filtered.iter().map(|g| g.id).min().unwrap();

        assert_eq!(lowest_id, id);
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
