use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;

pub mod code;
pub mod config;

const DEFAULT_MAX_DEPTH: usize = 50;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to folder to count LOC.
    path: Option<PathBuf>,

    /// Depth of folders to traverse.
    #[arg(short, long, default_value_t = DEFAULT_MAX_DEPTH)]
    max_depth: usize,

    /// Text file containing list of files to ignore.
    #[arg(short, long, default_value = ".gitignore")]
    ignore_file: String,
}

fn normalize(path: PathBuf) -> PathBuf {
    let mut result = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                result.pop();
            }
            other => result.push(other.as_os_str()),
        }
    }

    result
}

fn read_ignore_file(
    root_path: impl AsRef<Path>,
    path: impl AsRef<Path>,
) -> io::Result<HashSet<PathBuf>> {
    let path = path.as_ref();

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let base_dir = root_path.as_ref().canonicalize()?;
    let mut ignores = HashSet::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let expanded = normalize(base_dir.join(line));
        ignores.insert(expanded);
    }

    Ok(ignores)
}

fn format_size(n: usize) -> String {
    match n {
        n if n < 1_000 => n.to_string(),
        n if n < 1_000_000 => format!("{:.0}K", n as f64 / 1_000.0),
        n if n < 1_000_000_000 => format!("{:.0}M", n as f64 / 1_000_000.0),
        n => format!("{:.0}B", n as f64 / 1_000_000_000.0),
    }
}

fn main() -> ExitCode {
    let args = Args::parse();

    let Some(path) = args.path else {
        eprintln!("usage: habacode <path> [-d|--max-depth N]");
        return ExitCode::from(2);
    };

    let ignores = read_ignore_file(&path, &args.ignore_file).unwrap_or_default();
    let full_path = match path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let totals = match code::walk(&full_path, args.max_depth, &ignores) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    if totals.is_empty() {
        println!("No files found");
        return ExitCode::SUCCESS;
    }

    let mut rows: Vec<(&'static str, usize)> = totals.into_iter().collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));

    let name_w = rows.iter().map(|(n, _)| n.len()).max().unwrap_or(0);

    for (lang, n) in rows {
        println!("{:<width$}  {} LOC", lang, format_size(n), width = name_w);
    }

    ExitCode::SUCCESS
}
