use crate::error::{AppError, AppResult};
use crate::incident_detector::{Incident, Severity};
use serde_json::{Map, Value, json};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyMode {
    Core,
    Strict,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyWriteMode {
    Apply,
    DryRun,
}

#[derive(Clone, Debug, Default)]
pub struct PolicyReport {
    pub path: PathBuf,
    pub written: bool,
    pub desired: Value,
    pub incidents: Vec<Incident>,
}

pub struct PolicyManager {
    policy_dir: PathBuf,
    file_name: String,
}

impl PolicyManager {
    pub fn new(policy_dir: PathBuf, file_name: String) -> Self {
        Self {
            policy_dir,
            file_name,
        }
    }

    pub fn desired_policy(mode: PolicyMode) -> Value {
        match mode {
            PolicyMode::Core => json!({
                "GenAILocalFoundationalModelSettings": 1
            }),
            PolicyMode::Strict => json!({
                "GenAILocalFoundationalModelSettings": 1,
                "GenAiDefaultSettings": 2,
                "DevToolsGenAiSettings": 2
            }),
        }
    }

    pub fn apply(&self, mode: PolicyMode, write_mode: PolicyWriteMode) -> AppResult<PolicyReport> {
        let desired = Self::desired_policy(mode);
        let target_path = self.policy_dir.join(&self.file_name);
        let mut report = PolicyReport {
            path: target_path.clone(),
            desired: desired.clone(),
            ..PolicyReport::default()
        };

        if self.policy_dir.exists() {
            for entry in fs::read_dir(&self.policy_dir)
                .map_err(|error| AppError::io("read policy directory", &self.policy_dir, error))?
            {
                let entry = entry.map_err(|error| {
                    AppError::io("read policy directory entry", &self.policy_dir, error)
                })?;
                let path = entry.path();
                if path.extension().and_then(|value| value.to_str()) != Some("json") {
                    continue;
                }
                let content = fs::read_to_string(&path)
                    .map_err(|error| AppError::io("read policy file", &path, error))?;
                let parsed: Value = match serde_json::from_str(&content) {
                    Ok(value) => value,
                    Err(error) => {
                        report.incidents.push(Incident::new(
                            "malformed_policy_json",
                            Severity::Blocking,
                            path.clone(),
                            format!("Policy JSON is malformed: {error}"),
                        ));
                        continue;
                    }
                };
                if path.file_name().and_then(|value| value.to_str()) == Some(&self.file_name) {
                    continue;
                }
                collect_conflicts(&desired, &parsed, &path, &mut report.incidents);
            }
        }

        if report
            .incidents
            .iter()
            .any(|incident| incident.severity == Severity::Blocking)
        {
            return Ok(report);
        }

        if write_mode == PolicyWriteMode::DryRun {
            return Ok(report);
        }

        fs::create_dir_all(&self.policy_dir)
            .map_err(|error| AppError::io("create policy directory", &self.policy_dir, error))?;
        let temporary_path = self.policy_dir.join(format!("{}.tmp", self.file_name));
        let serialized = serde_json::to_string_pretty(&desired)
            .map_err(|error| AppError::json("serialize desired policy", &target_path, error))?;
        fs::write(&temporary_path, format!("{serialized}\n"))
            .map_err(|error| AppError::io("write temporary policy file", &temporary_path, error))?;
        fs::rename(&temporary_path, &target_path)
            .map_err(|error| AppError::io("install policy file atomically", &target_path, error))?;
        report.written = true;
        Ok(report)
    }
}

fn collect_conflicts(
    desired: &Value,
    existing: &Value,
    path: &PathBuf,
    incidents: &mut Vec<Incident>,
) {
    let Some(desired_object) = desired.as_object() else {
        return;
    };
    let Some(existing_object) = existing.as_object() else {
        incidents.push(Incident::new(
            "malformed_policy_json",
            Severity::Blocking,
            path.clone(),
            "Policy JSON root must be an object.",
        ));
        return;
    };
    for (key, desired_value) in desired_object {
        if let Some(existing_value) = existing_object.get(key) {
            if existing_value != desired_value {
                incidents.push(Incident::new(
                    "policy_conflict",
                    Severity::Blocking,
                    path.clone(),
                    format!(
                        "Existing policy '{key}' has value {} but desired value is {}.",
                        compact_json(existing_value),
                        compact_json(desired_value)
                    ),
                ));
            }
        }
    }
}

fn compact_json(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "<unprintable>".to_string())
}

pub fn policy_value_to_object(value: Value) -> Map<String, Value> {
    value.as_object().cloned().unwrap_or_default()
}
