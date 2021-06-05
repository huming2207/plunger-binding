use thiserror::Error;

#[derive(Error, Debug)]
pub enum EraserError {
    #[error("Invalid target - probably not a STM32L0?")]
    InvalidTarget,
    #[error("Invalid Readout Protection Level - RDP has been set to 2")]
    InvalidProtectionLevel,
    #[error(transparent)]
    SessionError(#[from] probe_rs::Error),
    #[error(transparent)]
    DebugProbeError(#[from] probe_rs::DebugProbeError),
}