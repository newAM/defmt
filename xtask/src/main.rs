use std::{process::Command, str::FromStr, sync::Mutex};

use anyhow::{anyhow, Context};
use clap::{Parser, Subcommand};
use colored::Colorize;
use once_cell::sync::Lazy;
use similar::{ChangeTag, TextDiff};

mod backcompat;
mod targets;
mod utils;

use crate::utils::{
    load_expected_output, overwrite_expected_output, run_capturing_stdout, run_command, rustc_is_nightly,
};

static ALL_ERRORS: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(vec![]));

const SNAPSHOT_TESTS_DIRECTORY: &str = "firmware/qemu";
const ALL_SNAPSHOT_TESTS: [&str; 12] = [
    "log",
    "bitflags",
    "timestamp",
    "panic",
    "assert",
    "assert-eq",
    "assert-ne",
    "unwrap",
    "defmt-test",
    "hints",
    "hints_inner",
    "dbg",
];

#[derive(Clone, Debug)]
struct Snapshot(String);

impl Snapshot {
    pub fn name(&self) -> &str {
        &self.0
    }
}

impl FromStr for Snapshot {
    type Err = String;

    fn from_str(test: &str) -> Result<Self, Self::Err> {
        if ALL_SNAPSHOT_TESTS.contains(&test) {
            Ok(Self(String::from(test)))
        } else {
            Err(format!(
                "Specified test '{}' does not exist, available tests are: {:?}",
                test, ALL_SNAPSHOT_TESTS
            ))
        }
    }
}

#[derive(Debug, Parser)]
struct Options {
    #[command(subcommand)]
    cmd: TestCommand,

    /// Treat compiler warnings as errors (`RUSTFLAGS="--deny warnings"`)
    #[arg(long, short)]
    deny_warnings: bool,

    /// Keep target toolchains that were installed as dependency
    #[arg(long, short)]
    keep_targets: bool,
}

#[derive(Debug, Subcommand)]
#[allow(clippy::enum_variant_names)]
enum TestCommand {
    TestAll,
    TestBackcompat,
    TestBook,
    TestCross,
    TestHost,
    TestLint,
    TestUi,
    /// Run snapshot tests or optionally overwrite the expected output
    TestSnapshot {
        /// Overwrite the expected output instead of comparing it.
        #[arg(long)]
        overwrite: bool,
        /// Runs a single snapshot test in Debug mode
        single: Option<Snapshot>,
    },
}

fn main() -> anyhow::Result<()> {
    let opt = Options::parse();
    let mut added_targets = None;

    match opt.cmd {
        TestCommand::TestBook => test_book(),
        TestCommand::TestBackcompat => backcompat::test(),
        TestCommand::TestHost => test_host(opt.deny_warnings),
        TestCommand::TestLint => test_lint(),
        TestCommand::TestUi => test_ui(),

        // following tests need to install additional targets
        cmd => {
            added_targets = Some(targets::install().expect("Error while installing required targets"));
            match cmd {
                TestCommand::TestCross => test_cross(),
                TestCommand::TestSnapshot { overwrite, single } => {
                    test_snapshot(overwrite, single);
                }
                TestCommand::TestAll => {
                    test_host(opt.deny_warnings);
                    test_cross();
                    test_snapshot(false, None);
                    backcompat::test();
                    test_book();
                    test_lint();
                }
                _ => unreachable!("get handled in outer `match`"),
            }
        }
    }

    if let Some(added_targets) = added_targets {
        if !opt.keep_targets && !added_targets.is_empty() {
            targets::uninstall(added_targets)
        }
    }

    let all_errors = ALL_ERRORS.lock().unwrap();
    if !all_errors.is_empty() {
        eprintln!();
        Err(anyhow!("😔 some tests failed: {:#?}", all_errors))
    } else {
        Ok(())
    }
}

fn do_test(test: impl FnOnce() -> anyhow::Result<()>, context: &str) {
    test().unwrap_or_else(|e| ALL_ERRORS.lock().unwrap().push(format!("{context}: {e}")));
}

fn test_host(deny_warnings: bool) {
    println!("🧪 host");

    let env = if deny_warnings {
        vec![("RUSTFLAGS", "--deny warnings")]
    } else {
        vec![]
    };

    do_test(
        || run_command("cargo", &["check", "--workspace"], None, &env),
        "host",
    );

    do_test(
        || {
            run_command(
                "cargo",
                &["check", "--workspace", "--features", "unstable-test"],
                None,
                &env,
            )
        },
        "host",
    );

    do_test(
        || {
            run_command(
                "cargo",
                &["check", "--workspace", "--features", "alloc"],
                None,
                &env,
            )
        },
        "host",
    );

    do_test(
        || {
            run_command(
                "cargo",
                &["test", "--workspace", "--features", "unstable-test"],
                None,
                &[],
            )
        },
        "host",
    );

    do_test(
        || {
            run_command(
                "cargo",
                &["test", "--workspace", "--features", "unstable-test,alloc"],
                None,
                &[],
            )
        },
        "host",
    );
}

