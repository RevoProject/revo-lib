use crate::error::{RevoLibError, RevoLibResult};
use crate::obs;
use crate::runtime;

#[derive(Clone, Copy)]
pub struct StreamingOutput {
    ptr: *mut obs::obs_output,
}

impl StreamingOutput {
    pub fn from_raw(ptr: *mut obs::obs_output) -> RevoLibResult<Self> {
        if ptr.is_null() {
            return Err(RevoLibError::NullPointer("stream output"));
        }
        Ok(Self { ptr })
    }

    pub fn as_raw(&self) -> *mut obs::obs_output {
        self.ptr
    }

    pub fn start(&self) -> RevoLibResult<()> {
        if !runtime::is_initialized() {
            return Err(RevoLibError::ObsNotInitialized);
        }
        let started = unsafe { obs::obs_output_start(self.ptr) };
        if started {
            Ok(())
        } else {
            Err(RevoLibError::Other("failed to start stream output".to_string()))
        }
    }

    pub fn stop(&self) {
        unsafe {
            obs::obs_output_stop(self.ptr);
        }
    }

    pub fn active(&self) -> bool {
        unsafe { obs::obs_output_active(self.ptr) }
    }
}
