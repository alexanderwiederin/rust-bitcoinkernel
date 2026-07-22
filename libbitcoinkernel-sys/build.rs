use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let is_android = target_os == "android";

    let ndk = if is_android {
        Some(
            env::var("ANDROID_NDK_HOME")
                .expect("Android target detected but ANDROID_NDK_HOME is not set"),
        )
    } else {
        None
    };

    let bitcoin_dir = Path::new("bitcoin");
    let out_dir = env::var("OUT_DIR").unwrap();
    let build_dir = Path::new(&out_dir).join("bitcoin");
    let install_dir = Path::new(&out_dir).join("install");

    println!("{} {}", bitcoin_dir.display(), build_dir.display());

    // Iterate through all files in the Bitcoin Core submodule directory
    println!("cargo:rerun-if-changed={}", bitcoin_dir.display());

    let build_config = "RelWithDebInfo";

    let mut cmake_configure = Command::new("cmake");
    cmake_configure
        .arg("-B")
        .arg(&build_dir)
        .arg("-S")
        .arg(bitcoin_dir)
        .arg(format!("-DCMAKE_BUILD_TYPE={build_config}"))
        .arg("-DBUILD_KERNEL_LIB=ON")
        .arg("-DBUILD_TESTS=OFF")
        .arg("-DBUILD_BENCH=OFF")
        .arg("-DBUILD_KERNEL_TEST=OFF")
        .arg("-DBUILD_TX=OFF")
        .arg("-DBUILD_WALLET_TOOL=OFF")
        .arg("-DENABLE_WALLET=OFF")
        .arg("-DENABLE_EXTERNAL_SIGNER=OFF")
        .arg("-DBUILD_UTIL=OFF")
        .arg("-DBUILD_BITCOIN_BIN=OFF")
        .arg("-DBUILD_DAEMON=OFF")
        .arg("-DBUILD_UTIL_CHAINSTATE=OFF")
        .arg("-DBUILD_CLI=OFF")
        .arg("-DBUILD_FUZZ_BINARY=OFF")
        .arg("-DBUILD_SHARED_LIBS=OFF")
        .arg("-DCMAKE_INSTALL_LIBDIR=lib")
        .arg("-DENABLE_IPC=OFF")
        .arg(format!("-DCMAKE_INSTALL_PREFIX={}", install_dir.display()));

    if is_android {
        let ndk = ndk.as_deref().unwrap();
        let toolchain_file = format!("{ndk}/build/cmake/android.toolchain.cmake");

        // Rust target triple -> NDK ABI name.
        let abi = match env::var("TARGET").unwrap() {
            t if t.contains("aarch64") => "arm64-v8a",
            t if t.contains("armv7") => "armeabi-v7a",
            t if t.contains("x86_64") => "x86_64",
            target => panic!("Unsupported Android ABI: {target}"),
        };

        // API level 24+ is required because Bitcoin Core uses getifaddrs
        // which was introduced in Android API 24 (Nougat).
        //
        // This can be overridden to a higher level by setting ANDROID_API_LEVEL.
        let api_level = match env::var("ANDROID_API_LEVEL") {
            Ok(level) => {
                let n: u32 = level.parse().expect("ANDROID_API_LEVEL must be a number");
                assert!(n >= 24, "ANDROID_API_LEVEL must be 24+");
                level
            }
            _ => "24".to_string(),
        };

        cmake_configure
            .arg(format!("-DCMAKE_TOOLCHAIN_FILE={toolchain_file}"))
            .arg(format!("-DANDROID_ABI={abi}"))
            .arg(format!("-DANDROID_PLATFORM=android-{api_level}"))
            .arg("-DCMAKE_SYSTEM_NAME=Android")
            .arg(format!("-DCMAKE_ANDROID_NDK={ndk}"))
            // The Android NDK toolchain sets CMAKE_FIND_ROOT_PATH_MODE_PACKAGE
            // to ONLY, which prevents cmake from finding host packages via
            // CMAKE_PREFIX_PATH. Override it so Boost headers can be located.
            .arg("-DCMAKE_FIND_ROOT_PATH_MODE_PACKAGE=BOTH");
    }

    cmake_configure
        .status()
        .expect("cmake should be installed and available in PATH");

    let num_jobs = env::var("NUM_JOBS")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(1); // Default to 1 if not set

    Command::new("cmake")
        .arg("--build")
        .arg(&build_dir)
        .arg("--config")
        .arg(build_config)
        .arg(format!("--parallel={num_jobs}"))
        .status()
        .unwrap();

    Command::new("cmake")
        .arg("--install")
        .arg(&build_dir)
        .arg("--config")
        .arg(build_config)
        .status()
        .unwrap();

    // Check if the build system used a multi-config generator
    let lib_dir = if install_dir.join("lib").join(build_config).exists() {
        install_dir.join("lib").join(build_config)
    } else {
        install_dir.join("lib")
    };

    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    println!("cargo:rustc-link-lib=static=bitcoinkernel");

    let compiler = cc::Build::new().get_compiler();

    if target_os == "windows" {
        println!("cargo:rustc-link-lib=bcrypt");
        println!("cargo:rustc-link-lib=shell32");
    } else if is_android {
        // Without these, changing either variable between builds won't trigger a rebuild,
        // so we could silently end up with a stale binary built against the wrong NDK or API level.
        println!("cargo:rerun-if-env-changed=ANDROID_NDK_HOME");
        println!("cargo:rerun-if-env-changed=ANDROID_API_LEVEL");
        // Android NDK ships libc++_static.a and libc++abi.a in the
        // per-architecture sysroot directory (not the API-level subdirectory).
        let ndk = ndk.as_deref().unwrap();

        // Rust target triple -> NDK sysroot lib directory triple.
        // armv7 differs: Rust says `armv7-linux-androideabi`, NDK says `arm-linux-androideabi`.
        let ndk_triple = env::var("TARGET")
            .map(|target| {
                if target.starts_with("armv7") {
                    "arm-linux-androideabi".into()
                } else {
                    target
                }
            })
            .unwrap();

        let host_tag = match std::env::consts::OS {
            "macos" => "darwin-x86_64",
            "linux" => "linux-x86_64",
            os => panic!("unsupported build host for Android cross-compilation: {os}"),
        };

        let ndk_lib_dir =
            format!("{ndk}/toolchains/llvm/prebuilt/{host_tag}/sysroot/usr/lib/{ndk_triple}");
        println!("cargo:rustc-link-search=native={ndk_lib_dir}");
        println!("cargo:rustc-link-lib=static=c++_static");
        println!("cargo:rustc-link-lib=static=c++abi");

        // The pre-compiled libcompiler_builtins for armv7-linux-androideabi
        // ships ARM EABI helper symbols tagged with @@LIBC_N (e.g.
        // __aeabi_memcpy@@LIBC_N).  When lld links a shared library or
        // executable it errors because the LIBC_N version node is not
        // defined in any version script.  --exclude-libs,ALL marks every
        // symbol pulled from static archives as local, which suppresses
        // the version-node error.
        if ndk_triple == "arm-linux-androideabi" {
            println!("cargo:rustc-link-arg=-Wl,--exclude-libs,ALL");
        }
    } else if compiler.is_like_clang() {
        if target_os == "macos" {
            println!("cargo:rustc-link-lib=dylib=c++");
        } else {
            println!("cargo:rustc-link-lib=dylib=stdc++");
        }
    } else if compiler.is_like_gnu() {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    }
}
