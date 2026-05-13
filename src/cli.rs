use crate::chrome_detector::ChromeDetector;
use crate::cleaner::{Cleaner, CleanupMode};
use crate::config::RuntimeConfig;
use crate::error::{AppError, AppResult};
use crate::incident_detector::{Incident, IncidentDetector, Severity};
use crate::model_scanner::ModelScanner;
use crate::policy_manager::{PolicyManager, PolicyMode, PolicyWriteMode};
use crate::reporter::{CommandReport, OutputMode, Reporter, has_blocking_incident};
use std::env;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Command {
    Scan,
    Clean,
    Protect,
    Repair,
    Status,
    Doctor,
    InternalPolicyWrite,
}

#[derive(Clone, Debug)]
struct CliOptions {
    command: Command,
    output_mode: OutputMode,
    cleanup_mode: CleanupMode,
    policy_mode: PolicyMode,
    chrome_root_override: Option<PathBuf>,
    policy_dir_override: Option<PathBuf>,
}

pub fn run_from_env() -> i32 {
    match run(std::env::args().skip(1).collect()) {
        Ok((output, has_blocking)) => {
            if !output.is_empty() {
                println!("{output}");
            }
            if has_blocking { 2 } else { 0 }
        }
        Err(error) => {
            eprintln!("{error}");
            1
        }
    }
}

pub fn run(args: Vec<String>) -> AppResult<(String, bool)> {
    let options = parse_args(args)?;
    let output_mode = options.output_mode;
    let internal_policy_write = options.command == Command::InternalPolicyWrite;
    let config = RuntimeConfig::from_environment(
        options.chrome_root_override.clone(),
        options.policy_dir_override.clone(),
    )?;
    let report = execute(options, config)?;
    if internal_policy_write {
        return Ok((String::new(), has_blocking_incident(&report)));
    }
    let output = Reporter::render(&report, output_mode);
    let blocked = has_blocking_incident(&report);
    Ok((output, blocked))
}

fn execute(options: CliOptions, config: RuntimeConfig) -> AppResult<CommandReport> {
    let scan = ModelScanner::new(config.chrome_config_root.clone()).scan()?;
    let environment_incidents = environment_incidents();
    let cleaner = Cleaner::new(config.chrome_config_root.clone());
    let policy_manager = PolicyManager::new(
        config.policy_managed_dir.clone(),
        config.policy_file_name.clone(),
    );

    let (cleanup, policy) = match options.command {
        Command::Scan | Command::Status | Command::Doctor => (None, None),
        Command::Clean | Command::Repair => {
            let cleanup = cleaner.clean(&scan.findings, options.cleanup_mode)?;
            let policy = apply_policy_with_optional_elevation(
                &config,
                options.policy_mode,
                options.cleanup_mode,
                &policy_manager,
            )?;
            (Some(cleanup), Some(policy))
        }
        Command::Protect => {
            let policy = apply_policy_with_optional_elevation(
                &config,
                options.policy_mode,
                options.cleanup_mode,
                &policy_manager,
            )?;
            (None, Some(policy))
        }
        Command::InternalPolicyWrite => {
            let policy = policy_manager.apply(
                options.policy_mode,
                policy_write_mode_from_cleanup(options.cleanup_mode),
            )?;
            (None, Some(policy))
        }
    };

    Ok(CommandReport {
        action: format!("{:?}", options.command).to_ascii_lowercase(),
        scan,
        cleanup,
        policy,
        environment_incidents,
    })
}

fn apply_policy_with_optional_elevation(
    config: &RuntimeConfig,
    policy_mode: PolicyMode,
    cleanup_mode: CleanupMode,
    policy_manager: &PolicyManager,
) -> AppResult<crate::policy_manager::PolicyReport> {
    let write_mode = policy_write_mode_from_cleanup(cleanup_mode);
    match policy_manager.apply(policy_mode, write_mode) {
        Ok(report) => Ok(report),
        Err(error)
            if write_mode == PolicyWriteMode::Apply
                && error.is_permission_denied()
                && env::var_os("REMOVE_CHROME_AI_ELEVATED").is_none() =>
        {
            run_elevated_policy_writer(config, policy_mode)
        }
        Err(error) => Err(error),
    }
}

