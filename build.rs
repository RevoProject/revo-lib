use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const OBS_COMMIT: &str = "32.1.0-rc1";

fn run(cmd: &mut Command, step: &str) -> Result<(), Box<dyn Error>> {
    let status = cmd.status()?;
    if !status.success() {
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

    let obs_root = manifest_dir.join("obs-libobs");
    let obs_src = obs_root.join("obs-studio");
    let build_dir = obs_src.join("build-headless");
    let install_prefix = build_dir.join("Release/core");

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
            .arg(OBS_COMMIT),
        "git fetch pinned commit",
    )?;
    run(
        Command::new("git")
            .arg("-C")
            .arg(&obs_src)
            .arg("checkout")
            .arg("--force")
            .arg(OBS_COMMIT),
        "git checkout pinned commit",
    )?;

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
    run(
        Command::new("cmake")
            .arg("..")
            .arg("-DCMAKE_BUILD_TYPE=Release")
            .arg("-DENABLE_UI=OFF")
            .arg("-DENABLE_FRONTEND=OFF")
            .arg("-DENABLE_WEBSOCKET=ON")
            .arg("-DENABLE_BROWSER=OFF")
            .arg("-DENABLE_AJA=OFF")
            .arg("-DENABLE_NVENC=OFF")
            .arg("-DENABLE_QSV11=OFF")
            .arg("-DENABLE_VST=OFF")
            .arg("-DENABLE_PIPEWIRE=ON")
            .arg("-DENABLE_NEW_MPEGTS_OUTPUT=OFF")
            .current_dir(&build_dir),
        "cmake configure",
    )?;

    // 4) Build
    run(
        Command::new("cmake")
            .arg("--build")
            .arg(".")
            .arg("-j")
            .arg(nproc())
            .current_dir(&build_dir),
        "cmake build",
    )?;

    // 5) Install to build dir: Release/core
    fs::create_dir_all(build_dir.join("Release/core"))?;
    run(
        Command::new("cmake")
            .arg("--install")
            .arg(".")
            .arg("--prefix")
            .arg(build_dir.join("Release/core"))
            .current_dir(&build_dir),
        "cmake install",
    )?;

    // 6) Generate Rust FFI bindings
    let wrapper = manifest_dir.join("src/ffi/wrapper.h");
    let bindings = bindgen::Builder::default()
        .header(wrapper.to_string_lossy())
        .clang_arg(format!("-I{}", install_prefix.join("include").display()))
        .clang_arg(format!("-I{}", install_prefix.join("include/obs").display()))
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
    println!("cargo:rustc-link-lib=dylib=obs");
    println!("cargo:rustc-link-lib=dylib=dl");
    println!("cargo:rustc-link-lib=dylib=pthread");
    println!("cargo:rustc-link-lib=dylib=m");

    println!("cargo:rerun-if-changed={}", wrapper.display());

    Ok(())
}
