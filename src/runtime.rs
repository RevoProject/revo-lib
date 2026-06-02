use libobs_wrapper::context::ObsContext as LibObsContext;
use libobs_wrapper::utils::StartupInfo;
#[cfg(any(target_os = "windows", target_os = "macos"))]
use libobs_bootstrapper::{
    ObsBootstrapper, ObsBootstrapperOptions, ObsBootstrapperResult, UpdateTargetMode,
};

use crate::error::{RevoLibError, RevoLibResult};

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn env_truthy(key: &str) -> bool {
    std::env::var(key)
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            matches!(v.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(false)
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn maybe_bootstrap_obs() -> RevoLibResult<()> {
    if env_truthy("REVO_LIB_DISABLE_BOOTSTRAP") {
        return Ok(());
    }

    let mut options = ObsBootstrapperOptions::new();

    if let Ok(repo) = std::env::var("REVO_LIBOBS_REPOSITORY") {
        let repo = repo.trim();
        if !repo.is_empty() {
            options = options.set_repository(repo);
        }
    }

    if env_truthy("REVO_LIBOBS_NO_RESTART") {
        options = options.set_no_restart();
    }

    if env_truthy("REVO_LIBOBS_SAME_MINOR") {
        options = options.set_update_target_mode(UpdateTargetMode::LatestCompatibleSameMajorMinor);
    }

    // Allow disabling automatic update checks while still installing when missing.
    if let Ok(update_flag) = std::env::var("REVO_LIBOBS_UPDATE") {
        let update_enabled = !matches!(
            update_flag.trim().to_ascii_lowercase().as_str(),
            "0" | "false" | "no" | "off"
        );
        options = options.set_update(update_enabled);
    }

    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| RevoLibError::Other(format!("Failed to create tokio runtime: {e}")))?;

    let result = runtime
        .block_on(ObsBootstrapper::bootstrap(&options))
        .map_err(|e| RevoLibError::Other(format!("OBS bootstrap failed: {e}")))?;

    match result {
        ObsBootstrapperResult::None => Ok(()),
        ObsBootstrapperResult::Restart => Err(RevoLibError::Other(
            "OBS binaries were updated. Restart the application to continue.".to_string(),
        )),
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn maybe_bootstrap_obs() -> RevoLibResult<()> {
    Ok(())
}

pub fn libobs_git_describe() -> &'static str {
    option_env!("REVO_LIBOBS_GIT_DESCRIBE").unwrap_or("libobs-wrapper")
}

/// Initialize OBS runtime with the given locale and optional module config path
pub fn init(_locale: &str, _module_config_path: Option<&str>) -> RevoLibResult<()> {
    maybe_bootstrap_obs()?;

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
        maybe_bootstrap_obs()?;

        let inner = LibObsContext::new(StartupInfo::default())
            .map_err(|e| RevoLibError::Other(format!("Failed to create OBS context: {e:?}")))?;
        Ok(Self { _inner: inner })
    }
}
