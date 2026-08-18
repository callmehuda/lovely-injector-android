//! Build script for `dobby-sys`.
//!
//! Dobby is a C++ inline-hook library. We build it as a static library via
//! CMake, then link it into the lovely `cdylib`.
//!
//! Cross-compilation: `cmake-rs` relies on `CC_<target>` / `CXX_<target>`
//! env vars to pick the right compiler, but it does NOT automatically set
//! `CMAKE_SYSTEM_NAME`. Without `CMAKE_SYSTEM_NAME=Android`, CMake's
//! `project()` call detects the host system (Linux x86_64 on the CI runner)
//! and bakes the host's target triple into the resulting object files —
//! which then fail to link against the actual Android target with cryptic
//! "incompatible with armelf_linux_eabi" errors.

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target = std::env::var("TARGET").unwrap_or_default();

    let mut cfg = cmake::Config::new("dobby");

    match target_os.as_str() {
        "android" => {
            cfg.define("CMAKE_SYSTEM_NAME", "Android");
            // When using the NDK's android.toolchain.cmake, the toolchain
            // sets CMAKE_ANDROID_ARCH_ABI internally from ANDROID_ABI, and
            // CMakeDetermineSystem.cmake validates that CMAKE_SYSTEM_PROCESSOR
            // matches the inferred arch. Setting CMAKE_SYSTEM_PROCESSOR
            // ourselves causes "CMAKE_ANDROID_ARCH_ABI='armeabi-v7a' and
            // CMAKE_SYSTEM_PROCESSOR='arm' is not a valid combination" on
            // armv7. So we DON'T set CMAKE_SYSTEM_PROCESSOR here — we let
            // the toolchain infer it from ANDROID_ABI.

            if let Ok(ndk_home) = std::env::var("ANDROID_NDK_HOME") {
                let toolchain = format!("{ndk_home}/build/cmake/android.toolchain.cmake");
                if std::path::Path::new(&toolchain).exists() {
                    cfg.define("CMAKE_TOOLCHAIN_FILE", &toolchain);
                }
                cfg.define("ANDROID_NDK", &ndk_home);
                cfg.define("ANDROID_USE_LEGACY_TOOLCHAIN_FILE", "OFF");
                cfg.define("ANDROID_PLATFORM", "android-24");
                // ANDROID_ABI drives both the toolchain's arch detection AND
                // Dobby's CMakeLists.txt backend selection.
                let abi = match target.as_str() {
                    "aarch64-linux-android" => "arm64-v8a",
                    "armv7-linux-androideabi" => "armeabi-v7a",
                    "x86_64-linux-android" => "x86_64",
                    "i686-linux-android" => "x86",
                    _ => "",
                };
                if !abi.is_empty() {
                    cfg.define("ANDROID_ABI", abi);
                }
            }
        }
        "linux" => {
            cfg.define("CMAKE_SYSTEM_NAME", "Linux");
        }
        "macos" => {
            cfg.define("CMAKE_SYSTEM_NAME", "Darwin");
            cfg.define("CMAKE_OSX_DEPLOYMENT_TARGET", "11.0");
        }
        "ios" => {
            cfg.define("CMAKE_SYSTEM_NAME", "iOS");
            cfg.define("CMAKE_OSX_DEPLOYMENT_TARGET", "11.0");
        }
        "windows" => {
            cfg.define("CMAKE_SYSTEM_NAME", "Windows");
        }
        _ => {}
    }

    let dst = cfg.build_target("dobby_static").build();

    let lib_path = dst.join("build");
    println!("cargo:warning=lib_path={}", lib_path.display());
    println!("cargo:rustc-link-search=native={}", lib_path.display());
    println!("cargo:rustc-link-lib=static=dobby");

    match target_os.as_str() {
        "macos" | "ios" => println!("cargo:rustc-link-lib=dylib=c++"),
        _ => println!("cargo:rustc-link-lib=dylib=stdc++"),
    }

    println!("cargo:rerun-if-env-changed=ANDROID_NDK_HOME");
    println!("cargo:rerun-if-env-changed=TARGET");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_OS");
}
