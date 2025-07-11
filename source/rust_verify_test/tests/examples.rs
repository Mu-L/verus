#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

use rust_verify_test_macros::examples_in_dir;
use std::path::{Path, PathBuf};

#[derive(Debug)]
enum Mode {
    ExpectSuccess,
    ExpectErrors,
    ExpectFailures,
    ExpectWarnings,
}

examples_in_dir!("../../examples");
examples_in_dir!("../../examples/guide");
examples_in_dir!("../../examples/pcm");
examples_in_dir!("../../examples/state_machines");
examples_in_dir!("../../examples/summer_school");
examples_in_dir!("../../examples/state_machines/tutorial");
examples_in_dir!("../../examples/state_machines/reference-examples");
examples_in_dir!("../../examples/std_test");

#[cfg(feature = "singular")]
examples_in_dir!("../../examples/integer_ring");

fn run_example_for_file(file_path: &str) {
    let relative_path = Path::new(file_path);

    let mut path = std::path::PathBuf::from("rust_verify");
    path.extend(relative_path);
    let path = path.to_str().expect("invalid example path");

    let mut reader =
        std::io::BufReader::new(std::fs::File::open(relative_path).expect("cannot open file"));
    let mut first_line = String::new();
    let first_line_elements: Vec<_> = {
        use std::io::BufRead;
        reader.read_line(&mut first_line).expect("unable to read first line");
        first_line.trim().split(" ").collect()
    };

    let mut mode = Mode::ExpectSuccess;

    let mut options = vec![];

    if let ["//", "rust_verify/tests/example.rs", command, ..] = &first_line_elements[..] {
        match *command {
            "expect-success" => mode = Mode::ExpectSuccess,
            "expect-errors" => mode = Mode::ExpectErrors,
            "expect-failures" => mode = Mode::ExpectFailures,
            "expect-warnings" => mode = Mode::ExpectWarnings,
            "expand-errors" => {
                mode = Mode::ExpectFailures;
                options.push("--expand-errors");
            }
            "ignore" => {
                if first_line_elements.len() > 3 {
                    // We require that any comment is separated by a `---` which acts as a good
                    // visual separator.
                    if first_line_elements[3] != "---" {
                        panic!(
                            "Expected '---' to separate the extra comment from the 'ignore' declaration. Found {:?}",
                            first_line_elements[3],
                        );
                    } else if first_line_elements.len() == 4 {
                        panic!(
                            "Expected comment after visual separator '---' but no comment found."
                        );
                    }
                } else {
                    panic!(
                        "{}",
                        "Expected '--- {reason}' after the 'ignore', but none was provided."
                    );
                }
                return;
            }
            _ => panic!(
                "invalid command {:?} for example file test: use one of 'expect-success', 'expect-errors', 'expect-failures', or 'ignore'",
                command
            ),
        }
    }

    let relative_path = PathBuf::from(relative_path);
    let output = run_verus(
        &options,
        relative_path.parent().expect("no parent dir"),
        &relative_path,
        true,
        true,
    );

    use regex::Regex;
    let re = Regex::new(r"verification results:: (\d+) verified, (\d+) errors").unwrap();
    let stdout = std::str::from_utf8(&output.stdout).expect("invalid stdout encoding");
    let stderr = std::str::from_utf8(&output.stderr).expect("invalid stderr encoding").trim();
    let mut errors = Vec::new();
    let mut expand_errors_notes = Vec::new();
    let mut is_failure = false;
    let (warnings, _notes) =
        parse_diags(stderr, &mut errors, &mut expand_errors_notes, &mut is_failure);
    let verifier_output: Option<(u64, u64)> = re.captures_iter(stdout).next().map(|x| {
        (
            x[1].parse().expect("invalid verifier output"),
            x[2].parse().expect("invalid verifier output"),
        )
    });

    let success = match mode {
        Mode::ExpectSuccess => {
            output.status.success()
                && match verifier_output {
                    Some((_, 0)) => true,
                    _ => false,
                }
                && !is_failure
                && warnings.len() == 0
        }
        Mode::ExpectErrors => !output.status.success(),
        Mode::ExpectWarnings => {
            output.status.success()
                && match verifier_output {
                    Some((_, 0)) => true,
                    _ => false,
                }
                && warnings.len() > 0
        }
        Mode::ExpectFailures => {
            !output.status.success()
                && match verifier_output {
                    Some((_, failures)) if failures > 0 => true,
                    _ => false,
                }
        }
    };

    if !success {
        eprintln!("- example {} - mode: {:?} - failed -", &path, mode);
        if warnings.len() > 0 {
            for w in warnings {
                eprintln!("- warning - {}", w.rendered);
            }
        }
        eprintln!("- stdout -\n{}\n", stdout);
        assert!(false);
    }
}
