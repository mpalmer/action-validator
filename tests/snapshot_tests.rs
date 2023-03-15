use rstest::rstest;
use std::{fs, ffi::{OsString, OsStr}};


#[derive(Debug)]
struct SnapshotTest {
    exitcode: i32,
    stderr: String,
    stdout: String,
    test_files: Vec<OsString>,
}

impl SnapshotTest {
    fn new(test_name: String) -> Self {
        let stderr = fs::read_to_string(
            format!("./tests/{test_name}/stderr"),
        ).unwrap_or(String::from(""));

        let stdout = fs::read_to_string(
            format!("./tests/{test_name}/stdout"),
        ).unwrap_or(String::from(""));

        let exitcode: i32 = fs::read_to_string(
            format!("./tests/{test_name}/exitcode"),
        ).map(|s| {
            s
            .strip_suffix("\n")
            .unwrap_or("")
            .parse::<i32>()
            .unwrap_or(0)
        }).unwrap_or(0);

        let test_files = Self::_get_files(test_name);

        SnapshotTest {
            exitcode,
            stderr,
            stdout,
            test_files,
        }
    }

    fn _get_files(test_name: String) -> Vec<OsString> {
        let yml = Some(OsStr::new("yml"));
        fs::read_dir(
            format!("./tests/{test_name}"),
        )
        .unwrap()
        .filter_map(Result::ok)
        .filter(|f| f.path().extension() == yml)
        .map(|f| f.path().into_os_string())
        .collect::<Vec<OsString>>()
    }
}

use assert_cmd::Command;

#[rstest]
#[case("001_basic_workflow")]
#[case("002_basic_action")]
#[case("003_successful_globs")]
#[case("004_failing_globs")]
#[case("005_conditional_step_in_action")]
#[case("006_workflow_dispatch_inputs_options")]
#[case("007_funky_syntax")]
#[case("008_job_dependencies")]
#[case("009_multi_file")]
fn snapshot(#[case] dir_name: String) {

    let test = SnapshotTest::new(dir_name);
    Command::cargo_bin(
        env!("CARGO_PKG_NAME"),
    )
    .expect("binary to execute")
    .args(test.test_files)
    .assert()
    .stdout(test.stdout)
    .stderr(test.stderr)
    .code(test.exitcode);
}
