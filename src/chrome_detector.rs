use std::env;
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChromeInstall {
    pub binary: PathBuf,
}

pub struct ChromeDetector;

impl ChromeDetector {
    pub fn detect_stable() -> Option<ChromeInstall> {
        ["google-chrome", "google-chrome-stable"]
            .iter()
            .find_map(|name| find_in_path(name).map(|binary| ChromeInstall { binary }))
    }
}

fn find_in_path(binary_name: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    for directory in env::split_paths(&path) {
        let candidate = directory.join(binary_name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}
