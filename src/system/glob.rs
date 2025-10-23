// The below functions have duplicate implementations for WASM and non-WASM targets.
// Each target might not use all of the functions, but they are all defined for both targets
// for simplicity.
#![allow(dead_code)]

#[cfg(feature = "js")]
mod js_fg {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "/src/js/system.js")]
    extern "C" {
        #[wasm_bindgen(js_name = String)]
        type Entry;

        #[wasm_bindgen(catch, js_namespace = fg, js_name = globSync)]
        pub fn glob_sync(pattern: &str) -> Result<Vec<js_sys::Object>, js_sys::Error>;
    }
}

#[cfg(feature = "js")]
pub fn glob_count_matches(pattern: &str) -> Result<usize, String> {
    js_fg::glob_sync(pattern)
        .map(|entries| entries.len())
        .map_err(|e| format!("{}", e.to_string()))
}

#[cfg(not(feature = "js"))]
pub fn glob_count_matches(pattern: &str) -> Result<usize, String> {
    use glob::glob;
    glob(pattern)
        .map(|paths| paths.count())
        .map_err(|e| e.to_string())
}
