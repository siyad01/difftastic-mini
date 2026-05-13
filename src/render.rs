// render.rs — terminal renderer, now config-driven.

use crossterm::{
    style::{Color, ResetColor, SetForegroundColor, Attribute, SetAttribute},
    ExecutableCommand,
};
use std::io::{stdout, Write};
use crate::types::{DiffLine, WordDiff};
use crate::config::Config;

// ─────────────────────────────────────────────────────────────
// PUBLIC ENTRY POINT
// ─────────────────────────────────────────────────────────────

pub fn render(old_path: &str, new_path: &str, lines: &[DiffLine], cfg: &Config) {
    if cfg.unified {
        render_unified(old_path, new_path, lines, cfg);
    } else {
        render_sidebyside(old_path, new_path, lines, cfg);
    }
}

// ─────────────────────────────────────────────────────────────
// SIDE-BY-SIDE RENDERER
// ─────────────────────────────────────────────────────────────

fn render_sidebyside(old_path: &str, new_path: &str, lines: &[DiffLine], cfg: &Config) {
    let mut out = stdout();

    // Identical check
    let all_equal = lines.iter().all(|l| matches!(l, DiffLine::Equal { .. }));
    if lines.is_empty() || all_equal {
        set_color(&mut out, Color::Green, cfg);
        writeln!(out, "Files are identical.").unwrap();
        reset_color(&mut out, cfg);
        return;
    }

    let panel  = cfg.width;
    let num_w  = 3usize;
    let cont_w = panel - num_w - 2;

    print_sep(&mut out, panel, cfg);
    // Header
    set_color(&mut out, Color::White, cfg);
    writeln!(out, " {:<width$} │  {}", old_path, new_path, width = panel - 1).unwrap();
    reset_color(&mut out, cfg);
    print_sep(&mut out, panel, cfg);

    for line in lines {
        print_line(&mut out, line, panel, num_w, cont_w, cfg);
    }

    print_sep(&mut out, panel, cfg);
    out.flush().unwrap();
}

fn print_sep(out: &mut impl Write, panel: usize, cfg: &Config) {
    set_color(out, Color::DarkGrey, cfg);
    writeln!(out, "{}", "─".repeat(panel * 2 + 3)).unwrap();
    reset_color(out, cfg);
}

fn print_line(
    out: &mut impl Write,
    line: &DiffLine,
    panel: usize,
    num_w: usize,
    cont_w: usize,
    cfg: &Config,
) {
    match line {
        DiffLine::Equal { old_num, new_num, text } => {
            set_color(out, Color::DarkGrey, cfg);
            let left = format!("{:>num_w$} {}", old_num, truncate(text, cont_w));
            write!(out, "{:<panel$}", left).unwrap();
            write!(out, " │ ").unwrap();
            let right = format!("{:>num_w$} {}", new_num, truncate(text, cont_w));
            write!(out, "{}", right).unwrap();
            reset_color(out, cfg);
            writeln!(out).unwrap();
        }

        DiffLine::Deleted { old_num, text } => {
            set_color(out, Color::Red, cfg);
            set_bold(out, cfg);
            let left = format!("{:>num_w$} {}", old_num, truncate(text, cont_w));
            write!(out, "{:<panel$}", left).unwrap();
            reset_bold(out, cfg);
            set_color(out, Color::DarkGrey, cfg);
            write!(out, " │ ").unwrap();
            write!(out, "{:<panel$}", "").unwrap();
            reset_color(out, cfg);
            writeln!(out).unwrap();
        }

        DiffLine::Inserted { new_num, text } => {
            set_color(out, Color::DarkGrey, cfg);
            write!(out, "{:<panel$}", "").unwrap();
            write!(out, " │ ").unwrap();
            set_color(out, Color::Green, cfg);
            set_bold(out, cfg);
            let right = format!("{:>num_w$} {}", new_num, truncate(text, cont_w));
            write!(out, "{}", right).unwrap();
            reset_bold(out, cfg);
            reset_color(out, cfg);
            writeln!(out).unwrap();
        }

        DiffLine::Changed { old_num, new_num, old_words, new_words } => {
            // Left panel
            set_color(out, Color::DarkGrey, cfg);
            write!(out, "{:>num_w$} ", old_num).unwrap();
            let mut used = 0usize;
            print_words(out, old_words, Color::Red, cont_w, &mut used, cfg);
            let pad = cont_w.saturating_sub(used);
            write!(out, "{}", " ".repeat(pad)).unwrap();

            // Separator
            set_color(out, Color::DarkGrey, cfg);
            write!(out, " │ ").unwrap();

            // Right panel
            set_color(out, Color::DarkGrey, cfg);
            write!(out, "{:>num_w$} ", new_num).unwrap();
            let mut used = 0usize;
            print_words(out, new_words, Color::Green, cont_w, &mut used, cfg);

            reset_color(out, cfg);
            writeln!(out).unwrap();
        }
    }
}

