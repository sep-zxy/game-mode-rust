use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("Config error: {0}")]
    Config(String),
    #[error("Permission error: {0}")]
    Permission(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Process error: {0}")]
    Process(String),
    #[error("System error: {0}")]
    System(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Conflict: {0}")]
    Conflict(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    pub fn to_user_message(&self) -> String {
        match self {
            AppError::Io(message) => format!("[IO] {message}"),
            AppError::Config(message) => format!("[CONFIG] {message}"),
            AppError::Permission(message) => format!("[PERMISSION] {message}"),
            AppError::Network(message) => format!("[NETWORK] {message}"),
            AppError::Process(message) => format!("[PROCESS] {message}"),
            AppError::System(message) => format!("[SYSTEM] {message}"),
            AppError::Validation(message) => format!("[VALIDATION] {message}"),
            AppError::NotFound(message) => format!("[NOT_FOUND] {message}"),
            AppError::Conflict(message) => format!("[CONFLICT] {message}"),
        }
    }
}

impl From<io::Error> for AppError {
    fn from(value: io::Error) -> Self {
        AppError::Io(value.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        AppError::Network(value.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        AppError::Config(value.to_string())
    }
}
