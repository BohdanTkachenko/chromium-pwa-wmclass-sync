use crate::logic::{
    generate_new_filename, get_final_filepath, get_fixed_content, is_chromium_pwa,
    needs_wmclass_fix,
};
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};

mod logic;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Rename .desktop files to match application names
    #[arg(long, default_value_t = false)]
    rename: bool,

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

fn process_file(file_path: &Path, args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    log::debug!(
        "Checking file: {:?}",
        file_path.file_name().unwrap_or_default()
    );

    let content = fs::read_to_string(file_path)?;

    if !is_chromium_pwa(&content) {
        log::debug!(" -> Skipped: Not a Chromium PWA.");
        return Ok(());
    }

    // Fix StartupWMClass
    if needs_wmclass_fix(&content) {
        if args.dry_run {
            log::info!(
                "DRY RUN: {:?}: Would change StartupWMClass.",
                file_path.file_name().unwrap_or_default()
            );
        } else if let Some(fixed_content) = get_fixed_content(&content) {
            fs::write(file_path, fixed_content)?;
            log::info!(
                "MODIFIED: {:?}: Updated StartupWMClass.",
                file_path.file_name().unwrap_or_default()
            );
        }
    } else {
        log::info!(
            "OK: {:?}: StartupWMClass already correct.",
            file_path.file_name().unwrap_or_default()
        );
    }

    // Rename file
    if args.rename {
        if let Some(new_filename) = generate_new_filename(&content) {
            let target_path = file_path.with_file_name(&new_filename);
            let final_path = get_final_filepath(&target_path, Some(file_path));

            if file_path != final_path {
                if final_path.file_name() != target_path.file_name() {
                    log::warn!(
                        " -> Target file {:?} already exists. Will use {:?} instead.",
                        new_filename,
                        final_path.file_name().unwrap_or_default()
                    );
                }

                if args.dry_run {
                    log::info!(
                        "DRY RUN: {:?}: Would be renamed to {:?}.",
                        file_path.file_name().unwrap_or_default(),
                        final_path.file_name().unwrap_or_default()
                    );
                } else {
                    fs::rename(file_path, &final_path)?;
                    log::info!(
                        "RENAMED: {:?} to {:?}.",
                        file_path.file_name().unwrap_or_default(),
                        final_path.file_name().unwrap_or_default()
                    );
                }
            } else {
                log::info!(
                    "OK: {:?}: Filename already correct.",
                    file_path.file_name().unwrap_or_default()
                );
            }
        }
    }

    Ok(())
}

fn process_directory(directory: &Path, args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    if !directory.is_dir() {
        log::debug!("Directory not found: {:?}", directory);
        return Ok(());
    }

    log::debug!("Scanning directory: {:?}", directory);

    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("desktop") {
            if let Err(e) = process_file(&path, args) {
                log::error!(
                    " -> Error processing {:?}: {}",
                    path.file_name().unwrap_or_default(),
                    e
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

    let apps_dir = args.apps_dir.as_ref().cloned().unwrap_or(default_apps_dir);

    if let Err(e) = process_directory(&apps_dir, &args) {
        log::error!("Fatal error scanning directory {:?}: {}", apps_dir, e);
        std::process::exit(1);
    }
}
