use thiserror::Error;

pub type Result<T> = std::result::Result<T, CoreError>;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Physics error: {0}")]
    Physics(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Invalid stage: {0}")]
    InvalidStage(i32),

    #[error("Invalid state transition")]
    InvalidTransition,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
