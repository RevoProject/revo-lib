use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const DEFAULT_OBS_REF: &str = "32.1.0-rc1";

fn run(cmd: &mut Command, step: &str) -> Result<(), Box<dyn Error>> {
    println!("cargo:warning=Running step: {step}");
    let output = cmd.output()?;
    println!("cargo:warning=stdout: {}", String::from_utf8_lossy(&output.stdout));
    eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    if !output.status.success() {
        return Err(format!("step failed: {step}").into());
    }
    Ok(())
}

fn nproc() -> String {
    std::thread::available_parallelism()
        .map(|n| n.get().to_string())
        .unwrap_or_else(|_| "4".to_string())
}

fn main() -> Result<(), Box<dyn Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);

    // ── Fast-path: use a pre-built libobs artifact ──────────────────────────
    // Set REVO_PREBUILT_LIBOBS_DIR to the directory downloaded from CI
    // (e.g. libobs-prebuilt-macos-x86_64/) and the full OBS compilation is
    // skipped.  Layout expected:
    //   <dir>/lib/          – libobs.dylib / libobs.so / libobs.a
    //   <dir>/include/      – public headers
    //   <dir>/bindings/libobs_bindings.rs  – pre-generated Rust FFI bindings
    if let Ok(prebuilt) = env::var("REVO_PREBUILT_LIBOBS_DIR") {
        let prebuilt = PathBuf::from(&prebuilt);
        println!("cargo:warning=Using prebuilt libobs from: {}", prebuilt.display());
        println!("cargo:rerun-if-env-changed=REVO_PREBUILT_LIBOBS_DIR");

        // Copy pre-generated bindings into OUT_DIR
        let out_dir = PathBuf::from(env::var("OUT_DIR")?);
        let src_bindings = prebuilt.join("bindings/libobs_bindings.rs");
        if src_bindings.exists() {
            fs::copy(&src_bindings, out_dir.join("libobs_bindings.rs"))?;
        } else {
            return Err(format!(
                "REVO_PREBUILT_LIBOBS_DIR is set but bindings/libobs_bindings.rs not found in {}",
                prebuilt.display()
            ).into());
        }

        // Link against the prebuilt library
        if cfg!(target_os = "macos") {
            // macOS builds libobs as a framework: framework/libobs.framework
            println!(
                "cargo:rustc-link-search=framework={}",
                prebuilt.join("framework").display()
            );
            println!("cargo:rustc-link-lib=framework=obs");
        } else {
            println!(
                "cargo:rustc-link-search=native={}",
                prebuilt.join("lib").display()
            );
            println!("cargo:rustc-link-lib=dylib=obs");
            println!("cargo:rustc-link-lib=dylib=dl");
            println!("cargo:rustc-link-lib=dylib=pthread");
            println!("cargo:rustc-link-lib=dylib=m");
        }

        return Ok(());
    }
    // ────────────────────────────────────────────────────────────────────────

    let obs_ref = env::var("REVO_OBS_REF").unwrap_or_else(|_| DEFAULT_OBS_REF.to_string());

    let obs_root = manifest_dir.join("obs-libobs");
    let obs_src = obs_root.join("obs-studio");
    let build_dir = obs_src.join("build-headless");
    let install_prefix = build_dir.join("install");

    // 1) Fetch obs-studio source into revo-lib/obs-libobs
    if !obs_src.exists() {
        fs::create_dir_all(&obs_root)?;
        run(
            Command::new("git")
                .arg("clone")
                .arg("https://github.com/obsproject/obs-studio.git")
                .arg(&obs_src),
            "git clone obs-studio",
        )?;
    }

    // Pin obs-studio to the known-good revision.
    run(
        Command::new("git")
            .arg("-C")
            .arg(&obs_src)
            .arg("fetch")
            .arg("origin")
            .arg(&obs_ref),
        "git fetch pinned commit",
    )?;
    run(
        Command::new("git")
            .arg("-C")
            .arg(&obs_src)
            .arg("checkout")
            .arg("--force")
            .arg(&obs_ref),
        "git checkout pinned commit",
    )?;

    let obs_describe = Command::new("git")
        .arg("-C")
        .arg(&obs_src)
        .arg("describe")
        .arg("--tags")
        .arg("--always")
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
            } else {
                None
            }
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| obs_ref.clone());
    println!("cargo:rustc-env=REVO_LIBOBS_GIT_DESCRIBE={obs_describe}");
    println!("cargo:warning=Revo-lib pinned OBS ref: {obs_describe}");

    // Ensure submodules match pinned commit.
    run(
        Command::new("git")
            .arg("-C")
            .arg(&obs_src)
            .arg("submodule")
            .arg("sync")
            .arg("--recursive"),
        "git submodule sync",
    )?;
    run(
        Command::new("git")
            .arg("-C")
            .arg(&obs_src)
            .arg("submodule")
            .arg("update")
            .arg("--init")
            .arg("--recursive"),
        "git submodule update",
    )?;

    // 2) rm -rf build-headless && mkdir build-headless
    if build_dir.exists() {
        fs::remove_dir_all(&build_dir)?;
    }
    fs::create_dir_all(&build_dir)?;

   // 3) Configure OBS
    let mut cmake = Command::new("cmake");
    cmake.arg("..")
        .arg("-DCMAKE_BUILD_TYPE=Release")
        .arg("-DENABLE_UI=OFF")
        .arg("-DENABLE_FRONTEND=OFF")
        .arg("-DENABLE_WEBSOCKET=OFF")
        .arg("-DENABLE_BROWSER=OFF")
        .arg("-DENABLE_AJA=OFF")
        .arg("-DENABLE_NVENC=OFF")
        .arg("-DENABLE_QSV11=OFF")
        .arg("-DENABLE_VST=OFF")
        .arg("-DENABLE_NEW_MPEGTS_OUTPUT=OFF")
        .arg("-DENABLE_PLUGINS=OFF")
        .arg("-DBUILD_TESTS=OFF");
    
    // --- PLATFORM-SPECIFIC FLAGS ---
    
    if cfg!(target_os = "macos") {
        // --- Patch libobs-metal LINKER_LANGUAGE ---
        println!("cargo:warning=Step: patch libobs-metal linker language");
        let mut patched_any = false;
        for entry in walkdir::WalkDir::new(&obs_src)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name() == "CMakeLists.txt")
        {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if content.contains("libobs-metal") && !content.contains("LINKER_LANGUAGE") {
                    println!("cargo:warning=Patching: {}", entry.path().display());
                    let patched = content + "\nset_target_properties(libobs-metal PROPERTIES LINKER_LANGUAGE CXX)\n";
                    fs::write(entry.path(), patched)?;
                    patched_any = true;
                }
            }
        }
        if !patched_any {
            println!("cargo:warning=No libobs-metal target found to patch");
        }

        // --- Install required dependencies via Homebrew ---
        println!("cargo:warning=Step: install deps via brew");
        for pkg in &["simde", "ffmpeg", "mbedtls", "speexdsp", "uthash", "jansson"] {
            println!("cargo:warning=brew install {pkg}");
            let status = Command::new("brew")
                .arg("install")
                .arg(pkg)
                .status()?;
            if !status.success() {
                println!("cargo:warning=brew install {pkg} failed or already installed, continuing");
            }
        }
    
        // --- Patch buildspec.cmake to skip dep downloads (no network) ---
        println!("cargo:warning=Step: patch buildspec.cmake to skip downloads");
        let buildspec_macos = obs_src.join("cmake/macos/buildspec.cmake");
        if buildspec_macos.exists() {
            println!("cargo:warning=Replacing buildspec.cmake with no-op");
            fs::write(&buildspec_macos, "# PATCHED: no-op, skip dependency downloads\n")?;
        } else {
            println!("cargo:warning=buildspec.cmake not found, skipping");
        }

        // --- Patch compilerconfig.cmake to remove Xcode-only and version requirements ---
        println!("cargo:warning=Step: patch compilerconfig.cmake to allow Ninja");
        let compilerconfig = obs_src.join("cmake/macos/compilerconfig.cmake");
        if compilerconfig.exists() {
            println!("cargo:warning=Replacing compilerconfig.cmake with no-op");
            fs::write(&compilerconfig, "# PATCHED: no-op, skip Xcode/SDK requirements\n")?;
        } else {
            println!("cargo:warning=compilerconfig.cmake not found, skipping");
        }
    
        // --- Configure cmake flags ---
        println!("cargo:warning=Step: configure cmake flags (macos)");
        
        // Determine Homebrew prefix based on architecture
        let brew_prefix = if cfg!(target_arch = "aarch64") {
            "/opt/homebrew"
        } else {
            "/usr/local"
        };
        
        cmake
            .arg("-G").arg("Ninja")
            .arg(format!("-DCMAKE_PREFIX_PATH={}", brew_prefix))
            .arg(format!("-DCMAKE_INSTALL_PREFIX={}", install_prefix.display()))
            .arg("-DCMAKE_CXX_STANDARD=17")
            .arg("-DENABLE_SCRIPTING=OFF")
            .arg("-DENABLE_VIRTUALCAM=OFF")
            .arg("-DENABLE_PIPEWIRE=OFF");
        let arch = if cfg!(target_arch = "aarch64") { "arm64" } else { "x86_64" };
        cmake.arg(format!("-DCMAKE_OSX_ARCHITECTURES={}", arch));
    
    } else if cfg!(target_os = "windows") {
        // --- Configure cmake flags (windows) ---
        println!("cargo:warning=Step: configure cmake flags (windows)");

        // Patch windows/compilerconfig.cmake to remove SDK version check
        println!("cargo:warning=Step: patch windows/compilerconfig.cmake");
        let win_compilerconfig = obs_src.join("cmake/windows/compilerconfig.cmake");
        if win_compilerconfig.exists() {
            println!("cargo:warning=Replacing windows/compilerconfig.cmake with no-op");
            fs::write(&win_compilerconfig, "# PATCHED: no-op, skip SDK version requirements\n")?;
        } else {
            println!("cargo:warning=windows/compilerconfig.cmake not found, skipping");
        }

        // Patch windows/buildspec.cmake to skip pre-built dep downloads (we use vcpkg)
        println!("cargo:warning=Step: patch windows/buildspec.cmake to skip downloads");
        let buildspec_win = obs_src.join("cmake/windows/buildspec.cmake");
        if buildspec_win.exists() {
            println!("cargo:warning=Replacing windows/buildspec.cmake with no-op");
            fs::write(&buildspec_win, "# PATCHED: no-op, deps provided by vcpkg\n")?;
        }

        // Windows SDK version: prefer CMAKE_SYSTEM_VERSION env var (set by workflow)
        let sdk_ver = env::var("CMAKE_SYSTEM_VERSION")
            .unwrap_or_else(|_| "10.0.20348.0".to_string());
        println!("cargo:warning=Using Windows SDK: {sdk_ver}");

        cmake.arg("-G").arg("Ninja");
        cmake.arg(format!("-DCMAKE_SYSTEM_VERSION={sdk_ver}"));
        cmake.arg(format!("-DCMAKE_INSTALL_PREFIX={}", install_prefix.display()));

        // vcpkg integration
        if let Ok(vcpkg_root) = env::var("VCPKG_ROOT") {
            let toolchain = PathBuf::from(&vcpkg_root)
                .join("scripts/buildsystems/vcpkg.cmake");
            println!("cargo:warning=Using vcpkg toolchain: {}", toolchain.display());
            cmake.arg(format!("-DCMAKE_TOOLCHAIN_FILE={}", toolchain.display()));
            cmake.arg("-DVCPKG_TARGET_TRIPLET=x64-windows-release");
        } else {
            println!("cargo:warning=VCPKG_ROOT not set, skipping vcpkg toolchain");
        }

        cmake
            .arg("-DENABLE_SCRIPTING=OFF")
            .arg("-DENABLE_VIRTUALCAM=OFF")
            .arg("-DENABLE_WIN_CRASH_HANDLER=OFF");

    } else {
        // --- Configure cmake flags (linux) ---
        println!("cargo:warning=Step: configure cmake flags (linux)");
        cmake
            .arg("-DENABLE_PIPEWIRE=ON")
            .arg("-DENABLE_WAYLAND=ON")
            .arg("-DENABLE_X11=ON");
    }

    // IMPORTANT LINE
    run(cmake.current_dir(&build_dir), "cmake configure")?;

    // 4) Build only libobs target
    let mut build_cmd = Command::new("cmake");
    build_cmd
        .arg("--build")
        .arg(".")
        .arg("--target").arg("libobs")
        .arg("--parallel")
        .arg(nproc())
        .current_dir(&build_dir);
    
    run(&mut build_cmd, "cmake build")?;

    // 5) Install to build dir: Release/core
    let install_dir = build_dir.join("install");
    fs::create_dir_all(&install_dir)?;
    let mut install_cmd = Command::new("cmake");
    install_cmd
        .arg("--install")
        .arg(".")
        .arg("--prefix")
        .arg(&install_dir)
        .current_dir(&build_dir);
    
    run(&mut install_cmd, "cmake install")?;

    // 6) Generate Rust FFI bindings
    let wrapper = manifest_dir.join("src/ffi/wrapper.h");
    let bindings = bindgen::Builder::default()
        .header(wrapper.to_string_lossy())
        .clang_arg(format!("-I{}", install_prefix.join("include").display()))
        .clang_arg(format!("-I{}", install_prefix.join("include/obs").display()))
        .clang_arg(format!("-I{}", obs_src.join("libobs").display()))
        .clang_arg(format!("-I{}", build_dir.display()))
        .clang_arg(format!("-I{}", build_dir.join("config").display()))
        .allowlist_function("obs_.*")
        .allowlist_function("base_.*")
        .allowlist_function("gs_.*")
        .allowlist_function("vsnprintf")
        .allowlist_type("obs_.*")
        .allowlist_type("gs_.*")
        .allowlist_type("__va_list_tag")
        .allowlist_var("OBS_.*")
        .allowlist_var("LOG_.*")
        .generate()
        .map_err(|_| "bindgen failed")?;

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let out_file = out_dir.join("libobs_bindings.rs");
    bindings.write_to_file(out_file)?;

    // 7) Link to installed libobs
    println!(
        "cargo:rustc-link-search=native={}",
        install_prefix.join("lib").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        install_prefix.join("lib64").display()
    );
    if cfg!(target_os = "windows") {
        // On Windows the import lib is obs.lib; no dl/pthread/m needed.
        println!("cargo:rustc-link-search=native={}", install_prefix.join("bin").display());
        println!("cargo:rustc-link-lib=dylib=obs");
    } else {
        println!("cargo:rustc-link-lib=dylib=obs");
        println!("cargo:rustc-link-lib=dylib=dl");
        println!("cargo:rustc-link-lib=dylib=pthread");
        println!("cargo:rustc-link-lib=dylib=m");
    }

    println!("cargo:rerun-if-changed={}", wrapper.display());

    Ok(())
}
