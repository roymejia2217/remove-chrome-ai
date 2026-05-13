use remove_chrome_ai::policy_manager::{PolicyManager, PolicyMode, PolicyWriteMode};
use serde_json::json;
use std::fs;

#[test]
fn generates_core_policy_json() {
    let value = PolicyManager::desired_policy(PolicyMode::Core);

    assert_eq!(
        value,
        json!({
            "GenAILocalFoundationalModelSettings": 1
        })
    );
}

#[test]
fn generates_strict_policy_json() {
    let value = PolicyManager::desired_policy(PolicyMode::Strict);

    assert_eq!(
        value,
        json!({
            "GenAILocalFoundationalModelSettings": 1,
            "GenAiDefaultSettings": 2,
            "DevToolsGenAiSettings": 2
        })
    );
}

#[test]
fn detects_conflicting_existing_policy_file() {
    let temp = tempfile::tempdir().unwrap();
    let policy_dir = temp.path().join("managed");
    fs::create_dir_all(&policy_dir).unwrap();
    fs::write(
        policy_dir.join("existing.json"),
        r#"{"GenAILocalFoundationalModelSettings":0}"#,
    )
    .unwrap();

    let manager = PolicyManager::new(policy_dir, "remove-chrome-ai.json".to_string());
    let report = manager
        .apply(PolicyMode::Core, PolicyWriteMode::Apply)
        .expect("policy manager should report conflict without failing");

    assert!(!report.written);
    assert!(report.incidents.iter().any(|incident| {
        incident.code == "policy_conflict"
            && incident.path.ends_with("existing.json")
            && incident
                .message
                .contains("GenAILocalFoundationalModelSettings")
    }));
}

#[test]
fn writes_policy_atomically_when_no_conflicts_exist() {
    let temp = tempfile::tempdir().unwrap();
    let policy_dir = temp.path().join("managed");

    let manager = PolicyManager::new(policy_dir.clone(), "remove-chrome-ai.json".to_string());
    let report = manager
        .apply(PolicyMode::Core, PolicyWriteMode::Apply)
        .expect("policy write should succeed");

    let written = fs::read_to_string(policy_dir.join("remove-chrome-ai.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&written).unwrap();

    assert!(report.written);
    assert_eq!(parsed, PolicyManager::desired_policy(PolicyMode::Core));
    assert!(!policy_dir.join("remove-chrome-ai.json.tmp").exists());
}