fn test_cross() {
    println!("🧪 cross");
    let targets = [
        "thumbv6m-none-eabi",
        "thumbv8m.base-none-eabi",
        "riscv32i-unknown-none-elf",
    ];

    for target in &targets {
        do_test(
            || run_command("cargo", &["check", "--target", target, "-p", "defmt"], None, &[]),
            "cross",
        );
        do_test(
            || {
                run_command(
                    "cargo",
                    &["check", "--target", target, "-p", "defmt", "--features", "alloc"],
                    None,
                    &[],
                )
            },
            "cross",
        );
    }

    do_test(
        || {
            run_command(
                "cargo",
                &[
                    "check",
                    "--target",
                    "thumbv6m-none-eabi",
                    "--workspace",
                    "--exclude",
                    "defmt-itm",
                    "--exclude",
                    "firmware",
                ],
                Some("firmware"),
                &[],
            )
        },
        "cross",
    );

    do_test(
        || {
            run_command(
                "cargo",
                &["check", "--target", "thumbv7em-none-eabi", "--workspace"],
                Some("firmware"),
                &[],
            )
        },
        "cross",
    );

    do_test(
        || {
            run_command(
                "cargo",
                &[
                    "check",
                    "--target",
                    "thumbv6m-none-eabi",
                    "--features",
                    "print-defmt",
                ],
                Some("firmware/panic-probe"),
                &[],
            )
        },
        "cross",
    );

    do_test(
        || {
            run_command(
                "cargo",
                &[
                    "check",
                    "--target",
                    "thumbv6m-none-eabi",
                    "--features",
                    "print-rtt",
                ],
                Some("firmware/panic-probe"),
                &[],
            )
        },
        "cross",
    )
}

fn test_snapshot(overwrite: bool, snapshot: Option<Snapshot>) {
    println!("🧪 qemu/snapshot");

    match snapshot {
        None => test_all_snapshots(overwrite),
        Some(snapshot) => {
            do_test(
                || test_single_snapshot(snapshot.name(), "", overwrite),
                "qemu/snapshot",
            );
        }
    }
}

fn test_all_snapshots(overwrite: bool) {
    let mut tests = ALL_SNAPSHOT_TESTS.to_vec();

    if rustc_is_nightly() {
        tests.push("alloc");
    }

    for test in tests {
        let features = if test == "alloc" { "alloc" } else { "" };

        do_test(
            || test_single_snapshot(test, features, overwrite),
            "qemu/snapshot",
        );
    }
}

fn test_single_snapshot(name: &str, features: &str, overwrite: bool) -> anyhow::Result<()> {
    println!("{}", name.bold());

    let is_test = name.contains("test");

    let mut args = if is_test {
        vec!["-q", "tt", name]
    } else {
        vec!["-q", "rb", name]
    };

    if !features.is_empty() {
        args.extend_from_slice(&["--features", features]);
    }

    let actual = run_capturing_stdout(
        Command::new("cargo")
            .args(&args)
            .env("DEFMT_LOG", "trace")
            .current_dir(SNAPSHOT_TESTS_DIRECTORY),
    )
    .with_context(|| name.to_string())?;

    if overwrite {
        overwrite_expected_output(name, actual.as_bytes(), is_test)?;
        return Ok(());
    }

    let expected = load_expected_output(name, is_test)?;
    let diff = TextDiff::from_lines(&expected, &actual);

    // if anything isn't ChangeTag::Equal, print it and turn on error flag
    let mut actual_matches_expected = true;
    for op in diff.ops() {
        for change in diff.iter_changes(op) {
            let styled_change = match change.tag() {
                ChangeTag::Delete => Some(("-".bold().red(), change.to_string().red())),
                ChangeTag::Insert => Some(("+".bold().green(), change.to_string().green())),
                ChangeTag::Equal => None,
            };
            if let Some((sign, change)) = styled_change {
                actual_matches_expected = false;
                eprint!("{sign}{change}");
            }
        }
    }

    if actual_matches_expected {
        Ok(())
    } else {
        Err(anyhow!("{}", name))
    }
}

fn test_book() {
    println!("🧪 book");
    do_test(|| run_command("cargo", &["clean"], None, &[]), "book");

    do_test(
        || {
            run_command(
                "cargo",
                &[
                    "build",
                    "-p",
                    "defmt",
                    "-p",
                    "defmt-decoder",
                    "--features",
                    "unstable-test",
                ],
                None,
                &[],
            )
        },
        "book",
    );

    do_test(
        || run_command("cargo", &["build", "-p", "cortex-m"], Some("firmware"), &[]),
        "book",
    );

    do_test(
        || {
            run_command(
                "mdbook",
                &[
                    "test",
                    "-L",
                    "../target/debug",
                    "-L",
                    "../target/debug/deps",
                    "-L",
                    "../firmware/target/debug",
                    "-L",
                    "../firmware/target/debug/deps",
                ],
                Some("book"),
                // logging macros need this but mdbook, not being Cargo, doesn't set the env var so
                // we use a dummy value
                &[("CARGO_CRATE_NAME", "krate")],
            )
        },
        "book",
    );
}

fn test_lint() {
    println!("🧪 lint");
    do_test(|| run_command("cargo", &["clean"], None, &[]), "lint");
    do_test(
        || run_command("cargo", &["fmt", "--all", "--", "--check"], None, &[]),
        "lint",
    );

    do_test(
        || run_command("cargo", &["clippy", "--workspace"], None, &[]),
        "lint",
    );
}

fn test_ui() {
    println!("🧪 lint");
    do_test(
        || run_command("cargo", &["test"], Some("firmware/defmt-test/macros"), &[]),
        "ui",
    );
}
