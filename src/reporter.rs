use crate::cleaner::CleanupReport;
use crate::incident_detector::{Incident, Severity};
use crate::model_scanner::ScanReport;
use crate::policy_manager::PolicyReport;
use serde_json::json;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputMode {
    Human,
    Json,
}

#[derive(Clone, Debug)]
pub struct CommandReport {
    pub action: String,
    pub scan: ScanReport,
    pub cleanup: Option<CleanupReport>,
    pub policy: Option<PolicyReport>,
    pub environment_incidents: Vec<Incident>,
}

pub struct Reporter;

impl Reporter {
    pub fn render(report: &CommandReport, mode: OutputMode) -> String {
        match mode {
            OutputMode::Human => render_human(report),
            OutputMode::Json => render_json(report),
        }
    }
}

fn render_human(report: &CommandReport) -> String {
    let mut output = String::new();
    output.push_str(&format!("Action: {}\n", report.action));
    output.push_str(&format!(
        "Model directories found: {}\n",
        report.scan.findings.len()
    ));
    let total_size: u64 = report
        .scan
        .findings
        .iter()
        .map(|finding| finding.size_bytes)
        .sum();
    output.push_str(&format!("Detected AI model bytes: {total_size}\n"));
    for finding in &report.scan.findings {
        output.push_str(&format!(
            "- {} ({} bytes): {}\n",
            finding.path.display(),
            finding.size_bytes,
            finding.reason
        ));
    }
    if let Some(cleanup) = &report.cleanup {
        output.push_str(&format!(
            "Deleted directories: {}\n",
            cleanup.deleted_paths.len()
        ));
        output.push_str(&format!("Bytes freed: {}\n", cleanup.bytes_freed));
    }
    if let Some(policy) = &report.policy {
        output.push_str(&format!("Policy file: {}\n", policy.path.display()));
        output.push_str(&format!("Policy written: {}\n", policy.written));
    }

    let incidents = all_incidents(report);
    if !incidents.is_empty() {
        output.push_str("Incidents:\n");
        for incident in incidents {
            output.push_str(&format!(
                "- {:?} [{}] {}: {}\n",
                incident.severity,
                incident.code,
                incident.path.display(),
                incident.message
            ));
        }
    }
    output
}

fn render_json(report: &CommandReport) -> String {
    let findings: Vec<_> = report
        .scan
        .findings
        .iter()
        .map(|finding| {
            json!({
                "path": finding.path,
                "kind": format!("{:?}", finding.kind),
                "size_bytes": finding.size_bytes,
                "confidence": finding.confidence,
                "reason": finding.reason
            })
        })
        .collect();
    let incidents: Vec<_> = all_incidents(report)
        .iter()
        .map(|incident| {
            json!({
                "code": incident.code,
                "severity": format!("{:?}", incident.severity),
                "path": incident.path,
                "message": incident.message
            })
        })
        .collect();

    serde_json::to_string_pretty(&json!({
        "action": report.action,
        "findings": findings,
        "cleanup": report.cleanup.as_ref().map(|cleanup| json!({
            "deleted_paths": cleanup.deleted_paths,
            "bytes_freed": cleanup.bytes_freed
        })),
        "policy": report.policy.as_ref().map(|policy| json!({
            "path": policy.path,
            "written": policy.written,
            "desired": policy.desired
        })),
        "incidents": incidents,
        "has_blocking_incidents": incidents.iter().any(|incident| {
            incident
                .get("severity")
                .and_then(|value| value.as_str())
                == Some("Blocking")
        })
    }))
    .expect("report serialization should not fail")
}

fn all_incidents(report: &CommandReport) -> Vec<Incident> {
    let mut incidents = Vec::new();
    incidents.extend(report.environment_incidents.clone());
    incidents.extend(report.scan.incidents.clone());
    if let Some(cleanup) = &report.cleanup {
        incidents.extend(cleanup.incidents.clone());
    }
    if let Some(policy) = &report.policy {
        incidents.extend(policy.incidents.clone());
    }
    incidents
}

pub fn has_blocking_incident(report: &CommandReport) -> bool {
    all_incidents(report)
        .iter()
        .any(|incident| incident.severity == Severity::Blocking)
}
