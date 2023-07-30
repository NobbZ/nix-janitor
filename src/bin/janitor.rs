use std::{
    future::Future,
    path::{Path, PathBuf},
    process::Stdio,
};

use chrono::{prelude::*, Duration};
use eyre::Result;
use futures::future::try_join_all;
use lazy_static::lazy_static;
use tokio::{process::Command, sync::Mutex};
use tracing::{Instrument, Level};
use tracing_subscriber::{fmt::format::FmtSpan, FmtSubscriber};

use janitor::{Generation, GenerationSet};

const VERSION: &str = env!("CARGO_PKG_VERSION");

lazy_static! {
    static ref DATE: Mutex<NaiveDateTime> = Mutex::new(Utc::now().naive_utc());
    static ref COUNT: usize = 5;
}

#[tokio::main]
async fn main() -> Result<()> {
    // Configure and initialize logging
    FmtSubscriber::builder()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_max_level(Level::TRACE)
        .init();

    // Configure thresholds and "print welcome"
    let mut date = DATE.lock().await;
    let count = *COUNT;
    tracing::info!(start_time = %date, version = VERSION, "Starting janitor");
    *date -= Duration::days(7);
    tracing::info!(
        keep_since = %date,
        keep_at_least = count,
        "Starting to clean the profiles"
    );
    drop(date); // Drop the Mutex to avoid deadlocks

    try_join_all(
        [
            "/nix/var/nix/profiles/system",
            "/nix/var/nix/profiles/per-user/$USER/profile",
            "/home/$USER/.local/state/nix/profiles/home-manager",
        ]
        .iter()
        .map(|p| -> Result<_> { Ok(shellexpand::env(p)?) })
        .map(|p| -> Result<_> { Ok(p?.to_string()) })
        .map(|p| -> Result<_> { Ok(PathBuf::from(p?)) })
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
async fn get_generations<P>(profile_path: Result<P>) -> Result<(PathBuf, GenerationSet)>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    let path = profile_path?.as_ref().to_owned();

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

    let parsed = Generation::parse_many(std::str::from_utf8(output.stdout.as_ref())?)?;

    Ok((path, parsed.into()))
}

#[tracing::instrument(skip(payload))]
async fn get_to_delete(
    payload: impl Future<Output = Result<(PathBuf, GenerationSet)>>,
) -> Result<(PathBuf, GenerationSet)> {
    let (path, generations) = payload.await?;
    let date = DATE.lock().await;
    let count = *COUNT;

    let to_delete = generations.generations_to_delete(count, *date);

    Ok((path, to_delete))
}

#[tracing::instrument(skip(payload))]
async fn run_delete(payload: impl Future<Output = Result<(PathBuf, GenerationSet)>>) -> Result<()> {
    let (path, generations) = payload.await?;
    let ids: Vec<_> = generations
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

    Ok(())
}
