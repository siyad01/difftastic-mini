// diff.rs — diff engine using `similar` (Myers diff, same as cargo uses).
// Fixed: hunk-based pairing of Deletes and Inserts instead of single look-ahead.

use similar::{ChangeTag, TextDiff};
use crate::types::{DiffLine, WordDiff};

pub fn diff(old_lines: &[String], new_lines: &[String]) -> Vec<DiffLine> {
    let old_text = old_lines.join("\n");
    let new_text = new_lines.join("\n");

    let text_diff = TextDiff::from_lines(&old_text, &new_text);

    // Collect all raw changes first
    // Each entry: (tag, text, old_line_num, new_line_num)
    // We track line numbers ourselves as we scan
    let mut raw: Vec<(ChangeTag, String)> = Vec::new();

    for change in text_diff.iter_all_changes() {
        let text = change.value().trim_end_matches('\n').to_string();
        raw.push((change.tag(), text));
    }

    // Now walk through raw changes, grouping consecutive Deletes+Inserts
    // into hunks, then pairing them as Changed lines.
    let mut result: Vec<DiffLine> = Vec::new();
    let mut old_num = 1usize;
    let mut new_num = 1usize;
    let mut i = 0;

    while i < raw.len() {
        match raw[i].0 {
            ChangeTag::Equal => {
                result.push(DiffLine::Equal {
                    old_num,
                    new_num,
                    text: raw[i].1.clone(),
                });
                old_num += 1;
                new_num += 1;
                i += 1;
            }

            // Collect the full hunk: all consecutive Deletes then Inserts
            ChangeTag::Delete | ChangeTag::Insert => {
                // Gather all deletes starting at i
                let mut deletes: Vec<String> = Vec::new();
                while i < raw.len() && raw[i].0 == ChangeTag::Delete {
                    deletes.push(raw[i].1.clone());
                    i += 1;
                }

                // Gather all inserts immediately following
                let mut inserts: Vec<String> = Vec::new();
                while i < raw.len() && raw[i].0 == ChangeTag::Insert {
                    inserts.push(raw[i].1.clone());
                    i += 1;
                }

                // Pair them up: min(deletes, inserts) become Changed lines
                // Remaining unpaired ones become pure Deleted or Inserted
                let pairs = deletes.len().min(inserts.len());

                for p in 0..pairs {
                    let (old_words, new_words) =
                        word_diff(&deletes[p], &inserts[p]);
                    result.push(DiffLine::Changed {
                        old_num,
                        new_num,
                        old_words,
                        new_words,
                    });
                    old_num += 1;
                    new_num += 1;
                }

                // Leftover deletes (more old lines than new)
                for d in deletes.iter().skip(pairs) {
                    result.push(DiffLine::Deleted {
                        old_num,
                        text: d.clone(),
                    });
                    old_num += 1;
                }

                // Leftover inserts (more new lines than old)
                for ins in inserts.iter().skip(pairs) {
                    result.push(DiffLine::Inserted {
                        new_num,
                        text: ins.clone(),
                    });
                    new_num += 1;
                }
            }
        }
    }

    result
}

pub fn word_diff(old: &str, new: &str) -> (Vec<WordDiff>, Vec<WordDiff>) {
    let old_joined = old.split_whitespace().collect::<Vec<_>>().join("\n");
    let new_joined = new.split_whitespace().collect::<Vec<_>>().join("\n");

    let wd = TextDiff::from_lines(&old_joined, &new_joined);

    let mut old_result: Vec<WordDiff> = Vec::new();
    let mut new_result: Vec<WordDiff> = Vec::new();

    for change in wd.iter_all_changes() {
        let word = change.value().trim_end_matches('\n').to_string();
        if word.is_empty() { continue; }
        match change.tag() {
            ChangeTag::Equal  => {
                old_result.push(WordDiff::Equal(word.clone()));
                new_result.push(WordDiff::Equal(word));
            }
            ChangeTag::Delete => old_result.push(WordDiff::Deleted(word)),
            ChangeTag::Insert => new_result.push(WordDiff::Inserted(word)),
        }
    }

    (old_result, new_result)
}

