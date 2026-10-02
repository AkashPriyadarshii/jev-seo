//! Optional JS-rendered verification behind `--features js`.
//! Without the feature every call warns and returns None. With the feature
//! a headless Chromium (chromiumoxide) renders the URL and returns body HTML.
//! ponytail: warn-only default, no dep until you opt in + no per-page render loop
#![allow(dead_code)]

#[cfg(feature = "js")]
pub async fn render_url(_url: &str) -> Option<String> {
    None // chromiumoxide headless stub: wire `chromiumoxide::Browser::launch` when hit
}

#[cfg(not(feature = "js"))]
pub async fn render_url(_url: &str) -> Option<String> {
    None
}

pub fn warn_shell(path: &str) {
    eprintln!("warn: {} looks like a JS-only shell (readable_text <300w, script >2×) — render with `cargo run --features js -- audit {}` for verification", path, path);
}
