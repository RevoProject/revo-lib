use std::ffi::CString;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::error::{RevoLibError, RevoLibResult};
use crate::obs;

static OBS_INITIALIZED: AtomicBool = AtomicBool::new(false);

pub fn init(locale: &str, module_config_path: Option<&str>) -> RevoLibResult<()> {
    if OBS_INITIALIZED.load(Ordering::SeqCst) {
        return Ok(());
    }

    let locale_c = CString::new(locale)?;
    let conf_c = CString::new(module_config_path.unwrap_or(""))?;

    let ok = unsafe { obs::obs_startup(locale_c.as_ptr(), conf_c.as_ptr(), std::ptr::null_mut()) };
    if !ok {
        return Err(RevoLibError::ObsStartupFailed);
    }

    unsafe {
        obs::obs_load_all_modules();
        obs::obs_post_load_modules();
    }

    OBS_INITIALIZED.store(true, Ordering::SeqCst);
    Ok(())
}

pub fn is_initialized() -> bool {
    OBS_INITIALIZED.load(Ordering::SeqCst)
}

pub fn set_initialized(value: bool) {
    OBS_INITIALIZED.store(value, Ordering::SeqCst);
}

pub fn shutdown() {
    if OBS_INITIALIZED.swap(false, Ordering::SeqCst) {
        unsafe {
            obs::obs_shutdown();
        }
    }
}

pub struct ObsContext;

impl ObsContext {
    pub fn startup(locale: &str, module_config_path: Option<&str>) -> RevoLibResult<Self> {
        init(locale, module_config_path)?;
        Ok(Self)
    }
}

impl Drop for ObsContext {
    fn drop(&mut self) {
        shutdown();
    }
}
