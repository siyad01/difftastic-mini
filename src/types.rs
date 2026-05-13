// types.rs — shared data structures. No logic, just shapes.
// This file is the vocabulary every other module speaks.

// ─────────────────────────────────────────────
// WORD-LEVEL TYPES
// Used inside Changed lines to show exactly which words shifted.
// ─────────────────────────────────────────────

/// A single word's diff status.
///
/// This is a Rust enum with data — more powerful than Go's or Python's enums.
/// In TypeScript this would be a discriminated union:
///   type WordDiff =
///     | { kind: "equal",    text: string }
///     | { kind: "inserted", text: string }
///     | { kind: "deleted",  text: string }
///
/// Each variant *carries its own data*. Not just a label — a label + payload.
#[derive(Debug, Clone)]
pub enum WordDiff {
    Equal(String),
    Inserted(String),
    Deleted(String),
}

// ─────────────────────────────────────────────
// LINE-LEVEL TYPES
// The top-level result of comparing two files.
// ─────────────────────────────────────────────

/// One line's diff status — the output of the Myers algorithm.
///
/// In Go this might be:
///   type DiffLine struct {
///       Kind    string  // "equal" | "inserted" | "deleted" | "changed"
///       OldLine *string
///       NewLine *string
///       Words   []WordDiff
///   }
///
/// But Rust's enum is cleaner — each variant carries exactly the fields it needs.
/// An Equal line needs both sides. A Deleted line only needs the old text.
/// With a struct you'd have nullable fields. With enum, each variant is precise.
#[derive(Debug, Clone)]
pub enum DiffLine {
    Equal {
        old_num: usize,
        new_num: usize,
        text: String,
    },

    Deleted {
        old_num: usize,
        text: String,
    },

    Inserted {
        new_num: usize,
        text: String,
    },

    Changed {
        old_num: usize,
        new_num: usize,
        old_words: Vec<WordDiff>,
        new_words: Vec<WordDiff>,
    },
}

// ─────────────────────────────────────────────
// FILE-LEVEL TYPES
// ─────────────────────────────────────────────

/// A file loaded into memory — output of reader.rs
pub struct FileContent {
    pub path: String,       // the file path as given on CLI
    pub lines: Vec<String>, // every line, in order, without the \n
}
