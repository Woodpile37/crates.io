use anyhow::anyhow;
use clap::Parser;
use crates_io_tarball::process_tarball;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs::File;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::EnvFilter;
use walkdir::WalkDir;

/// Runs through all crate files in a folder and shows parsing errors.
#[derive(clap::Parser, Debug, Clone)]
pub struct Options {
    /// Path to the folder to scan for crate files
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    setup_tracing();

    let options = Options::parse();

    let path = options.path;
    if !path.is_dir() {
        return Err(anyhow!("`{}` not found or not a directory", path.display()));
    }

    info!(path = %path.display(), "Searching for crate files");

    let pb = ProgressBar::new(u64::MAX);
    pb.set_style(ProgressStyle::with_template("{human_pos} crate files found").unwrap());

    let mut paths = WalkDir::new(path)
        .into_iter()
        .par_bridge()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.into_path())
        .filter(|path| path.is_file() && path.extension().unwrap_or_default() == "crate")
        .progress_with(pb)
        .collect::<Vec<_>>();

    paths.par_sort();

    let num_files = paths.len();
    info!(%num_files, "Processing crate files");

    let pb = ProgressBar::new(num_files as u64);
    pb.set_style(
        ProgressStyle::with_template("{bar:60} ({pos}/{len}, ETA {eta}) {wide_msg}").unwrap(),
    );

    paths
        .par_iter()
        .progress_with(pb.clone())
        .for_each(|path| process_path(path, &pb));

    Ok(())
}

fn process_path(path: &Path, pb: &ProgressBar) {
    let file =
        File::open(path).map_err(|error| pb.suspend(|| warn!(%error, "Failed to read crate file")));

    let Ok(file) = file else {
        return;
    };

    let path_no_ext = path.with_extension("");
    let pkg_name = path_no_ext.file_name().unwrap().to_string_lossy();
    pb.set_message(format!("{pkg_name}"));

    let result = process_tarball(&pkg_name, &file, u64::MAX);
    pb.suspend(|| match result {
        Ok(result) => debug!(%pkg_name, path = %path.display(), ?result),
        Err(error) => warn!(%pkg_name, path = %path.display(), %error, "Failed to process tarball"),
    })
}

fn setup_tracing() {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .without_time()
        .with_target(false)
        .init();
}
