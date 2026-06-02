#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

// Re-export public types from libobs-wrapper for backward compatibility
pub use libobs_wrapper::*;
// Re-export the low-level generated FFI (`libobs`) symbols so older code
// referencing `revo_lib::obs::...` continues to work.
pub use libobs_wrapper::sys::*;
