use std::ffi::NulError;

#[derive(Debug)]
pub enum RevoLibError {
    ObsStartupFailed,
    ObsNotInitialized,
    NullPointer(&'static str),
    CString(NulError),
    Other(String),
}

impl std::fmt::Display for RevoLibError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RevoLibError::ObsStartupFailed => write!(f, "obs_startup failed"),
            RevoLibError::ObsNotInitialized => write!(f, "OBS is not initialized"),
            RevoLibError::NullPointer(ctx) => write!(f, "null pointer: {ctx}"),
            RevoLibError::CString(err) => write!(f, "CString error: {err}"),
            RevoLibError::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for RevoLibError {}

impl From<NulError> for RevoLibError {
    fn from(value: NulError) -> Self {
        Self::CString(value)
    }
}

pub type RevoLibResult<T> = Result<T, RevoLibError>;
