use libobs_wrapper::data::output::ObsOutputRef;

use crate::error::RevoLibResult;
use crate::output::OutputWrapper;

/// Wrapper around libobs-wrapper RecordingOutput for backward compatibility
#[derive(Clone)]
pub struct RecordingOutput {
    output: OutputWrapper,
}

impl RecordingOutput {
    /// Create from a raw ObsOutputRef
    pub fn from_raw(output: ObsOutputRef) -> RevoLibResult<Self> {
        Ok(Self {
            output: OutputWrapper::new(output, "recording"),
        })
    }

    /// Get reference to the underlying output
    pub fn as_raw(&self) -> ObsOutputRef {
        self.output.as_raw()
    }

    /// Start recording
    pub fn start(&self) -> RevoLibResult<()> {
        self.output.start()
    }

    /// Stop recording
    pub fn stop(&self) {
        self.output.stop();
    }

    /// Check if recording is active
    pub fn active(&self) -> bool {
        self.output.active()
    }
}
