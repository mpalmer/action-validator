use std::{fs::{self, File}, ffi::{OsString, OsStr}};
use std::io::prelude::*;

use rstest::rstest;
use assert_cmd::Command;


#[derive(Debug)]
struct SnapshotTest {
    test_dir: String,
    exitcode: i32,
    stderr: String,
    stdout: String,
    test_files: Vec<OsString>,
}

impl SnapshotTest {
    fn new(test_dir: String) -> Self {
        let stderr = fs::read_to_string(
            format!("./tests/{test_dir}/stderr"),
        ).unwrap_or(String::from(""));

        let stdout = fs::read_to_string(
            format!("./tests/{test_dir}/stdout"),
        ).unwrap_or(String::from(""));

        let exitcode: i32 = fs::read_to_string(
            format!("./tests/{test_dir}/exitcode"),
        ).map(|s| {
            s
            .strip_suffix("\n")
            .unwrap_or(s.as_str())
            .parse::<i32>()
            .unwrap_or(0)
        }).unwrap_or(0);

        let test_files = Self::_get_files(&test_dir);

        SnapshotTest {
            test_dir,
            exitcode,
            stderr,
            stdout,
            test_files,
        }
    }

    #[cfg(not(feature = "save-snapshots"))]
    fn execute(self) {
        Command::cargo_bin(
            env!("CARGO_PKG_NAME"),
        )
            .expect("binary to execute")
            .args(self.test_files)
            .assert()
            .stdout(self.stdout)
            .stderr(self.stderr)
            .code(self.exitcode);
    }

    #[cfg(feature = "save-snapshots")]
    fn execute(&self) {
        let test_dir = self.test_dir.to_owned();
        let test_files = self.test_files.to_owned();
        let result = Command::cargo_bin(
            env!("CARGO_PKG_NAME"),
        )
            .expect("binary to execute")
            .args(test_files).ok().unwrap_or_else(|e| e.as_output().unwrap().to_owned());

        if !result.stdout.is_empty() {
            self._save_contents(
                format!("./tests/{test_dir}/stdout"),
                result.stdout,
            );
        }
        if !result.stderr.is_empty() {
            self._save_contents(
                format!("./tests/{test_dir}/stderr"),
                result.stderr,
            );
        }
        if let Some(exitcode) = result.status.code() {
            if exitcode > 0 {
                self._save_contents(
                    format!("./tests/{test_dir}/exitcode"),
                    format!("{exitcode}").into(),
                );
            }
        }
    }

    fn _save_contents(
        &self,
        file_name: String,
        contents: Vec<u8>,
    ) {
        println!("Saving {}", file_name);
        let res = File::create(file_name).unwrap().write_all(
            &contents,
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    fn _get_files(test_dir: &String) -> Vec<OsString> {
        let yml = Some(OsStr::new("yml"));
        fs::read_dir(
            format!("./tests/{test_dir}"),
        )
        .unwrap()
        .filter_map(Result::ok)
        .filter(|f| f.path().extension() == yml)
        .map(|f| f.path().into_os_string())
        .collect::<Vec<OsString>>()
    }
}


#[cfg(not(feature = "remote-checks"))]
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
    SnapshotTest::new(dir_name).execute();
}

#[cfg(feature = "remote-checks")]
#[rstest]
#[case("010_remote_checks_ok")]
#[case("011_remote_checks_failure")]
fn snapshot_remote_checks(#[case] dir_name: String) {
    SnapshotTest::new(dir_name).execute();
}
