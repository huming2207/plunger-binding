use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlungerError {
    #[error("Invalid target: {0}")]
    InvalidTarget(String),
    #[error("Invalid Read Protection Level")]
    InvalidProtectionLevel,
    #[error(transparent)]
    ProbeRsSessionError(#[from] probe_rs::Error),
    #[error(transparent)]
    ProbeFlashingError(#[from] probe_rs::flashing::FlashError),
    #[error(transparent)]
    ProbeRsCommError(#[from] probe_rs::DebugProbeError),
    #[error("Invalid state: {0}")]
    StateError(String),
}

impl From<PlungerError> for napi::Error {
    fn from(err: PlungerError) -> Self {
        napi::Error {
            status: match err {
                PlungerError::InvalidTarget(_) => napi::Status::InvalidArg,
                PlungerError::InvalidProtectionLevel => napi::Status::GenericFailure,
                PlungerError::ProbeRsSessionError(_) => napi::Status::GenericFailure,
                PlungerError::ProbeRsCommError(_) => napi::Status::GenericFailure,
                PlungerError::StateError(_) => napi::Status::Unknown,
                PlungerError::ProbeFlashingError(_) => napi::Status::GenericFailure,
            },
            reason: err.to_string(),
        }
    }
}
