# Contributing to difftastic-mini

Thank you for your interest. This project is intentionally small and focused.
Before opening a PR, please read this document fully.

## Philosophy

difftastic-mini has one job: show exactly what changed, at the word level, in a terminal.
Every contribution should make that job clearer, faster, or more correct.
Features that expand scope significantly belong in a fork.

## Getting started

```bash
git clone https://github.com/siyad01/difftastic-mini
cd difftastic-mini
cargo build
cargo test
```

All 8 tests must pass before and after your change.

## Project structure

| File | Responsibility | Change if you are... |
|------|---------------|----------------------|
| `src/main.rs` | Entry point, orchestration | Adding a new top-level flag |
| `src/config.rs` | CLI flag parsing | Adding or changing a flag |
| `src/reader.rs` | File I/O | Changing how files are read |
| `src/diff.rs` | Diff algorithm + tests | Fixing diff correctness |
| `src/render.rs` | Terminal output | Changing visual layout or colors |
| `src/types.rs` | Shared data structures | Adding a new diff variant |

One responsibility per file. If your change touches more than two files, check whether
you are adding a feature or changing an abstraction. If the latter, open an issue first.

## How to add a feature

1. Open an issue describing the problem it solves and who is blocked without it
2. Fork the repo and create a branch: `git checkout -b feature/your-feature-name`
3. Write the test first — in `src/diff.rs` for algorithm changes, or a manual test case for render changes
4. Write the minimum code that makes the test pass
5. Run `cargo test` — all tests must pass
6. Run `cargo clippy` — no warnings allowed
7. Open a PR with a description that explains the problem, not the implementation

## How to report a bug

Open an issue with:
- The exact command you ran
- The contents of both files (or a minimal reproduction)
- The output you got
- The output you expected

If you can reproduce it with a `#[test]` case in `src/diff.rs`, include that — it will be merged immediately.

## Code style

- No `unwrap()` in production code paths — use `match` or `?`
- Every public function gets a one-line doc comment (`///`)
- No external dependency added without opening an issue first explaining: what it does, why this one, what it costs in binary size
- `cargo fmt` before committing — the project uses default rustfmt settings

## What will be accepted

- Bug fixes with a regression test
- Performance improvements with a benchmark
- New flags that follow the existing `config.rs` pattern
- Improved word tokenization (punctuation-aware splitting)
- Better error messages

## What will not be accepted

- Directory diff support (scope expansion — consider a fork)
- GUI or web output
- Dependencies that significantly increase binary size without clear user value
- Breaking changes to existing flags

## Running the full check before a PR

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo build --release
```

All four must succeed with zero warnings or errors.