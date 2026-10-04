#![forbid(unsafe_code)]

mod generate;
mod spec;
mod yaml;

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const USAGE: &str = "\
cargo xtask <command>

Commands:
  generate        Specs -> registry, bindings, wrappers, stubs, docs pages   (M2)
  regen-check     Run generate, then fail if the working tree is dirty       (M2)
  golden <name>   Re-produce the golden CSVs of one indicator from its oracle
  bench           Criterion benchmarks and the Python vs TA-Lib comparison   (M4)
";

/// The milestone that is going to implement each command, per docs/MILESTONES.md.
const PENDING: &[(&str, &str)] = &[("bench", "M4")];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("xtask lives at <repo>/crates/xtask")
        .to_path_buf()
}

/// The interpreter that has the oracle installed: the active virtualenv when
/// there is one, so `maturin develop && cargo xtask golden` uses one environment.
fn python() -> PathBuf {
    if let Ok(explicit) = std::env::var("PYTHON") {
        return PathBuf::from(explicit);
    }
    if let Ok(venv) = std::env::var("VIRTUAL_ENV") {
        for candidate in ["bin/python", "Scripts/python.exe"] {
            let path = Path::new(&venv).join(candidate);
            if path.exists() {
                return path;
            }
        }
    }
    PathBuf::from("python3")
}

fn golden(args: &[String]) -> ExitCode {
    let Some(name) = args.first() else {
        eprintln!("xtask golden: name of the indicator folder is required");
        return ExitCode::FAILURE;
    };

    let root = repo_root();
    let script = root.join("scripts/oracle/talib_golden.py");
    let mut command = Command::new(python());
    command.current_dir(&root).arg(&script).arg(name);
    command.args(&args[1..]);

    match command.status() {
        Ok(status) if status.success() => ExitCode::SUCCESS,
        Ok(status) => {
            eprintln!("xtask golden: the oracle exited with {status}");
            ExitCode::FAILURE
        }
        Err(error) => {
            eprintln!(
                "xtask golden: could not run {}: {error}",
                python().display()
            );
            ExitCode::FAILURE
        }
    }
}

/// Write every generated file, or, with `check`, report the ones that drifted.
fn run_generate(check: bool) -> ExitCode {
    let root = repo_root();
    let generated = match generate::generate(&root) {
        Ok(generated) => generated,
        Err(errors) => {
            for error in &errors {
                eprintln!("{error}");
            }
            eprintln!("\n{} spec error(s); nothing was written", errors.len());
            return ExitCode::FAILURE;
        }
    };

    if check {
        let stale: Vec<&generate::Generated> = generated
            .iter()
            .filter(|item| std::fs::read_to_string(&item.path).unwrap_or_default() != item.text)
            .collect();
        if stale.is_empty() {
            println!("{} generated file(s) are up to date", generated.len());
            return ExitCode::SUCCESS;
        }
        for item in &stale {
            eprintln!(
                "out of date: {}",
                item.path
                    .strip_prefix(&root)
                    .unwrap_or(&item.path)
                    .display()
            );
        }
        eprintln!("\nrun `cargo xtask generate` and commit the result");
        return ExitCode::FAILURE;
    }

    match generate::write_all(&generated) {
        Ok(written) if written.is_empty() => {
            println!("{} generated file(s) already up to date", generated.len());
            ExitCode::SUCCESS
        }
        Ok(written) => {
            for path in &written {
                println!(
                    "wrote {}",
                    path.strip_prefix(&root).unwrap_or(path).display()
                );
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("xtask generate: {error}");
            ExitCode::FAILURE
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(command) = args.first() else {
        eprint!("{USAGE}");
        return ExitCode::FAILURE;
    };

    if matches!(command.as_str(), "-h" | "--help" | "help") {
        print!("{USAGE}");
        return ExitCode::SUCCESS;
    }

    if command == "golden" {
        return golden(&args[1..]);
    }

    if command == "generate" {
        return run_generate(false);
    }

    if command == "regen-check" {
        return run_generate(true);
    }

    match PENDING.iter().find(|(name, _)| name == command) {
        Some((name, milestone)) => {
            eprintln!("xtask {name}: not implemented until {milestone}");
            ExitCode::FAILURE
        }
        None => {
            eprintln!("xtask: unknown command `{command}`\n");
            eprint!("{USAGE}");
            ExitCode::FAILURE
        }
    }
}
