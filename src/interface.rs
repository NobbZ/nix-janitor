use clap::{crate_authors, ArgAction, Parser};
use tracing::{metadata::LevelFilter, Level};
use tracing_subscriber::fmt::format::FmtSpan;

#[cfg_attr(test, derive(Clone))]
#[derive(Debug, Parser)]
#[command(version, author = crate_authors!())]
pub struct NJParser {
    /// The number of days to keep generations
    #[clap(long, short = 'd', default_value = "7")]
    pub keep_days: i64,
    /// The minimum number of generations to keep
    #[clap(long, short = 'l', default_value = "5")]
    pub keep_at_least: usize,

    /// Delete by age only (still keeps at least 1 generation, regardless of age)
    #[clap(long, short = 'a', conflicts_with = "keep_at_least")]
    pub by_age_only: bool,

    /// Increase verbosity (up to three times)
    #[clap(long = "verbose", short = 'v', action = ArgAction::Count, conflicts_with = "quiet")]
    pub verbosity: u8,

    /// Only log warnings and errors
    #[clap(long, short = 'q', conflicts_with = "verbosity")]
    pub quiet: bool,
}

impl NJParser {
    pub fn log_level_and_span(&self) -> (Level, FmtSpan) {
        match (self.quiet, self.verbosity) {
            (true, 0) => (Level::WARN, FmtSpan::NONE),
            (false, 0) => (Level::INFO, FmtSpan::NONE),
            (false, 1) => (Level::DEBUG, FmtSpan::NONE),
            (false, 2) => (Level::TRACE, FmtSpan::NONE),
            (false, _) => (Level::TRACE, FmtSpan::ENTER | FmtSpan::EXIT),
            (true, _) => unreachable!("--quiet and --verbose are mutually exclusive"),
        }
    }
}

impl From<&NJParser> for FmtSpan {
    fn from(parser: &NJParser) -> Self {
        parser.log_level_and_span().1
    }
}

impl From<&NJParser> for LevelFilter {
    fn from(parser: &NJParser) -> Self {
        parser.log_level_and_span().0.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use rstest::rstest;

    #[rstest]
    #[case::short(vec!["janitor", "-d", "3", "-l", "2"])]
    #[case::long(vec!["janitor", "--keep-days", "3", "--keep-at-least", "2"])]
    fn test_parser(#[case] args: Vec<&str>) {
        let args = NJParser::parse_from(args);
        assert_eq!(args.keep_days, 3);
        assert_eq!(args.keep_at_least, 2);
        assert_eq!(args.by_age_only, false);
        assert_eq!(args.verbosity, 0);
        assert_eq!(args.quiet, false);
    }

    #[rstest]
    #[case::age_only(vec!["janitor", "-a", "-l", "2"])]
    #[case::verbose_quiet(vec!["janitor", "-v", "-q"])]
    fn test_conflicts(#[case] args: Vec<&str>) {
        let result = NJParser::try_parse_from(args);
        dbg!(&result);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[rstest]
    #[case::default(vec!["janitor"], Level::INFO, FmtSpan::NONE)]
    #[case::quiet(vec!["janitor", "-q"], Level::WARN, FmtSpan::NONE)]
    #[case::verbose(vec!["janitor", "-v"], Level::DEBUG, FmtSpan::NONE)]
    #[case::trace(vec!["janitor", "-vv"], Level::TRACE, FmtSpan::NONE)]
    #[case::trace_span(vec!["janitor", "-vvv"], Level::TRACE, FmtSpan::ENTER | FmtSpan::EXIT)]
    fn test_log_level_and_span(
        #[case] args: Vec<&str>,
        #[case] level: Level,
        #[case] span: FmtSpan,
    ) {
        let args = NJParser::parse_from(args);
        let (log_level, span_events) = args.log_level_and_span();
        assert_eq!(log_level, level);
        assert_eq!(span_events, span);

        assert_eq!(log_level, LevelFilter::from(&args));

        assert_eq!(span_events, FmtSpan::from(&args));
    }
}
