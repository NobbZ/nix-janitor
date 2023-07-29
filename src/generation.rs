use chrono::prelude::*;
use eyre::{eyre, Context, Result};

/// Represents a single generation of a nix profile.
///
/// # Fields
///
/// * `id` - The unique id of this generation.
/// * `date` - The date and time this generation was created.
/// * `current` - Whether this generation is the currently active one.
///
/// # Examples
///
/// ```
/// use janitor::Generation;
/// use chrono::NaiveDateTime;
///
/// let generation = Generation {
///     id: 661,
///     date: NaiveDateTime::parse_from_str("2023-06-01 08:10:47", "%Y-%m-%d %H:%M:%S").unwrap(),  
///     current: false,
/// };
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Generation {
    /// The ID of this generation.
    ///
    /// Nix uses this ID itself to identify the generation within the profile.
    pub id: u32,

    /// The date and time this generation was created.
    pub date: NaiveDateTime,

    /// Whether this generation is the currently active one.
    pub current: bool,
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
    /// Parses a generation from an input string.
    ///
    /// # Arguments
    ///
    /// * `input` - The input string to parse. Should contain the id, date, time
    ///   and optionally "(current)" to indicate if this is the current generation.
    ///
    /// # Errors
    ///
    /// Returns an `eyre::Result` which can fail with:
    ///
    /// - An `eyre::Error` if the id fails to parse as a `u32`.
    /// - An `eyre::Error` if the date or time strings are missing.
    /// - A `chrono::ParseError` if the date/time fails to parse.
    ///
    /// # Examples
    ///
    /// ```
    /// # use janitor::*;
    /// use chrono::NaiveDateTime;
    /// use eyre::Result;
    ///
    /// # fn main() -> Result<()> {
    /// let input = "661 2023-06-01 08:10:47";
    /// let generation = Generation::parse(input)?;
    /// assert_eq!(generation.id, 661);
    /// assert_eq!(generation.date, NaiveDateTime::parse_from_str("2023-06-01 08:10:47", "%Y-%m-%d %H:%M:%S").unwrap());
    /// assert!(!generation.current);
    ///
    /// let input = "681 2023-07-16 11:35:46 (current)";
    /// let generation = Generation::parse(input)?;
    /// assert!(generation.current);
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse<S>(input: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        let mut parts = input.as_ref().split_whitespace();

        let id = parts
            .next()
            .unwrap()
            .parse::<u32>()
            .wrap_err("Failed to parse generation id")?;
        let date_str = parts.next().ok_or_else(|| eyre!("Date missing"))?;
        let time_str = parts.next().ok_or_else(|| eyre!("Time missing"))?;
        let date_time_str = format!("{} {}", date_str, time_str);
        let date = NaiveDateTime::parse_from_str(&date_time_str, "%Y-%m-%d %H:%M:%S")?;

        let current = match parts.next() {
            Some("(current)") => true,
            None => false,
            _ => return Err(eyre!("Invalid current flag")),
        };

        Ok(Self { id, date, current })
    }

    /// Parses multiple generations from a string with each generation on a new line.
    ///
    /// Empty lines, or those only containing whitespace, will be ignored.
    ///
    /// # Arguments
    ///
    /// * `input` - The input string to parse. Each line should contain a single
    ///   generation in the format accepted by [Generation::parse].
    ///
    /// # Errors
    ///
    /// Returns an `eyre::Result` which will accumulate any errors from the individual
    /// calls to [Generation::parse] on each line.
    ///
    /// # Examples
    ///
    /// ```
    /// # use janitor::Generation;
    ///
    /// # fn main() -> eyre::Result<()> {
    /// let input = "
    /// 661 2023-06-01 08:10:47
    /// 662 2023-06-05 21:35:55  
    /// ";
    ///
    /// let generations = Generation::parse_many(input)?;
    /// assert_eq!(generations.len(), 2);
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_many<S>(input: S) -> Result<Vec<Self>>
    where
        S: AsRef<str>,
    {
        input
            .as_ref()
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(Self::parse)
            .collect::<Result<Vec<Self>>>()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use rstest::rstest;

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

    #[rstest]
    #[case::without_current("681   2023-07-16 11:35:46", generation!(681, "2023-07-16 11:35:46"))]
    #[case::with_current("681   2023-07-16 11:35:46  (current)", generation!(681, "2023-07-16 11:35:46", true))]
    fn parse_single(#[case] input: &str, #[case] expected: Generation) -> Result<()> {
        let parsed = Generation::parse(input)?;

        assert_eq!(parsed, expected);

        Ok(())
    }

    #[rstest]
    #[case::invalid_time("123 2023-01-01 25:61:00")]
    #[case::missing_time("123 2023-01-01 ")]
    #[case::invalid_date("123 2023-01-32 00:00:00")]
    #[case::missing_date("123")]
    #[case::invalid_id("abc 2023-01-01 00:00:00")]
    #[case::invalid_current("123 2023-01-01 00:00:00 (invalid)")]
    fn parse_errors(#[case] input: &str) {
        assert!(Generation::parse(input).is_err());
    }

    #[rstest]
    #[case::without_current(INPUT_WITHOUT_CURRENT, GENERATIONS_WITHOUT_CURRENT.clone())]
    #[case::with_current(INPUT_WITH_CURRENT, GENERATIONS_WITH_CURRENT.clone())]
    #[case::with_current_in_the_middle(
        INPUT_WITH_CURRENT_IN_THE_MIDDLE,
        GENERATIONS_WITH_CURRENT_IN_THE_MIDDLE.clone()
    )]
    fn parse_many<G>(#[case] input: &str, #[case] expected: G)
    where
        G: AsRef<[Generation]>,
    {
        let parsed = Generation::parse_many(input).unwrap();

        assert_eq!(parsed, expected.as_ref());
    }
}
