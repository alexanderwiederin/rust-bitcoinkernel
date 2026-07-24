use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

const BUILD_CONFIG: &str = "RelWithDebInfo";
const MIN_ANDROID_API: u32 = 24;

const BASE_CMAKE_FLAGS: &[&str] = &[
    "-DBUILD_KERNEL_LIB=ON",
    "-DBUILD_TESTS=OFF",
    "-DBUILD_BENCH=OFF",
    "-DBUILD_KERNEL_TEST=OFF",
    "-DBUILD_TX=OFF",
    "-DBUILD_WALLET_TOOL=OFF",
    "-DENABLE_WALLET=OFF",
    "-DENABLE_EXTERNAL_SIGNER=OFF",
    "-DBUILD_UTIL=OFF",
    "-DBUILD_BITCOIN_BIN=OFF",
    "-DBUILD_DAEMON=OFF",
    "-DBUILD_UTIL_CHAINSTATE=OFF",
    "-DBUILD_CLI=OFF",
    "-DBUILD_FUZZ_BINARY=OFF",
    "-DBUILD_SHARED_LIBS=OFF",
    "-DCMAKE_INSTALL_LIBDIR=lib",
    "-DENABLE_IPC=OFF",
];

trait Target {
    // Extra cmake flags needed to cross-compile for this target
    fn cmake_args(&self) -> Vec<String> {
        Vec::new()
    }

    // Link directives for the C++ runtime and any system libraries
    // Each string is emitted verbatim as `cargo:{directive}`.
    fn link_directives(&self) -> Vec<String>;

    // Environment variables that should force a rebuild when they change
    fn rerun_env(&self) -> &[&str] {
        &[]
    }
}

fn target_from_env() -> Box<dyn Target> {
    match &*env::var("CARGO_CFG_TARGET_OS").unwrap() {
        "windows" => Box::new(Windows),
        "macos" => Box::new(MacOs),
        "android" => Box::new(Android::from_env()),
        _ => Box::new(Unix),
    }
}

struct Windows;

impl Target for Windows {
    fn link_directives(&self) -> Vec<String> {
        vec![
            "rustc-link-lib=bcrypt".to_string(),
            "rustc-link-lib=shell32".to_string(),
        ]
    }
}

struct MacOs;

impl Target for MacOs {
    fn link_directives(&self) -> Vec<String> {
        let compiler = cc::Build::new().get_compiler();

        if compiler.is_like_clang() {
            vec!["rustc-link-lib=dylib=c++".to_string()]
        } else if compiler.is_like_gnu() {
            vec!["rustc-link-lib=dylib=stdc++".to_string()]
        } else {
            Vec::new()
        }
    }
}

struct Unix;

impl Target for Unix {
    fn link_directives(&self) -> Vec<String> {
        let compiler = cc::Build::new().get_compiler();

        if compiler.is_like_clang() || compiler.is_like_gnu() {
            vec!["rustc-link-lib=dylib=stdc++".to_string()]
        } else {
            Vec::new()
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Abi {
    Arm64,
    ArmV7,
    X86_64,
}

impl Abi {
    fn from_env() -> Self {
        match &*env::var("CARGO_CFG_TARGET_ARCH").unwrap() {
            "aarch64" => Self::Arm64,
            "arm" => Self::ArmV7,
            "x86_64" => Self::X86_64,
            arch => panic!("Unsupported Android ABI: {arch}"),
        }
    }

    // Value for -DANDROID_ABI.
    fn cmake_name(self) -> &'static str {
        match self {
            Self::Arm64 => "arm64-v8a",
            Self::ArmV7 => "armeabi-v7a",
            Self::X86_64 => "x86_64",
        }
    }

    // NDK sysroot lib directory triple. Differs from the Rust triple for
    // armv7: Rust says `armv7-linux-androideabi`, the NDK says
    // `arm-linux-androideabi`.
    fn ndk_triple(&self) -> &'static str {
        match self {
            Self::Arm64 => "aarch64-linux-android",
            Self::ArmV7 => "arm-linux-androideabi",
            Self::X86_64 => "x86_64-linux-android",
        }
    }
}

struct Android {
    ndk: PathBuf,
    abi: Abi,
    api_level: u32,
}

impl Android {
    fn from_env() -> Self {
        let ndk = env::var("ANDROID_NDK_HOME")
            .map(PathBuf::from)
            .expect("Android target detected but ANDROID_NDK_HOME is not set");

        // API level 24+ is required because Bitcoin Core uses getifaddrs
        // which was introduced in Android API 24 (Nougat).
        //
        // This can be overridden to a higher level by setting ANDROID_API_LEVEL.
        let api_level = match env::var("ANDROID_API_LEVEL") {
            Ok(level) => {
                let n: u32 = level.parse().expect("ANDROID_API_LEVEL must be a number");
                assert!(
                    n >= MIN_ANDROID_API,
                    "ANDROID_API_LEVEL must be {MIN_ANDROID_API}+"
                );
                n
            }
            Err(_) => MIN_ANDROID_API,
        };

        Self {
            ndk,
            abi: Abi::from_env(),
            api_level,
        }
    }

    fn toolchain_file(&self) -> PathBuf {
        self.ndk
            .join("build")
            .join("cmake")
            .join("android.toolchain.cmake")
    }

