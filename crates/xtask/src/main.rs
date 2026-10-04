#![forbid(unsafe_code)]

use std::process::ExitCode;

const USAGE: &str = "\
cargo xtask <command>

Commands:
  generate      Specs -> registry, bindings, wrappers, stubs, docs pages   (M2)
  regen-check   Run generate, then fail if the working tree is dirty       (M2)
  golden        Re-produce the golden CSVs of one indicator from its oracle (M1)
  bench         Criterion benchmarks and the Python vs TA-Lib comparison   (M4)
";

/// The milestone that is going to implement each command, per docs/MILESTONES.md.
const PENDING: &[(&str, &str)] = &[
    ("generate", "M2"),
    ("regen-check", "M2"),
    ("golden", "M1"),
    ("bench", "M4"),
];

fn main() -> ExitCode {
    let Some(command) = std::env::args().nth(1) else {
        eprint!("{USAGE}");
        return ExitCode::FAILURE;
    };

    if matches!(command.as_str(), "-h" | "--help" | "help") {
        print!("{USAGE}");
        return ExitCode::SUCCESS;
    }

    match PENDING.iter().find(|(name, _)| *name == command) {
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
