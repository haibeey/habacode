use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

pub mod code;
pub mod config;

const DEFAULT_MAX_DEPTH: usize = 50;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let mut path: Option<PathBuf> = None;
    let mut max_depth: usize = DEFAULT_MAX_DEPTH;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-d" | "--max-depth" => {
                let Some(v) = args.next() else {
                    eprintln!("error: {arg} requires a value");
                    return ExitCode::from(2);
                };

                match v.parse() {
                    Ok(n) => max_depth = n,
                    Err(_) => {
                        eprintln!("error: invalid value for {arg}: {v}");
                        return ExitCode::from(2);
                    }
                }
            }

            _ if path.is_none() => path = Some(PathBuf::from(arg)),
            _ => {
                eprintln!("error: unexpected argument: {arg}");
                return ExitCode::from(2);
            }
        }
    }

    let Some(path) = path else {
        eprintln!("usage: habacode <path> [-d|--max-depth N]");
        return ExitCode::from(2);
    };

    let totals = match code::walk(&path, max_depth) {
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
        println!("{:<width$}  {} LOC", lang, n, width = name_w);
    }

    ExitCode::SUCCESS
}
