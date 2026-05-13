use crate::error::{AppError, AppResult};
use crate::incident_detector::{Incident, Severity};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FindingKind {
    BaseModel,
    ClassifierModel,
    ModelsManifest,
    ManifestModel,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelFinding {
    pub path: PathBuf,
    pub kind: FindingKind,
    pub size_bytes: u64,
    pub confidence: u8,
    pub reason: String,
}

#[derive(Clone, Debug, Default)]
pub struct ScanReport {
    pub findings: Vec<ModelFinding>,
    pub incidents: Vec<Incident>,
}

pub struct ModelScanner {
    chrome_root: PathBuf,
}

impl ModelScanner {
    pub fn new(chrome_root: PathBuf) -> Self {
        Self { chrome_root }
    }

    pub fn scan(&self) -> AppResult<ScanReport> {
        let mut report = ScanReport::default();
        if !self.chrome_root.exists() {
            return Ok(report);
        }
        if !self.chrome_root.is_dir() {
            return Err(AppError::Config(format!(
                "Chrome root '{}' is not a directory",
                self.chrome_root.display()
            )));
        }

        for candidate in known_candidates() {
            let path = self.chrome_root.join(candidate.directory);
            if !path.exists() && !path.is_symlink() {
                continue;
            }
            let metadata = fs::symlink_metadata(&path)
                .map_err(|error| AppError::io("inspect model candidate", &path, error))?;
            if metadata.file_type().is_symlink() {
                report.incidents.push(Incident::new(
                    "symlink_candidate",
                    Severity::Blocking,
                    path,
                    "Candidate is a symlink and will not be followed or removed.",
                ));
                continue;
            }
            if !metadata.is_dir() {
                report.incidents.push(Incident::new(
                    "non_directory_candidate",
                    Severity::Warning,
                    path,
                    "Candidate exists but is not a directory.",
                ));
                continue;
            }
            let size_bytes = directory_size(&path)?;
            report.findings.push(ModelFinding {
                path,
                kind: candidate.kind,
                size_bytes,
                confidence: 100,
                reason: format!(
                    "Exact Chrome on-device AI component directory '{}'.",
                    candidate.directory
                ),
            });
        }

        Ok(report)
    }
}

#[derive(Clone, Copy)]
struct Candidate {
    directory: &'static str,
    kind: FindingKind,
}

fn known_candidates() -> [Candidate; 4] {
    [
        Candidate {
            directory: "OptGuideOnDeviceModel",
            kind: FindingKind::BaseModel,
        },
        Candidate {
            directory: "OptGuideOnDeviceClassifierModel",
            kind: FindingKind::ClassifierModel,
        },
        Candidate {
            directory: "OptimizationGuideModelsManifest",
            kind: FindingKind::ModelsManifest,
        },
        Candidate {
            directory: "OptGuideManifestModel",
            kind: FindingKind::ManifestModel,
        },
    ]
}

fn directory_size(path: &Path) -> AppResult<u64> {
    let mut total = 0_u64;
    for entry in fs::read_dir(path).map_err(|error| AppError::io("read directory", path, error))? {
        let entry = entry.map_err(|error| AppError::io("read directory entry", path, error))?;
        let entry_path = entry.path();
        let metadata = fs::symlink_metadata(&entry_path)
            .map_err(|error| AppError::io("inspect directory entry", &entry_path, error))?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            total = total.saturating_add(directory_size(&entry_path)?);
        } else if metadata.is_file() {
            total = total.saturating_add(metadata.len());
        }
    }
    Ok(total)
}
