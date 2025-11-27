use std::process::Command;

pub fn ls_files() -> Result<Vec<String>, std::io::Error> {
    let output = Command::new("git").args(["ls-files", "-z"]).output()?;

    if !output.status.success() {
        return Err(std::io::Error::other(format!(
            "git ls-files failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let files = String::from_utf8_lossy(&output.stdout)
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    Ok(files)
}
