use std::path::{Path, PathBuf};
use std::{ffi::OsStr, fs};

use assert_cmd::Command;
use fixtures::fixtures;

#[derive(Debug)]
struct SnapshotTest {
    test_dir: PathBuf,
    workflow_files: Vec<PathBuf>,
}

impl SnapshotTest {
    fn new(test_dir: &Path) -> Self {
        let workflow_files = fs::read_dir(test_dir)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|f| f.path().extension() == Some(OsStr::new("yml")))
            .map(|f| f.path().to_path_buf())
            .collect();

        SnapshotTest {
            test_dir: test_dir.to_path_buf(),
            workflow_files,
        }
    }

    #[cfg(not(feature = "test-save-snapshots"))]
    fn execute(self) {
        let stderr = fs::read_to_string(self.test_dir.join("stderr")).unwrap_or(String::from(""));

        let stdout = fs::read_to_string(self.test_dir.join("stdout")).unwrap_or(String::from(""));

        let exitcode: i32 = fs::read_to_string(self.test_dir.join("exitcode"))
            .map(|s| {
                s.strip_suffix("\n")
                    .unwrap_or(s.as_str())
                    .parse::<i32>()
                    .unwrap_or(0)
            })
            .unwrap_or(0);

        #[cfg(not(feature = "test-js"))]
        Command::cargo_bin(env!("CARGO_PKG_NAME"))
            .expect("binary to execute")
            .args(self.workflow_files)
            .assert()
            .stdout(stdout)
            .stderr(stderr)
            .code(exitcode);

        #[cfg(feature = "test-js")]
        Command::new("npx")
            .arg("action-validator")
            .args(self.workflow_files)
            .assert()
            .stdout(stdout)
            .stderr(stderr)
            .code(exitcode);
    }

    #[cfg(feature = "test-save-snapshots")]
    fn execute(&self) {
        use std::fs::File;
        use std::io::prelude::*;

        let result = Command::cargo_bin(env!("CARGO_PKG_NAME"))
            .expect("binary to execute")
            .args(&self.workflow_files)
            .ok()
            .unwrap_or_else(|e| e.as_output().unwrap().to_owned());

        if !result.stdout.is_empty() {
            File::create(self.test_dir.join("stdout"))
                .unwrap()
                .write_all(&result.stdout)
                .unwrap();
        }
        if !result.stderr.is_empty() {
            File::create(self.test_dir.join("stderr"))
                .unwrap()
                .write_all(&result.stderr)
                .unwrap();
        }
        if let Some(exitcode) = result.status.code() {
            if exitcode > 0 {
                File::create(self.test_dir.join("exitcode"))
                    .unwrap()
                    .write_all(exitcode.to_string().as_bytes())
                    .unwrap();
            }
        }
    }
}

#[fixtures("tests/fixtures/*")]
fn snapshot(dir: &Path) {
    SnapshotTest::new(dir).execute();
}
