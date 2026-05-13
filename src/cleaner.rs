use crate::error::{AppError, AppResult};
use crate::incident_detector::{Incident, Severity};
use crate::model_scanner::ModelFinding;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CleanupMode {
    Apply,
    DryRun,
}

#[derive(Clone, Debug, Default)]
pub struct CleanupReport {
    pub deleted_paths: Vec<PathBuf>,
    pub bytes_freed: u64,
    pub incidents: Vec<Incident>,
}

pub struct Cleaner {
    chrome_root: PathBuf,
}

impl Cleaner {
    pub fn new(chrome_root: PathBuf) -> Self {
        Self { chrome_root }
    }

    pub fn clean(&self, findings: &[ModelFinding], mode: CleanupMode) -> AppResult<CleanupReport> {
        let mut report = CleanupReport::default();
        for finding in findings {
            if !is_direct_child_of(&finding.path, &self.chrome_root) || finding.confidence < 100 {
                report.incidents.push(Incident::new(
                    "unsafe_cleanup_candidate",
                    Severity::Blocking,
                    finding.path.clone(),
                    "Cleanup candidate is outside the Chrome root or below confidence threshold.",
                ));
                continue;
            }
            if !finding.path.exists() {
                continue;
            }
            let metadata = fs::symlink_metadata(&finding.path)
                .map_err(|error| AppError::io("inspect cleanup candidate", &finding.path, error))?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                report.incidents.push(Incident::new(
                    "unsafe_cleanup_candidate",
                    Severity::Blocking,
                    finding.path.clone(),
                    "Cleanup candidate is not a real directory.",
                ));
                continue;
            }
            report.bytes_freed = report.bytes_freed.saturating_add(finding.size_bytes);
            report.deleted_paths.push(finding.path.clone());
            if mode == CleanupMode::Apply {
                fs::remove_dir_all(&finding.path).map_err(|error| {
                    AppError::io("delete Chrome AI model directory", &finding.path, error)
                })?;
            }
        }
        Ok(report)
    }
}

fn is_direct_child_of(path: &Path, parent: &Path) -> bool {
    path.parent() == Some(parent)
}
