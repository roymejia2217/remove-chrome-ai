use crate::error::{AppError, AppResult};
use std::env;
use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
pub struct ConfigInput {
    pub home: Option<PathBuf>,
    pub xdg_config_home: Option<PathBuf>,
    pub chrome_root_override: Option<PathBuf>,
    pub policy_dir_override: Option<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeConfig {
    pub chrome_config_root: PathBuf,
    pub policy_managed_dir: PathBuf,
    pub policy_file_name: String,
}

impl RuntimeConfig {
    pub fn from_environment(
        chrome_root_override: Option<PathBuf>,
        policy_dir_override: Option<PathBuf>,
    ) -> AppResult<Self> {
        Self::resolve(ConfigInput {
            home: env::var_os("HOME").map(PathBuf::from),
            xdg_config_home: env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
            chrome_root_override,
            policy_dir_override,
        })
    }

    pub fn resolve(input: ConfigInput) -> AppResult<Self> {
        let chrome_config_root = match input.chrome_root_override {
            Some(path) => path,
            None => {
                let config_home = match input.xdg_config_home {
                    Some(path) if !path.as_os_str().is_empty() => path,
                    _ => {
                        let home = input.home.ok_or_else(|| {
                            AppError::Config(
                                "HOME is required when no Chrome root override is provided"
                                    .to_string(),
                            )
                        })?;
                        home.join(".config")
                    }
                };
                config_home.join("google-chrome")
            }
        };

        Ok(Self {
            chrome_config_root,
            policy_managed_dir: input
                .policy_dir_override
                .unwrap_or_else(|| PathBuf::from("/etc/opt/chrome/policies/managed")),
            policy_file_name: "remove-chrome-ai.json".to_string(),
        })
    }
}
