use std::fmt::{Display, Formatter};
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum AppError {
    Config(String),
    Io {
        action: String,
        path: PathBuf,
        source: io::Error,
    },
    Json {
        action: String,
        path: PathBuf,
        source: serde_json::Error,
    },
    InvalidArgument(String),
}

impl AppError {
    pub fn io(action: impl Into<String>, path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            action: action.into(),
            path: path.into(),
            source,
        }
    }

    pub fn json(
        action: impl Into<String>,
        path: impl Into<PathBuf>,
        source: serde_json::Error,
    ) -> Self {
        Self::Json {
            action: action.into(),
            path: path.into(),
            source,
        }
    }

    pub fn is_permission_denied(&self) -> bool {
        match self {
            Self::Io { source, .. } => source.kind() == io::ErrorKind::PermissionDenied,
            _ => false,
        }
    }
}

impl Display for AppError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(message) => write!(formatter, "configuration error: {message}"),
            Self::Io {
                action,
                path,
                source,
            } => {
                write!(
                    formatter,
                    "failed to {action} '{}': {source}",
                    path.display()
                )
            }
            Self::Json {
                action,
                path,
                source,
            } => {
                write!(
                    formatter,
                    "failed to {action} '{}': {source}",
                    path.display()
                )
            }
            Self::InvalidArgument(message) => write!(formatter, "invalid argument: {message}"),
        }
    }
}

impl std::error::Error for AppError {}

pub type AppResult<T> = Result<T, AppError>;
