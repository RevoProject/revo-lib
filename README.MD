# revo-lib

![License](https://img.shields.io/github/license/obsproject/obs-studio)
![Platform](https://img.shields.io/badge/platform-linux-blue) 
![Rust](https://img.shields.io/badge/rust-stable-orange)

Safe Rust wrapper around **libobs-wrapper** for streaming and recording functionality

> [!INFO]
> ✅ **Migration to libobs-rs completed** — Using stable, production-grade libobs-wrapper bindings instead of manually built obs-studio with bindgen

> [!WARNING]
> The current release only works on Linux based systems

## 💻 Usage

- **RevoStream** — *coming soon*

## 📦 Installation

Add to your Cargo.toml:

```toml
[dependencies]
revo-lib = { git = "https://github.com/revoproject/revo-lib" }
```

## 📋 Migration Summary

### What Changed

| Aspect | Before | After |
|--------|--------|-------|
| **FFI Bindings** | Manual bindgen from C headers | libobs-wrapper pre-built bindings |
| **Build Complexity** | Clone obs-studio, CMake, compile (5+ min) | Use pre-built crate (~30s) |
| **Local Size** | 231MB (obs-studio sources) | 0MB (external dependency) |
| **Output Wrappers** | Raw `*mut obs_output` pointers | Safe `ObsOutputRef` with trait-based API |
| **Threading** | Atomic flags for init state | Thread-safe context with RAII |
| **Error Handling** | Custom `RevoLibError` enum | Conversion from `libobs_wrapper::ObsError` |

### Architecture Improvements

**Before:**
- ❌ Manual obs-studio cloning (231MB+)
- ❌ Bindgen FFI generation at build time
- ❌ Raw C pointer management
- ❌ Duplicated output code (StreamingOutput + RecordingOutput)
- ❌ Single-threaded initialization

**After:**
- ✅ Pre-built, production-grade libobs-wrapper
- ✅ Safe, idiomatic Rust abstractions
- ✅ Trait-based polymorphic outputs (`ObsOutputTrait`)
- ✅ DRY principle (unified output handling)
- ✅ Thread-safe context with automatic cleanup (RAII)

## 📦 Dependencies

| Crate | Purpose | Version |
|-------|---------|---------|
| `libobs-wrapper` | Safe Rust wrapper around libobs | From GitHub main |

**Removed:**
- ~~`bindgen`~~ — Replaced by libobs-wrapper
- ~~`libc`~~ — Handled by libobs-wrapper
- ~~obs-studio sources~~ — No longer needed

## 📚 API Reference

### Core Types

```rust
// Runtime initialization
let ctx = revo_lib::runtime::ObsContext::startup("en-US", None)?;

// Streaming
use revo_lib::streaming::StreamingOutput;
let stream = StreamingOutput::from_raw(output)?;
stream.start()?;
stream.stop();

// Recording
use revo_lib::recording::RecordingOutput;
let recording = RecordingOutput::from_raw(output)?;
recording.start()?;
recording.stop();
```

### Error Handling

```rust
use revo_lib::error::{RevoLibError, RevoLibResult};

match some_operation() {
    Ok(result) => { /* ... */ }
    Err(RevoLibError::ObsStartupFailed) => eprintln!("OBS init failed"),
    Err(RevoLibError::Other(msg)) => eprintln!("Error: {}", msg),
    Err(e) => eprintln!("Other error: {}", e),
}
```

## 🔗 References

- [libobs-wrapper documentation](https://docs.rs/libobs-wrapper/)
- [libobs-rs repository](https://github.com/libobs-rs/libobs-rs)
- [OBS Studio documentation](https://obsproject.com/)

## 📜 Additional Scripts

- [cleanup.sh](cleanup.sh) - Clean up all built files

## 📝 License

This project is licensed under the same terms as OBS Studio (GNU General Public License v3.0 or later).
