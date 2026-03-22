use crate::logic::{
    generate_new_filename, get_companion_content, get_final_filepath, get_neutered_content,
    get_sync_source, is_chromium_pwa, is_companion_file, is_managed_original,
};
use clap::Parser;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

mod logic;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Print the changes that would be made without modifying files
    #[arg(long, default_value_t = false)]
    dry_run: bool,

    /// Enable verbose (DEBUG level) logging
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// The directory to scan for .desktop files
    #[arg(long, value_name = "DIR")]
    apps_dir: Option<PathBuf>,
}

fn process_file(
    file_path: &Path,
    content: &str,
    args: &Args,
    companion_map: &HashMap<String, PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let original_filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid filename")?
        .to_string();

    let existing_companion = companion_map.get(&original_filename);

    if is_managed_original(content) {
        // Already neutered — check if companion exists
        if let Some(companion_path) = existing_companion {
            if companion_path.exists() {
                log::info!(
                    "OK: {:?}: Already managed, companion {:?} exists.",
                    original_filename,
                    companion_path.file_name().unwrap_or_default()
                );
                return Ok(());
            }
        }

        // Companion missing — recreate from neutered content
        log::debug!(
            "Recreating missing companion for {:?}",
            original_filename
        );
        let companion_content = get_companion_content(content, &original_filename)
            .ok_or("Failed to generate companion content")?;
        let new_filename = generate_new_filename(&companion_content)
            .ok_or("Failed to generate filename")?;
        let target_path = file_path.with_file_name(&new_filename);
        let final_path = get_final_filepath(&target_path, None);

        if args.dry_run {
            log::info!(
                "DRY RUN: Would create companion {:?} for {:?}.",
                final_path.file_name().unwrap_or_default(),
                original_filename
            );
        } else {
            fs::write(&final_path, companion_content)?;
            log::info!(
                "CREATED: Companion {:?} for {:?}.",
                final_path.file_name().unwrap_or_default(),
                original_filename
            );
        }
        return Ok(());
    }

    // Fresh/regenerated original — full processing
    // Generate companion content BEFORE neutering
    let companion_content = get_companion_content(content, &original_filename)
        .ok_or("Failed to generate companion content")?;
    let new_filename =
        generate_new_filename(content).ok_or("Failed to generate filename")?;
    let target_path = file_path.with_file_name(&new_filename);
    let final_path =
        get_final_filepath(&target_path, existing_companion.map(|p| p.as_path()));

    // Delete old companion if name changed
    if let Some(old_companion) = existing_companion {
        if old_companion != &final_path {
            if args.dry_run {
                log::info!(
                    "DRY RUN: Would delete old companion {:?}.",
                    old_companion.file_name().unwrap_or_default()
                );
            } else {
                fs::remove_file(old_companion)?;
                log::info!(
                    "DELETED: Old companion {:?}.",
                    old_companion.file_name().unwrap_or_default()
                );
            }
        }
    }

    // Write companion
    if args.dry_run {
        log::info!(
            "DRY RUN: Would create companion {:?} for {:?}.",
            final_path.file_name().unwrap_or_default(),
            original_filename
        );
    } else {
        fs::write(&final_path, &companion_content)?;
        log::info!(
            "CREATED: Companion {:?} for {:?}.",
            final_path.file_name().unwrap_or_default(),
            original_filename
        );
    }

    // Neuter original
    let neutered =
        get_neutered_content(content).ok_or("Failed to generate neutered content")?;
    if args.dry_run {
        log::info!(
            "DRY RUN: Would neuter original {:?}.",
            original_filename
        );
    } else {
        fs::write(file_path, neutered)?;
        log::info!(
            "NEUTERED: {:?} (hidden from launcher).",
            original_filename
        );
    }

    Ok(())
}

fn process_directory(
    directory: &Path,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error>> {
    if !directory.is_dir() {
        log::debug!("Directory not found: {:?}", directory);
        return Ok(());
    }

    log::debug!("Scanning directory: {:?}", directory);

    // Pass 1: Scan all .desktop files, build companion map and collect files
    let mut companion_map: HashMap<String, PathBuf> = HashMap::new();
    let mut files_with_content: Vec<(PathBuf, String)> = Vec::new();

    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file()
            && path.extension().and_then(|s| s.to_str()) == Some("desktop")
        {
            let content = fs::read_to_string(&path)?;
            if let Some(source) = get_sync_source(&content) {
                companion_map.insert(source, path.clone());
            }
            files_with_content.push((path, content));
        }
    }

    // Pass 2: Process each Chrome PWA original (skip companion files)
    for (path, content) in &files_with_content {
        if is_companion_file(content) {
            log::debug!(
                "Skipping companion file: {:?}",
                path.file_name().unwrap_or_default()
            );
            continue;
        }
        if !is_chromium_pwa(content) {
            log::debug!(
                "Skipping non-PWA: {:?}",
                path.file_name().unwrap_or_default()
            );
            continue;
        }
        if let Err(e) = process_file(path, content, args, &companion_map) {
            log::error!(
                " -> Error processing {:?}: {}",
                path.file_name().unwrap_or_default(),
                e
            );
        }
    }

    // Pass 3: Delete orphaned companions (original no longer exists)
    for (source_filename, companion_path) in &companion_map {
        let original_path = directory.join(source_filename);
        if !original_path.exists() {
            if args.dry_run {
                log::info!(
                    "DRY RUN: Would delete orphaned companion {:?} (original {:?} no longer exists).",
                    companion_path.file_name().unwrap_or_default(),
                    source_filename
                );
            } else {
                fs::remove_file(companion_path)?;
                log::info!(
                    "DELETED: Orphaned companion {:?} (original {:?} no longer exists).",
                    companion_path.file_name().unwrap_or_default(),
                    source_filename
                );
            }
        }
    }

    Ok(())
}

fn main() {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    env_logger::Builder::new()
        .filter_level(log_level)
        .format_timestamp(None)
        .format_target(false)
        .init();

    log::debug!("Arguments parsed: {:?}", args);

    let default_apps_dir = home::home_dir()
        .map(|p| p.join(".local/share/applications"))
        .expect("Could not determine home directory");

    let apps_dir = args
        .apps_dir
        .as_ref()
        .cloned()
        .unwrap_or(default_apps_dir);

    if let Err(e) = process_directory(&apps_dir, &args) {
        log::error!("Fatal error scanning directory {:?}: {}", apps_dir, e);
        std::process::exit(1);
    }
}
