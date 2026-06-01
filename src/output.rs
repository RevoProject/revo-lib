use libobs_wrapper::data::output::{ObsOutputRef, ObsOutputTrait};

use crate::error::{RevoLibError, RevoLibResult};

#[derive(Clone)]
pub(crate) struct OutputWrapper {
    output: ObsOutputRef,
    kind: &'static str,
}

impl OutputWrapper {
    pub(crate) fn new(output: ObsOutputRef, kind: &'static str) -> Self {
        Self { output, kind }
    }

    pub(crate) fn as_raw(&self) -> ObsOutputRef {
        self.output.clone()
    }

    pub(crate) fn start(&self) -> RevoLibResult<()> {
        (&self.output)
            .start()
            .map_err(|e| RevoLibError::Other(format!("failed to start {} output: {e:?}", self.kind)))
    }

    pub(crate) fn stop(&self) {
        let mut output = self.output.clone();
        let _ = output.stop();
    }

    pub(crate) fn active(&self) -> bool {
        self.output.is_active().unwrap_or(false)
    }
}