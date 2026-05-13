use remove_chrome_ai::cleaner::{Cleaner, CleanupMode};
use remove_chrome_ai::model_scanner::{FindingKind, ModelScanner};
use std::fs;
use std::path::Path;

fn write_file(path: &Path, bytes: &[u8]) {
    fs::create_dir_all(path.parent().expect("test path has parent")).unwrap();
    fs::write(path, bytes).unwrap();
}

#[test]
fn scanner_finds_known_model_directories_and_sizes_them() {
    let temp = tempfile::tempdir().unwrap();
    let chrome_root = temp.path().join("google-chrome");
    write_file(
        &chrome_root.join("OptGuideOnDeviceModel/2025.8.21.1028/weights.bin"),
        &[1, 2, 3, 4],
    );
    write_file(
        &chrome_root.join("OptGuideOnDeviceClassifierModel/1.0/ts.bin"),
        &[5, 6],
    );
    write_file(&chrome_root.join("Default/History"), &[7, 8, 9]);

    let report = ModelScanner::new(chrome_root.clone()).scan().unwrap();

    assert_eq!(report.findings.len(), 2);
    assert!(report.findings.iter().any(|finding| {
        finding.kind == FindingKind::BaseModel
            && finding.path == chrome_root.join("OptGuideOnDeviceModel")
            && finding.size_bytes == 4
    }));
    assert!(report.findings.iter().any(|finding| {
        finding.kind == FindingKind::ClassifierModel
            && finding.path == chrome_root.join("OptGuideOnDeviceClassifierModel")
            && finding.size_bytes == 2
    }));
}

#[test]
fn scanner_refuses_symlinked_candidates() {
    let temp = tempfile::tempdir().unwrap();
    let chrome_root = temp.path().join("google-chrome");
    let external = temp.path().join("external");
    fs::create_dir_all(&chrome_root).unwrap();
    fs::create_dir_all(&external).unwrap();

    #[cfg(unix)]
    std::os::unix::fs::symlink(&external, chrome_root.join("OptGuideOnDeviceModel")).unwrap();

    let report = ModelScanner::new(chrome_root).scan().unwrap();

    assert!(report.findings.is_empty());
    assert!(report.incidents.iter().any(|incident| {
        incident.code == "symlink_candidate" && incident.path.ends_with("OptGuideOnDeviceModel")
    }));
}

#[test]
fn cleaner_deletes_only_high_confidence_candidates_and_is_idempotent() {
    let temp = tempfile::tempdir().unwrap();
    let chrome_root = temp.path().join("google-chrome");
    write_file(
        &chrome_root.join("OptGuideOnDeviceModel/2025.8.21.1028/weights.bin"),
        &[1, 2, 3, 4],
    );
    write_file(&chrome_root.join("Default/History"), &[7, 8, 9]);

    let first_report = ModelScanner::new(chrome_root.clone()).scan().unwrap();
    let first_cleanup = Cleaner::new(chrome_root.clone())
        .clean(&first_report.findings, CleanupMode::Apply)
        .unwrap();
    let second_report = ModelScanner::new(chrome_root.clone()).scan().unwrap();
    let second_cleanup = Cleaner::new(chrome_root.clone())
        .clean(&second_report.findings, CleanupMode::Apply)
        .unwrap();

    assert_eq!(first_cleanup.deleted_paths.len(), 1);
    assert_eq!(first_cleanup.bytes_freed, 4);
    assert_eq!(second_cleanup.deleted_paths.len(), 0);
    assert!(chrome_root.join("Default/History").exists());
    assert!(!chrome_root.join("OptGuideOnDeviceModel").exists());
}
