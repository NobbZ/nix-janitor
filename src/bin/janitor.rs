use std::{env, future::Future, process::Stdio};

use chrono::{prelude::*, Duration};
use clap::{crate_authors, Parser};
use eyre::Result;
use futures::future::try_join_all;
use tokio::process::Command;
use tracing::{Instrument, Level};
use tracing_subscriber::{fmt::format::FmtSpan, FmtSubscriber};

use janitor::{Generation, GenerationSet, Job, Profile};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Parser)]
#[command(version, author = crate_authors!())]
struct Cli {
    /// The number of days to keep generations
    #[clap(long, short = 'd', default_value = "7")]
    keep_days: i64,
    /// The minimum number of generations to keep
    #[clap(long, short = 'l', default_value = "5")]
    keep_at_least: usize,

    /// Enable verbose output
    #[clap(long, short = 'v')]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    let level = if args.verbose {
        Level::TRACE
    } else {
        Level::INFO
    };

    // Configure and initialize logging
    FmtSubscriber::builder()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_max_level(level)
        .init();

    let profile_paths = Profile::all();

    // Configure thresholds and "print welcome"
    let now = Utc::now().naive_utc();
    let keep_since = now - Duration::days(args.keep_days);
    let keep_at_least = args.keep_at_least;
    tracing::info!(
        start_time = %now,
        %keep_since,
        keep_at_least = args.keep_at_least,
        profiles = ?profile_paths,
        version = VERSION,
        "Starting janitor"
    );

    try_join_all(
        profile_paths
            .iter()
            .map(|path| Job::new(path, keep_since, keep_at_least, ()))
            .map(get_generations)
            .map(get_to_delete)
            .map(run_delete)
            .collect::<Vec<_>>(),
    )
    .instrument(tracing::info_span!("processing_profiles"))
    .await?;

    Ok(())
}

#[tracing::instrument]
async fn get_generations(job: Job<()>) -> Result<Job<GenerationSet>> {
    let path = job.path();

    let output = Command::new("nix-env")
        .arg("--list-generations")
        .arg("--profile")
        .arg(path)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()
        .instrument(tracing::info_span!("nix-env"))
        .await?;

    if !output.status.success() {
        return Err(eyre::eyre!(
            "nix-env failed: {stdout}",
            stdout = std::str::from_utf8(output.stderr.as_ref())?
        ));
    }

    let parsed = Generation::parse_many(std::str::from_utf8(output.stdout.as_ref())?)?.into();

    Ok(job.set_data(parsed))
}

#[tracing::instrument(skip(job), fields(path))]
async fn get_to_delete(
    job: impl Future<Output = Result<Job<GenerationSet>>>,
) -> Result<Job<GenerationSet>> {
    let job = job.await?;
    let path = job.path();
    tracing::Span::current().record("path", path.to_str());

    let keep_since = job.keep_since();
    let keep_at_least = job.keep_at_least();

    let to_delete = job.data().generations_to_delete(keep_at_least, keep_since);

    Ok(job.set_data(to_delete))
}

#[tracing::instrument(skip(job), fields(path))]
async fn run_delete(job: impl Future<Output = Result<Job<GenerationSet>>>) -> Result<Job<()>> {
    let job = job.await?;
    let path = job.path();
    tracing::Span::current().record("path", path.to_str());

    let ids: Vec<_> = job
        .data()
        .iter()
        .map(|g| g.id)
        .map(|id| id.to_string())
        .collect();

    tracing::info!(?path, ?ids, "deleting generations");

    let output = Command::new("nix-env")
        .arg("--profile")
        .arg(path)
        .arg("--delete-generations")
        .args(&ids)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()
        .instrument(tracing::info_span!("delete_generations"))
        .await?;

    if !output.status.success() {
        return Err(eyre::eyre!(
            "nix-env failed: {stderr}",
            stderr = std::str::from_utf8(output.stderr.as_ref())?
        ));
    }

    tracing::info!(?path, ?ids, "deleted generations");

    Ok(job.set_data(()))
}
