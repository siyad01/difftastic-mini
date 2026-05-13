// config.rs — single responsibility: parse CLI arguments into a Config struct.
// main.rs reads this once and passes it everywhere. Nothing else touches args.

/// All runtime configuration derived from CLI flags.
/// Add new flags here — never read env::args() anywhere else.
pub struct Config {
    pub old_path:  String,
    pub new_path:  String,
    pub color:     bool,   // --no-color disables ANSI escapes
    pub width:     usize,  // --width=N sets panel width (default 60)
    pub unified:   bool,   // --unified: classic unified diff style instead of side-by-side
}

impl Config {
    /// Parse args and return Config, or print usage and exit.
    /// In Go this would use the `flag` package.
    /// In Python, `argparse`. In Rust we do it manually here
    /// (or use `clap` — we keep it manual so there are zero new deps).
    pub fn from_args() -> Config {
        let args: Vec<String> = std::env::args().collect();

        // Defaults
        let mut color   = true;
        let mut width   = 60usize;
        let mut unified = false;
        let mut paths: Vec<String> = Vec::new();

        // Walk args, skip the binary name (args[0])
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--no-color" => {
                    color = false;
                }
                "--unified" => {
                    unified = true;
                }
                s if s.starts_with("--width=") => {
                    // "--width=80" → parse the part after "="
                    let val = &s["--width=".len()..];
                    width = val.parse().unwrap_or_else(|_| {
                        eprintln!("Error: --width must be a number, got '{}'", val);
                        std::process::exit(1);
                    });
                    if width < 20 {
                        eprintln!("Error: --width must be at least 20");
                        std::process::exit(1);
                    }
                }
                s if s.starts_with("--") => {
                    eprintln!("Unknown flag: {}", s);
                    print_usage();
                    std::process::exit(1);
                }
                // Not a flag — must be a file path
                _ => paths.push(args[i].clone()),
            }
            i += 1;
        }

        if paths.len() != 2 {
            print_usage();
            std::process::exit(1);
        }

        Config {
            old_path: paths[0].clone(),
            new_path: paths[1].clone(),
            color,
            width,
            unified,
        }
    }
}

fn print_usage() {
    eprintln!("Usage: difftastic-mini [OPTIONS] <old_file> <new_file>");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --no-color        Disable colored output (good for piping to files)");
    eprintln!("  --width=N         Panel width in characters (default: 60, min: 20)");
    eprintln!("  --unified         Unified diff style instead of side-by-side");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  difftastic-mini old.txt new.txt");
    eprintln!("  difftastic-mini --no-color old.txt new.txt > diff.txt");
    eprintln!("  difftastic-mini --width=80 old.txt new.txt");
    eprintln!("  difftastic-mini --unified old.txt new.txt");
}