fn print_words(
    out: &mut impl Write,
    words: &[WordDiff],
    highlight: Color,
    max_chars: usize,
    used: &mut usize,
    cfg: &Config,
) {
    for (idx, word) in words.iter().enumerate() {
        let space = if idx > 0 { " " } else { "" };
        let (text, color, bold) = match word {
            WordDiff::Equal(s)    => (s.as_str(), Color::White, false),
            WordDiff::Deleted(s)  => (s.as_str(), highlight,    true),
            WordDiff::Inserted(s) => (s.as_str(), highlight,    true),
        };
        let token = format!("{}{}", space, text);
        let tlen  = token.chars().count();

        if *used + tlen > max_chars {
            let rem = max_chars.saturating_sub(*used);
            if rem > 1 {
                let t: String = token.chars().take(rem - 1).collect();
                set_color(out, color, cfg);
                write!(out, "{}…", t).unwrap();
                *used = max_chars;
            }
            break;
        }

        if bold { set_bold(out, cfg); }
        set_color(out, color, cfg);
        write!(out, "{}", token).unwrap();
        if bold { reset_bold(out, cfg); }
        *used += tlen;
    }
}

// ─────────────────────────────────────────────────────────────
// UNIFIED RENDERER
// Classic unified diff format — good for patch files and CI output
// ─────────────────────────────────────────────────────────────

fn render_unified(old_path: &str, new_path: &str, lines: &[DiffLine], cfg: &Config) {
    let mut out = stdout();

    set_color(&mut out, Color::White, cfg);
    writeln!(out, "--- {}", old_path).unwrap();
    writeln!(out, "+++ {}", new_path).unwrap();
    reset_color(&mut out, cfg);

    for line in lines {
        match line {
            DiffLine::Equal { text, .. } => {
                writeln!(out, " {}", text).unwrap();
            }
            DiffLine::Deleted { text, .. } => {
                set_color(&mut out, Color::Red, cfg);
                writeln!(out, "-{}", text).unwrap();
                reset_color(&mut out, cfg);
            }
            DiffLine::Inserted { text, .. } => {
                set_color(&mut out, Color::Green, cfg);
                writeln!(out, "+{}", text).unwrap();
                reset_color(&mut out, cfg);
            }
            DiffLine::Changed { old_words, new_words, .. } => {
                set_color(&mut out, Color::Red, cfg);
                write!(out, "-").unwrap();
                for w in old_words {
                    match w {
                        WordDiff::Equal(s) | WordDiff::Deleted(s) => write!(out, "{} ", s).unwrap(),
                        _ => {}
                    }
                }
                writeln!(out).unwrap();

                set_color(&mut out, Color::Green, cfg);
                write!(out, "+").unwrap();
                for w in new_words {
                    match w {
                        WordDiff::Equal(s) | WordDiff::Inserted(s) => write!(out, "{} ", s).unwrap(),
                        _ => {}
                    }
                }
                writeln!(out).unwrap();
                reset_color(&mut out, cfg);
            }
        }
    }
    out.flush().unwrap();
}

// ─────────────────────────────────────────────────────────────
// COLOR HELPERS — all color calls go through these.
// When cfg.color = false, they do nothing. This is how --no-color works:
// one flag, zero code changes in the rendering logic above.
// ─────────────────────────────────────────────────────────────

fn set_color(out: &mut impl Write, color: Color, cfg: &Config) {
    if cfg.color {
        out.execute(SetForegroundColor(color)).unwrap();
    }
}

fn reset_color(out: &mut impl Write, cfg: &Config) {
    if cfg.color {
        out.execute(ResetColor).unwrap();
    }
}

fn set_bold(out: &mut impl Write, cfg: &Config) {
    if cfg.color {
        out.execute(SetAttribute(Attribute::Bold)).unwrap();
    }
}

fn reset_bold(out: &mut impl Write, cfg: &Config) {
    if cfg.color {
        out.execute(SetAttribute(Attribute::Reset)).unwrap();
    }
}

// ─────────────────────────────────────────────────────────────
// UTILITIES
// ─────────────────────────────────────────────────────────────

fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max_len - 1).collect::<String>())
    }
}