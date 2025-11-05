use fixtures::fixtures;
use std::env::current_dir;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{ffi::OsStr, fs};

static REPO_DIR_WILDCARD: &str = "{{repo}}";

#[derive(Debug, serde::Deserialize)]
struct SnapshotTestConfig {
    cli_args: Option<Vec<String>>,
}

#[derive(Debug)]
struct SnapshotTest {
    config: SnapshotTestConfig,
    current_dir: PathBuf,
    test_dir: PathBuf,
}

impl SnapshotTest {
    fn new(test_dir: &Path) -> Self {
        let test_config_file = test_dir.join("test.json");

        let config: SnapshotTestConfig = serde_json::from_reader(BufReader::new(
            File::open(&test_config_file).expect(&format!(
                "missing test conifg file ({})",
                test_config_file.to_string_lossy(),
            )),
        ))
        .expect(&format!(
            "invalid test config file ({})",
            test_config_file.to_string_lossy(),
        ));

        SnapshotTest {
            config,
            current_dir: current_dir().unwrap(),
            test_dir: test_dir.to_path_buf(),
        }
    }

    fn build_command(&self) -> Command {
        #[cfg(not(feature = "test-js"))]
        {
            Command::new(assert_cmd::cargo::cargo_bin!())
        }

        #[cfg(feature = "test-js")]
        {
            let mut cmd = Command::new("npx");
            cmd.arg("@action-validator/cli");
            cmd
        }
    }

    fn execute(self) {
        use std::ffi::OsString;

        let pwd = self.current_dir.to_str().unwrap();

        let cli_args: Vec<_> = if let Some(cli_args) = &self.config.cli_args {
            cli_args.iter().map(OsString::from).collect()
        } else {
            fs::read_dir(&self.test_dir)
                .unwrap()
                .filter_map(Result::ok)
                .filter(|f| f.path().extension() == Some(OsStr::new("yml")))
                .map(|f| f.path().into_os_string())
                .collect()
        };

        #[cfg(not(feature = "test-save-snapshots"))]
        {
            use assert_cmd::assert::OutputAssertExt as _;

            let stderr = fs::read_to_string(self.test_dir.join("stderr"))
                .unwrap_or(String::from(""))
                .replace(REPO_DIR_WILDCARD, pwd);

            let stdout = fs::read_to_string(self.test_dir.join("stdout"))
                .unwrap_or(String::from(""))
                .replace(REPO_DIR_WILDCARD, pwd);

            let exitcode: i32 = fs::read_to_string(self.test_dir.join("exitcode"))
                .map(|s| {
                    s.strip_suffix("\n")
                        .unwrap_or(s.as_str())
                        .parse::<i32>()
                        .unwrap_or(0)
                })
                .unwrap_or(0);

            self.build_command()
                .args(&cli_args)
                .assert()
                .stdout(stdout)
                .stderr(stderr)
                .code(exitcode);
        }

        #[cfg(feature = "test-save-snapshots")]
        {
            use assert_cmd::output::OutputOkExt as _;
            use std::io::prelude::*;

            let result = self
                .build_command()
                .args(&cli_args)
                .ok()
                .unwrap_or_else(|e| e.as_output().unwrap().to_owned());

            if !result.stdout.is_empty() {
                File::create(self.test_dir.join("stdout"))
                    .unwrap()
                    .write_all(
                        String::from_utf8(result.stdout)
                            .unwrap()
                            .replace(pwd, REPO_DIR_WILDCARD)
                            .as_bytes(),
                    )
                    .unwrap();
            }
            if !result.stderr.is_empty() {
                File::create(self.test_dir.join("stderr"))
                    .unwrap()
                    .write_all(
                        String::from_utf8(result.stderr)
                            .unwrap()
                            .replace(pwd, REPO_DIR_WILDCARD)
                            .as_bytes(),
                    )
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
}

#[fixtures(["tests/fixtures/*"])]
#[cfg_attr(
    feature = "test-js",
    fixtures::ignore(
        paths = "tests/fixtures/013_rejects_gitignore_extended_glob_syntax",
        reason = "The WASM implementation of action validator currently (incorrectly) accepts extended gitignore syntax"
    )
)]
#[test]
fn snapshot(dir: &Path) {
    SnapshotTest::new(dir).execute();
}