// ─────────────────────────────────────────────────────────────
// TESTS
// In Rust, tests live right next to the code they test.
// `#[cfg(test)]` means this block only compiles during `cargo test`.
// This is like Python's unittest or Go's _test.go files —
// but colocated rather than separate.
// ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;  // import everything from the parent module (diff.rs)
    use crate::types::DiffLine;

    // Helper: turn a slice of &str into Vec<String>
    // We use this everywhere so tests stay readable
    fn lines(input: &[&str]) -> Vec<String> {
        input.iter().map(|s| s.to_string()).collect()
    }

    // ── PROPERTY: identical files produce only Equal lines ────

    #[test]
    fn identical_files_are_all_equal() {
        let content = lines(&["fn main() {", "    println!(\"hi\");", "}"]);
        let result = diff(&content, &content);

        // Every line must be Equal — no deletions or insertions
        for line in &result {
            assert!(
                matches!(line, DiffLine::Equal { .. }),
                "Expected Equal, got {:?}", line
            );
        }
        assert_eq!(result.len(), 3);
    }

    // ── PROPERTY: empty old file = all insertions ─────────────

    #[test]
    fn empty_old_all_insertions() {
        let old = lines(&[]);
        let new = lines(&["line one", "line two", "line three"]);
        let result = diff(&old, &new);

        assert_eq!(result.len(), 3);
        for line in &result {
            assert!(
                matches!(line, DiffLine::Inserted { .. }),
                "Expected Inserted, got {:?}", line
            );
        }
    }

    // ── PROPERTY: empty new file = all deletions ──────────────

    #[test]
    fn empty_new_all_deletions() {
        let old = lines(&["line one", "line two"]);
        let new = lines(&[]);
        let result = diff(&old, &new);

        assert_eq!(result.len(), 2);
        for line in &result {
            assert!(
                matches!(line, DiffLine::Deleted { .. }),
                "Expected Deleted, got {:?}", line
            );
        }
    }

    // ── PROPERTY: both empty = no diff lines at all ───────────

    #[test]
    fn both_empty_no_lines() {
        let result = diff(&lines(&[]), &lines(&[]));
        assert_eq!(result.len(), 0);
    }

    // ── PROPERTY: line numbers are correct and 1-based ────────

    #[test]
    fn line_numbers_are_one_based() {
        let old = lines(&["a", "b", "c"]);
        let new = lines(&["a", "x", "c"]);
        let result = diff(&old, &new);

        // Line 1: "a" = Equal, old_num=1, new_num=1
        assert!(matches!(
            &result[0],
            DiffLine::Equal { old_num: 1, new_num: 1, .. }
        ));

        // Line 2: "b" vs "x" = Changed, old_num=2, new_num=2
        assert!(matches!(
            &result[1],
            DiffLine::Changed { old_num: 2, new_num: 2, .. }
        ));

        // Line 3: "c" = Equal, old_num=3, new_num=3
        assert!(matches!(
            &result[2],
            DiffLine::Equal { old_num: 3, new_num: 3, .. }
        ));
    }

    // ── PROPERTY: pure insertion shifts new line numbers ──────

    #[test]
    fn insertion_shifts_new_line_numbers() {
        let old = lines(&["a", "c"]);
        let new = lines(&["a", "b", "c"]);  // "b" inserted in middle
        let result = diff(&old, &new);

        // "a" equal at 1,1
        assert!(matches!(
            &result[0],
            DiffLine::Equal { old_num: 1, new_num: 1, .. }
        ));

        // "b" inserted — only new_num, should be 2
        assert!(matches!(
            &result[1],
            DiffLine::Inserted { new_num: 2, .. }
        ));

        // "c" equal — old_num=2, new_num=3 (shifted by insertion)
        assert!(matches!(
            &result[2],
            DiffLine::Equal { old_num: 2, new_num: 3, .. }
        ));
    }

    // ── PROPERTY: word diff marks correct words ───────────────

    #[test]
    fn word_diff_marks_changed_words() {
        let (old_words, new_words) =
            word_diff("hello world foo", "hello rust foo");

        // "hello" should be Equal on both sides
        assert!(matches!(&old_words[0], crate::types::WordDiff::Equal(s) if s == "hello"));
        assert!(matches!(&new_words[0], crate::types::WordDiff::Equal(s) if s == "hello"));

        // "world" deleted from old, "rust" inserted in new
        assert!(matches!(&old_words[1], crate::types::WordDiff::Deleted(s) if s == "world"));
        assert!(matches!(&new_words[1], crate::types::WordDiff::Inserted(s) if s == "rust"));

        // "foo" equal on both sides
        assert!(matches!(old_words.last().unwrap(), crate::types::WordDiff::Equal(s) if s == "foo"));
        assert!(matches!(new_words.last().unwrap(), crate::types::WordDiff::Equal(s) if s == "foo"));
    }

    // ── PROPERTY: total line count is conserved ───────────────
    // old_lines + inserted = new_lines + deleted
    // This is a fundamental invariant of any correct diff algorithm.

    #[test]
    fn line_count_invariant() {
        let old = lines(&["a", "b", "c", "d"]);
        let new = lines(&["a", "x", "y", "d", "e"]);
        let result = diff(&old, &new);

        let mut old_count = 0usize;
        let mut new_count = 0usize;

        for line in &result {
            match line {
                DiffLine::Equal    { .. } => { old_count += 1; new_count += 1; }
                DiffLine::Deleted  { .. } => { old_count += 1; }
                DiffLine::Inserted { .. } => { new_count += 1; }
                DiffLine::Changed  { .. } => { old_count += 1; new_count += 1; }
            }
        }

        assert_eq!(old_count, old.len(), "old line count mismatch");
        assert_eq!(new_count, new.len(), "new line count mismatch");
    }
}