    fn sysroot_lib_dir(&self) -> PathBuf {
        let host_tag = match env::consts::OS {
            "macos" => "darwin-x86_64",
            "linux" => "linux-x86_64",
            os => panic!("unsupported build host for Android cross-compilation: {os}"),
        };
        self.ndk
            .join("toolchains/llvm/prebuilt")
            .join(host_tag)
            .join("sysroot/usr/lib")
            .join(self.abi.ndk_triple())
    }
}

impl Target for Android {
    fn cmake_args(&self) -> Vec<String> {
        vec![
            format!("-DCMAKE_TOOLCHAIN_FILE={}", self.toolchain_file().display()),
            format!("-DANDROID_ABI={}", self.abi.cmake_name()),
            format!("-DANDROID_PLATFORM=android-{}", self.api_level),
            "-DCMAKE_SYSTEM_NAME=Android".to_string(),
            format!("-DCMAKE_ANDROID_NDK={}", self.ndk.display()),
            // The NDK toolchain sets CMAKE_FIND_ROOT_PATH_MODE_PACKAGE to ONLY,
            // which prevents cmake from finding host packages via
            // CMAKE_PREFIX_PATH. Relax it for headers so Boost can be located
            "-DCMAKE_FIND_ROOT_PATH_MODE_PACKAGE=BOTH".to_string(),
        ]
    }

    fn link_directives(&self) -> Vec<String> {
        // The NDK ships libc++_static.a and libc++abi.a in the per-architecture
        // sysroot directory, not the API-level subdirectory.
        let mut out = vec![
            format!(
                "rustc-link-search=native={}",
                self.sysroot_lib_dir().display()
            ),
            "rustc-link-lib=static=c++_static".to_string(),
            "rustc-link-lib=static=c++abi".to_string(),
        ];

        // The pre-compiled libcompiler_builtins for armv7-linux-androideabi ships
        // ARM EABI helper symbols tagged with @@LIBC_N (e.g. __aeabi_memcpy@@LIBC_N).
        // lld errors when linking a shared library or executable because the LIBC_N
        // version node is not defined in any version script. --exclude-libs,ALL
        // marks every symbol pulled from static archives as local, suppressing it.
        if self.abi == Abi::ArmV7 {
            out.push("rustc-link-arg=-Wl,--exclude-libs,ALL".to_string());
        }

        out
    }

    fn rerun_env(&self) -> &[&str] {
        // Without these, changing either variable between builds won't trigger a
        // rebuild, so we could silently end up with a stale binary built against
        // the wrong NDK or API level.
        &["ANDROID_NDK_HOME", "ANDROID_API_LEVEL"]
    }
}

fn run(cmd: &mut Command, what: &str) {
    let status = cmd
        .status()
        .unwrap_or_else(|e| panic!("failed to spawn {what}: {e}"));

    assert!(status.success(), "{what} failed with {status}");
}

fn main() {
    let target = target_from_env();

    for var in target.rerun_env() {
        println!("cargo:rerun-if-env-changed={var}");
    }

    let lib_dir = build_kernel(target.as_ref());

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=bitcoinkernel");

    for directive in target.link_directives() {
        println!("cargo:{directive}")
    }
}

fn build_kernel(target: &dyn Target) -> PathBuf {
    let bitcoin_dir = Path::new("bitcoin");
    let out_dir = env::var("OUT_DIR").unwrap();
    let build_dir = Path::new(&out_dir).join("bitcoin");
    let install_dir = Path::new(&out_dir).join("install");

    // Iterate through all files in the Bitcoin Core submodule directory
    println!("cargo:rerun-if-changed={}", bitcoin_dir.display());

    cmake_configure(target, bitcoin_dir, &build_dir, &install_dir);
    cmake_build(&build_dir);
    cmake_install(&build_dir);

    resolve_lib_dir(&install_dir)
}

fn cmake_flags(target: &dyn Target, install_dir: &Path) -> Vec<String> {
    let mut flags = vec![format!("-DCMAKE_BUILD_TYPE={BUILD_CONFIG}")];
    flags.extend(BASE_CMAKE_FLAGS.iter().map(|s| s.to_string()));
    flags.extend(target.cmake_args());
    flags.push(format!("-DCMAKE_INSTALL_PREFIX={}", install_dir.display()));

    flags
}

fn cmake_configure(target: &dyn Target, source_dir: &Path, build_dir: &Path, install_dir: &Path) {
    run(
        Command::new("cmake")
            .arg("-B")
            .arg(build_dir)
            .arg("-S")
            .arg(source_dir)
            .args(cmake_flags(target, install_dir)),
        "cmake configure",
    );
}

fn cmake_build(build_dir: &Path) {
    let num_jobs = env::var("NUM_JOBS")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get() as u32)
                .unwrap_or(1) // Default to 1 if not set
        });

    run(
        Command::new("cmake")
            .arg("--build")
            .arg(build_dir)
            .arg("--config")
            .arg(BUILD_CONFIG)
            .arg(format!("--parallel={num_jobs}")),
        "cmake build",
    );
}

fn cmake_install(build_dir: &Path) {
    run(
        Command::new("cmake")
            .arg("--install")
            .arg(build_dir)
            .arg("--config")
            .arg(BUILD_CONFIG),
        "cmake install",
    );
}

fn resolve_lib_dir(install_dir: &Path) -> PathBuf {
    // Multi-config generators (MSVC, Xcode) nest the config name under lib/.
    let multi_config = install_dir.join("lib").join(BUILD_CONFIG);
    if multi_config.exists() {
        multi_config
    } else {
        install_dir.join("lib")
    }
}
