// main.rs — clean orchestrator, now config-driven.

mod types;
mod reader;
mod diff;
mod render;
mod config;

use std::path::Path;
use config::Config;

fn main() {
    let cfg = Config::from_args();

    if !Path::new(&cfg.old_path).exists() {
        eprintln!("Error: '{}' does not exist.", cfg.old_path);
        std::process::exit(1);
    }
    if !Path::new(&cfg.new_path).exists() {
        eprintln!("Error: '{}' does not exist.", cfg.new_path);
        std::process::exit(1);
    }

    if cfg.old_path == cfg.new_path {
        println!("Files are identical (same path).");
        std::process::exit(0);
    }

    let old_file = match reader::read_file(&cfg.old_path) {
        Ok(f) => f,
        Err(e) => { eprintln!("Error: {}", e); std::process::exit(1); }
    };
    let new_file = match reader::read_file(&cfg.new_path) {
        Ok(f) => f,
        Err(e) => { eprintln!("Error: {}", e); std::process::exit(1); }
    };

    if old_file.lines.is_empty() && new_file.lines.is_empty() {
        println!("Both files are empty.");
        std::process::exit(0);
    }

    let diff_lines = diff::diff(&old_file.lines, &new_file.lines);
    render::render(&old_file.path, &new_file.path, &diff_lines, &cfg);
}