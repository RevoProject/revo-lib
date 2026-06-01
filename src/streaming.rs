use libobs_wrapper::data::output::ObsOutputRef;

use crate::error::RevoLibResult;
use crate::output::OutputWrapper;

/// Wrapper around libobs-wrapper StreamingOutput for backward compatibility
#[derive(Clone)]
pub struct StreamingOutput {
    output: OutputWrapper,
}

impl StreamingOutput {
    /// Create from a raw ObsOutputRef
    pub fn from_raw(output: ObsOutputRef) -> RevoLibResult<Self> {
        Ok(Self {
            output: OutputWrapper::new(output, "streaming"),
        })
    }

    /// Get reference to the underlying output
    pub fn as_raw(&self) -> ObsOutputRef {
        self.output.as_raw()
    }

    /// Start streaming
    pub fn start(&self) -> RevoLibResult<()> {
        self.output.start()
    }

    /// Stop streaming
    pub fn stop(&self) {
        self.output.stop();
    }

    /// Check if streaming is active
    pub fn active(&self) -> bool {
        self.output.active()
    }
}
