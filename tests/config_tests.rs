use remove_chrome_ai::config::{ConfigInput, RuntimeConfig};
use std::path::PathBuf;

#[test]
fn resolves_xdg_chrome_root_when_set() {
    let input = ConfigInput {
        home: Some(PathBuf::from("/home/alex")),
        xdg_config_home: Some(PathBuf::from("/custom/config")),
        chrome_root_override: None,
        policy_dir_override: None,
    };

    let config = RuntimeConfig::resolve(input).expect("config should resolve");

    assert_eq!(
        config.chrome_config_root,
        PathBuf::from("/custom/config/google-chrome")
    );
}

#[test]
fn falls_back_to_home_config_when_xdg_is_missing() {
    let input = ConfigInput {
        home: Some(PathBuf::from("/home/alex")),
        xdg_config_home: None,
        chrome_root_override: None,
        policy_dir_override: None,
    };

    let config = RuntimeConfig::resolve(input).expect("config should resolve");

    assert_eq!(
        config.chrome_config_root,
        PathBuf::from("/home/alex/.config/google-chrome")
    );
}

#[test]
fn fails_when_home_is_missing_and_no_chrome_root_override_exists() {
    let input = ConfigInput {
        home: None,
        xdg_config_home: None,
        chrome_root_override: None,
        policy_dir_override: None,
    };

    assert!(RuntimeConfig::resolve(input).is_err());
}