fn run_elevated_policy_writer(
    config: &RuntimeConfig,
    policy_mode: PolicyMode,
) -> AppResult<crate::policy_manager::PolicyReport> {
    let executable = env::current_exe().map_err(|error| {
        AppError::io(
            "resolve current executable for privilege elevation",
            PathBuf::from("current executable"),
            error,
        )
    })?;
    let launcher = find_elevation_launcher().ok_or_else(|| {
        AppError::Config(
            "policy installation requires root privileges, but neither sudo nor pkexec was found"
                .to_string(),
        )
    })?;

    let mut command = ProcessCommand::new(&launcher);
    if launcher == "sudo" {
        command.arg("--preserve-env=REMOVE_CHROME_AI_ELEVATED");
        command.arg("env");
    } else {
        command.arg("env");
    }
    command.arg("REMOVE_CHROME_AI_ELEVATED=1");
    command.arg(executable);
    command.arg("__write-policy");
    command.arg("--policy-dir");
    command.arg(&config.policy_managed_dir);
    if policy_mode == PolicyMode::Strict {
        command.arg("--strict");
    }

    let status = command.status().map_err(|error| {
        AppError::io(
            "launch privilege elevation command",
            PathBuf::from(&launcher),
            error,
        )
    })?;

    if !status.success() {
        return Err(AppError::Config(format!(
            "elevated policy installation failed with status {status}"
        )));
    }

    Ok(crate::policy_manager::PolicyReport {
        path: config.policy_managed_dir.join(&config.policy_file_name),
        written: true,
        desired: PolicyManager::desired_policy(policy_mode),
        incidents: vec![Incident::new(
            "policy_elevated",
            Severity::Info,
            config.policy_managed_dir.clone(),
            "Policy installation was completed through system privilege elevation.",
        )],
    })
}

fn find_elevation_launcher() -> Option<String> {
    ["sudo", "pkexec"]
        .iter()
        .find(|candidate| command_exists(candidate))
        .map(|candidate| (*candidate).to_string())
}

fn command_exists(binary_name: &str) -> bool {
    let Some(path) = env::var_os("PATH") else {
        return false;
    };
    env::split_paths(&path).any(|directory| directory.join(binary_name).is_file())
}

fn environment_incidents() -> Vec<Incident> {
    let mut incidents = IncidentDetector::detect_chrome_processes().incidents;
    if ChromeDetector::detect_stable().is_none() {
        incidents.push(Incident::new(
            "chrome_stable_not_found",
            Severity::Info,
            PathBuf::from("PATH"),
            "Google Chrome stable binary was not found in PATH; filesystem cleanup can still run against the resolved Chrome config root.",
        ));
    }
    incidents
}

fn policy_write_mode_from_cleanup(mode: CleanupMode) -> PolicyWriteMode {
    match mode {
        CleanupMode::Apply => PolicyWriteMode::Apply,
        CleanupMode::DryRun => PolicyWriteMode::DryRun,
    }
}

fn parse_args(args: Vec<String>) -> AppResult<CliOptions> {
    let mut command = Command::Clean;
    let mut output_mode = OutputMode::Human;
    let mut cleanup_mode = CleanupMode::Apply;
    let mut policy_mode = PolicyMode::Core;
    let mut chrome_root_override = None;
    let mut policy_dir_override = None;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "scan" => command = Command::Scan,
            "clean" => command = Command::Clean,
            "protect" => command = Command::Protect,
            "repair" => command = Command::Repair,
            "status" => command = Command::Status,
            "doctor" => command = Command::Doctor,
            "__write-policy" => command = Command::InternalPolicyWrite,
            "--json" => output_mode = OutputMode::Json,
            "--dry-run" => cleanup_mode = CleanupMode::DryRun,
            "--strict" => policy_mode = PolicyMode::Strict,
            "--chrome-root" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    AppError::InvalidArgument("--chrome-root requires a path".to_string())
                })?;
                chrome_root_override = Some(PathBuf::from(value));
            }
            "--policy-dir" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    AppError::InvalidArgument("--policy-dir requires a path".to_string())
                })?;
                policy_dir_override = Some(PathBuf::from(value));
            }
            "--help" | "-h" => {
                return Err(AppError::InvalidArgument(help_text()));
            }
            unknown => {
                return Err(AppError::InvalidArgument(format!(
                    "unknown option or command '{unknown}'\n{}",
                    help_text()
                )));
            }
        }
        index += 1;
    }

    Ok(CliOptions {
        command,
        output_mode,
        cleanup_mode,
        policy_mode,
        chrome_root_override,
        policy_dir_override,
    })
}

fn help_text() -> String {
    "Usage: remove-chrome-ai [scan|clean|protect|repair|status|doctor] [--dry-run] [--strict] [--json] [--chrome-root PATH] [--policy-dir PATH]".to_string()
}
