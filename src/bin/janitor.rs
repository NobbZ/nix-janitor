use std::{env, fmt, future::Future, path::PathBuf, process::Stdio};

use chrono::{prelude::*, Duration};
use eyre::Result;
use futures::future::try_join_all;
use tokio::process::Command;
use tracing::{Instrument, Level};
use tracing_subscriber::{fmt::format::FmtSpan, FmtSubscriber};

use janitor::{Generation, GenerationSet};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const KEEP_AT_LEAST: usize = 5;
const KEEP_DAYS: i64 = 7;

struct Job<T> {
    path: PathBuf,
    keep_since: NaiveDateTime,
    keep_at_least: usize,
    data: T,
}

impl<T> fmt::Debug for Job<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Job")
            .field("path", &self.path)
            .field("keep_since", &self.keep_since)
            .field("keep_at_least", &self.keep_at_least)
            .finish()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Configure and initialize logging
    FmtSubscriber::builder()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_max_level(Level::TRACE)
        .init();

    let profile_paths = {
        let mut paths = vec![
            "/nix/var/nix/profiles/per-user/$USER/profile",
            "/home/$USER/.local/state/nix/profiles/home-manager",
        ];

        if is_root::is_root() {
            paths.push("/nix/var/nix/profiles/system");
        }

        paths
            .iter()
            .map(|p| -> Result<_> { Ok(shellexpand::env_with_context(p, context).unwrap()) })
            .map(|p| -> Result<_> { Ok(PathBuf::from(p?.to_string())) })
            .filter_map(|pr| pr.ok())
            .filter(|p| p.exists())
            .collect::<Vec<_>>()
    };

    // Configure thresholds and "print welcome"
    let now = Utc::now().naive_utc();
    let keep_since = now - Duration::days(KEEP_DAYS);
    let keep_at_least = KEEP_AT_LEAST;
    tracing::info!(
        start_time = %now,
        %keep_since,
        keep_at_least = KEEP_AT_LEAST,
        profiles = ?profile_paths,
        version = VERSION,
        "Starting janitor"
    );

    try_join_all(
        profile_paths
            .iter()
            .map(|path| Job {
                path: path.clone(),
                keep_since,
                keep_at_least,
                data: (),
            })
            .map(get_generations)
            .map(get_to_delete)
            .map(run_delete)
            .collect::<Vec<_>>(),
    )
    .instrument(tracing::info_span!("processing_profiles"))
    .await?;

    Ok(())
}

fn context(s: &str) -> Result<Option<String>> {
    match s {
        "USER" => Ok(get_username()),
        v => Err(eyre::eyre!("unknown variable: {v}")),
    }
}

fn get_username() -> Option<String> {
    if is_root::is_root() {
        tracing::debug!("running as root, using SUDO_USER");
        env::var("SUDO_USER").ok()
    } else {
        tracing::debug!("running regular user, using USER");
        env::var("USER").ok()
    }
}

#[tracing::instrument]
async fn get_generations(job: Job<()>) -> Result<Job<GenerationSet>> {
    let path = job.path;

    let output = Command::new("nix-env")
        .arg("--list-generations")
        .arg("--profile")
        .arg(&path)
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

    Ok(Job {
        data: parsed,
        path,
        keep_since: job.keep_since,
        keep_at_least: job.keep_at_least,
    })
}

#[tracing::instrument(skip(job), fields(path))]
async fn get_to_delete(
    job: impl Future<Output = Result<Job<GenerationSet>>>,
) -> Result<Job<GenerationSet>> {
    let job = job.await?;
    let path = job.path;
    tracing::Span::current().record("path", path.to_str());

    let keep_since = job.keep_since;
    let keep_at_least = job.keep_at_least;

    let to_delete = job.data.generations_to_delete(keep_at_least, keep_since);

    Ok(Job {
        data: to_delete,
        path,
        keep_since,
        keep_at_least,
    })
}

#[tracing::instrument(skip(job), fields(path))]
async fn run_delete(job: impl Future<Output = Result<Job<GenerationSet>>>) -> Result<Job<()>> {
    let job = job.await?;
    let path = job.path;
    tracing::Span::current().record("path", path.to_str());

    let ids: Vec<_> = job
        .data
        .iter()
        .map(|g| g.id)
        .map(|id| id.to_string())
        .collect();

    tracing::info!(?path, ?ids, "deleting generations");

    let output = Command::new("nix-env")
        .arg("--profile")
        .arg(&path)
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

    Ok(Job {
        data: (),
        path,
        keep_since: job.keep_since,
        keep_at_least: job.keep_at_least,
    })
}
