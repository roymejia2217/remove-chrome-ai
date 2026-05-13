use crate::error::{AppError, AppResult};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Severity {
    Info,
    Warning,
    Blocking,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Incident {
    pub code: String,
    pub severity: Severity,
    pub path: PathBuf,
    pub message: String,
}

impl Incident {
    pub fn new(
        code: impl Into<String>,
        severity: Severity,
        path: impl Into<PathBuf>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity,
            path: path.into(),
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct EnvironmentIncidents {
    pub incidents: Vec<Incident>,
}

pub struct IncidentDetector;

impl IncidentDetector {
    pub fn detect_chrome_processes() -> EnvironmentIncidents {
        let mut incidents = Vec::new();
        let proc_dir = Path::new("/proc");
        let Ok(entries) = fs::read_dir(proc_dir) else {
            return EnvironmentIncidents { incidents };
        };

        for entry in entries.flatten() {
            let file_name = entry.file_name();
            if !file_name
                .to_string_lossy()
                .chars()
                .all(|character| character.is_ascii_digit())
            {
                continue;
            }
            let comm_path = entry.path().join("comm");
            let Ok(name) = fs::read_to_string(&comm_path) else {
                continue;
            };
            let process_name = name.trim();
            if matches!(
                process_name,
                "chrome" | "google-chrome" | "google-chrome-stable"
            ) {
                incidents.push(Incident::new(
                    "chrome_running",
                    Severity::Warning,
                    comm_path,
                    "Google Chrome appears to be running; restart Chrome after policy changes.",
                ));
            }
        }

        EnvironmentIncidents { incidents }
    }

    pub fn ensure_directory_owned_or_missing(path: &Path) -> AppResult<()> {
        if !path.exists() {
            return Ok(());
        }
        let metadata = fs::symlink_metadata(path)
            .map_err(|error| AppError::io("inspect directory metadata", path, error))?;
        if !metadata.is_dir() {
            return Err(AppError::Config(format!(
                "expected '{}' to be a directory",
                path.display()
            )));
        }
        Ok(())
    }
}
