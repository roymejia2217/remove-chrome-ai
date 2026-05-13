use remove_chrome_ai::cli;
use serde_json::json;
use std::fs;

#[test]
fn internal_policy_writer_writes_policy_without_user_cleanup() {
    let temp = tempfile::tempdir().unwrap();
    let policy_dir = temp.path().join("managed");

    let (output, blocked) = cli::run(vec![
        "__write-policy".to_string(),
        "--policy-dir".to_string(),
        policy_dir.display().to_string(),
        "--strict".to_string(),
    ])
    .expect("internal policy writer should succeed");

    let written = fs::read_to_string(policy_dir.join("remove-chrome-ai.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&written).unwrap();

    assert!(output.is_empty());
    assert!(!blocked);
    assert_eq!(
        parsed,
        json!({
            "GenAILocalFoundationalModelSettings": 1,
            "GenAiDefaultSettings": 2,
            "DevToolsGenAiSettings": 2
        })
    );
}
