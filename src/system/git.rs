#[cfg(feature = "js")]
mod platform {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "/src/js/system.js")]
    extern "C" {
        #[wasm_bindgen(catch, js_namespace = git, js_name = lsFiles)]
        fn ls_files_js() -> Result<js_sys::Array, js_sys::Error>;
    }

    pub(super) fn ls_files() -> Result<Vec<String>, std::io::Error> {
        let files = ls_files_js().map_err(|e| std::io::Error::other(format!("{e:?}")))?;

        Ok(files.iter().filter_map(|value| value.as_string()).collect())
    }
}

#[cfg(not(feature = "js"))]
mod platform {
    pub(super) fn ls_files() -> Result<Vec<String>, std::io::Error> {
        use std::process::Command;

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
}

pub fn ls_files() -> Result<Vec<String>, std::io::Error> {
    platform::ls_files()
}
