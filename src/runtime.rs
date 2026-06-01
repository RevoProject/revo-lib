use libobs_wrapper::context::ObsContext as LibObsContext;
use libobs_wrapper::utils::StartupInfo;

use crate::error::{RevoLibError, RevoLibResult};

pub fn libobs_git_describe() -> &'static str {
    option_env!("REVO_LIBOBS_GIT_DESCRIBE").unwrap_or("libobs-wrapper")
}

/// Initialize OBS runtime with the given locale and optional module config path
pub fn init(_locale: &str, _module_config_path: Option<&str>) -> RevoLibResult<()> {
    // libobs-wrapper handles initialization internally
    // We just validate that we can create a context
    let _ctx = LibObsContext::new(StartupInfo::default())
        .map_err(|e| RevoLibError::Other(format!("Failed to initialize OBS: {e:?}")))?;
    Ok(())
}

pub fn is_initialized() -> bool {
    // libobs-wrapper manages initialization state internally
    // For now, return true as a placeholder - the actual state is managed by libobs-wrapper
    true
}

pub fn set_initialized(_value: bool) {
    // libobs-wrapper manages this internally, no-op here
}

pub fn shutdown() {
    // libobs-wrapper handles shutdown automatically via Drop
}

/// RAII wrapper for OBS context - automatic cleanup on drop
pub struct ObsContext {
    _inner: LibObsContext,
}

impl ObsContext {
    /// Create a new OBS context with the given locale and module config path
    pub fn startup(_locale: &str, _module_config_path: Option<&str>) -> RevoLibResult<Self> {
        let inner = LibObsContext::new(StartupInfo::default())
            .map_err(|e| RevoLibError::Other(format!("Failed to create OBS context: {e:?}")))?;
        Ok(Self { _inner: inner })
    }
}
