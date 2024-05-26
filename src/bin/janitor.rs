#![cfg(not(tarpaulin_include))]

use std::io;
use std::{env, future::Future, process::Stdio};

use chrono::{prelude::*, Duration};
use clap::Parser;
use eyre::{OptionExt, Result};
use futures::future::try_join_all;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tracing::Instrument;
use tracing_subscriber::FmtSubscriber;

use janitor::{interface::NJParser, option, Generation, GenerationSet, Job, Profile};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    let args = <NJParser as Parser>::parse();

    // Configure and initialize logging
    FmtSubscriber::builder()
        .with_span_events((&args).into())
        .with_max_level(&args)
        .init();

    if args.verbosity > 3 {
        tracing::warn!(
            verbosity = args.verbosity,
            "Verbosity above 3 does not change anything"
        );
    }

    let profile_paths = Profile::all();

    // Configure thresholds and "print welcome"
    let now = Utc::now().naive_utc();
    let keep_since = now - Duration::days(args.keep_days);
    let keep_at_least = option::optional(!args.by_age_only, args.keep_at_least);
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
            .map(|path| Job::new(path, keep_since, keep_at_least.unwrap_or(1), ()))
            .map(get_generations)
            .map(get_to_delete)
            .map(run_delete)
            .collect::<Vec<_>>(),
    )
    .instrument(tracing::info_span!("processing_profiles"))
    .await?;

    if args.gc {
        perform_gc(args.verbosity > 0).await?;
    };

    Ok(())
}

#[tracing::instrument]
async fn perform_gc(verbose: bool) -> Result<()> {
    let mut cmd = Command::new("nix-store");
    cmd.args(["--verbose", "--gc"]);
    cmd.stderr(Stdio::piped());
    cmd.stdout(Stdio::piped());

    let mut child = cmd.spawn()?;

    let stderr = child
        .stderr
        .take()
        .ok_or_eyre("chid did not have a handle to stderr")?;

    let stdout = child
        .stdout
        .take()
        .ok_or_eyre("child did not have a handle to stdout")?;

    let mut stderr_reader = BufReader::new(stderr).lines();
    let mut stdout_reader = BufReader::new(stdout).lines();

    // Ensure child runs in the tokio runtime and is able to proceed, while we
    // await its output
    let (tx, mut rx) = mpsc::channel(1);
    tokio::spawn(async move {
        let status = child.wait().await;
        tx.send(status).await.unwrap();
    });

    let status = {
        loop {
            tokio::select! {
                maybe_line = stderr_reader.next_line() => process_stderr_line(maybe_line)?,
                maybe_line = stdout_reader.next_line() => process_stdout_line(maybe_line)?,
                Some(status) = rx.recv() => { break status?; },
            }
        }
    };

    if !status.success() {
        tracing::warn!(code = status.code(), "nix-store --gc failed");
    };

    tracing::info!("nix-store --gc completed successfully");

    Ok(())
}

fn process_stderr_line(maybe_line: Result<Option<String>, io::Error>) -> Result<()> {
    if let Some(line) = maybe_line? {
        if line == "waiting for the big garbage collector lock..." {
            tracing::debug!("waiting for the big garbage collector lock");
        } else if line == "finding garbage collector roots..." {
            tracing::debug!("finding garbage collector roots");
        } else if line == "deleting garbage..." {
            tracing::info!("start deleting garbage");
        } else if line == "deleting unused links..." {
            tracing::info!("deleting unused links");
        } else if line.starts_with("note: currently hard linking saves") {
            let saved = line
                .strip_prefix("note: currently hard linking saves ")
                .unwrap();
            tracing::info!(%saved, "hard linking saves");
        } else if line.starts_with("deleting '") {
            let path = line
                .strip_prefix("deleting '")
                .unwrap()
                .strip_suffix('\'')
                .unwrap();
            tracing::debug!(%path, "deleting path");
        } else if line.starts_with("removing stale temporary roots file '") {
            let path = line
                .strip_prefix("removing stale temporary roots file '")
                .unwrap()
                .strip_suffix('\'')
                .unwrap();
            tracing::debug!(%path, "removing stale temporary roots file");
        } else if line.starts_with("removing stale link from '") {
            let line = line
                .strip_prefix("removing stale link from '")
                .unwrap()
                .strip_suffix('\'')
                .unwrap();
            let paths = line.split("' to '").collect::<Vec<_>>();
            let auto_root = paths[0];
            let target = paths[1];
            tracing::debug!(%auto_root, %target, "removing stale link");
        } else if line.starts_with("deleting unused link '") {
            let path = line
                .strip_prefix("deleting unused link '")
                .unwrap()
                .strip_suffix('\'')
                .unwrap();
            tracing::debug!(%path, "deleting hardlink");
        } else {
            tracing::warn!(stderr = %line, "unrecognized output from nix-store --gc");
        };
    };

    Ok(())
}

fn process_stdout_line(maybe_line: Result<Option<String>, io::Error>) -> Result<()> {
    if let Some(line) = maybe_line? {
        match line.split_whitespace().collect::<Vec<_>>().as_slice() {
            &[deleted, "store", "paths", "deleted,", size, unit, "freed"] => {
                let freed = format!("{} {}", size, unit);
                tracing::info!(%deleted, %freed, "completed collection");
            }
            _ => {
                tracing::warn!(stdout = %line, "unrecognized output from nix-store --gc");
            }
        }
    };

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
