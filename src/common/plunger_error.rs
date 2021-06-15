use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlungerError {
    #[error("Invalid target")]
    InvalidTarget,
    #[error("Invalid Read Protection Level")]
    InvalidProtectionLevel,
    #[error(transparent)]
    SessionError(#[from] probe_rs::Error),
    #[error(transparent)]
    DebugProbeError(#[from] probe_rs::DebugProbeError),
}