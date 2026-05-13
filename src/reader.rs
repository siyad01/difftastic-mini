// reader.rs — single responsibility: read a file from disk → FileContent
// Knows about files. Knows nothing about diffs or terminals.

use std::fs;
use crate::types::FileContent;

// In Go this would be:
//   func ReadFile(path string) (FileContent, error)
//
// In Python:
//   def read_file(path: str) -> FileContent
//
// Rust's version returns Result<FileContent, String>
// That's Go's (value, error) pattern — but the compiler *forces* you to handle it.
// You cannot ignore a Result. If you try, it won't compile.
pub fn read_file(path: &str) -> Result<FileContent, String> {

    // fs::read_to_string reads the whole file as one String.
    // The ? at the end means: "if this is an error, return it immediately."
    // Same as Go's:  if err != nil { return nil, err }
    // But shorter — the ? does that automatically.
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("Cannot read '{}': {}", path, e))?;

    // Split the file into lines.
    // .lines() is an iterator — like Python's str.splitlines()
    // .map(str::to_string) converts each &str slice → owned String
    // .collect() gathers the iterator into a Vec<String>
    // Think: list(map(str, raw.splitlines())) in Python
    let lines: Vec<String> = raw.lines()
        .map(str::to_string)
        .collect();

    Ok(FileContent {
        path: path.to_string(),
        lines,
    })
}