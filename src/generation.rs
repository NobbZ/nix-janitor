use chrono::prelude::*;
use eyre::Result;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Generation {
    pub(crate) id: u32,
    pub(crate) date: NaiveDateTime,
    pub(crate) current: bool,
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

#[cfg(test)]
mod test {
    use super::*;

    use lazy_static::lazy_static;

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
}
