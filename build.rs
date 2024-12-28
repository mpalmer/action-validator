use std::{fs, io::Result, path::PathBuf};

use clap::{CommandFactory, ValueEnum};
use clap_complete::{generate_to, Shell};
use clap_mangen::generate_to as generate_manpage;

mod cli {
    include!("src/cli.rs");
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=src/cli.rs");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=GEN_DIR");

    if let Some(out_path) = std::env::var_os("GEN_DIR").or(std::env::var_os("OUT_DIR")) {
        let out_dir = PathBuf::from(out_path);

        // Generate manpage
        let man_dir = out_dir.join("man");
        fs::create_dir_all(&man_dir)?;
        let mut cmd = cli::CliConfig::command();
        generate_manpage(cmd.clone(), &man_dir)?;

        // Generate shell completions
        let completions_dir = out_dir.join("completions");
        fs::create_dir_all(&completions_dir)?;

        for shell in Shell::value_variants() {
            generate_to(*shell, &mut cmd, "action-validator", &completions_dir)?;
        }
    }

    println!(
        "cargo:rustc-env=TARGET={}",
        std::env::var("TARGET").unwrap()
    );

    Ok(())
}
