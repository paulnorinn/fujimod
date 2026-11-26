use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};

/// QEMU's configure, which Unicorn's CMake runs, rejects source and build
/// directories containing spaces or colons and word-splits the compiler path.
///
/// A Windows drive prefix is not such a colon: the MSYS shell that runs
/// configure reports the same directory as `/c/...`, so only the rest of the
/// path decides.
fn breaks_configure(path: &Path) -> bool {
    let Some(text) = path.to_str() else {
        return true;
    };
    let text = match text.as_bytes() {
        [drive, b':', ..] if drive.is_ascii_alphabetic() => &text[2..],
        _ => text,
    };
    text.contains(|c: char| c.is_whitespace() || c == ':')
}

#[cfg(unix)]
fn link(target: &Path, alias: &Path) {
    let _ = std::fs::remove_file(alias);
    std::os::unix::fs::symlink(target, alias).expect("link Unicorn build alias");
}

#[cfg(windows)]
fn link(target: &Path, alias: &Path) {
    let _ = std::fs::remove_dir(alias);
    let _ = std::fs::remove_file(alias);
    if target.is_dir() {
        std::os::windows::fs::symlink_dir(target, alias)
    } else {
        std::os::windows::fs::symlink_file(target, alias)
    }
    .expect("link Unicorn build alias");
}

/// QEMU's C sources need a compiler with C11 atomics and POSIX declarations;
/// MSVC's `cl.exe` provides neither, so a Windows host must select clang-cl.
#[cfg(windows)]
fn require_clang_cl(compiler: PathBuf) -> PathBuf {
    let name = compiler
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    assert!(
        name.starts_with("clang-cl"),
        "Unicorn needs clang-cl on Windows (selected {}); set CC and CXX to the full path of clang-cl.exe",
        compiler.display()
    );
    compiler
}

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("emscripten") {
        let directory = std::env::var("CAMERA_UNICORN_STATIC_DIR")
            .expect("CAMERA_UNICORN_STATIC_DIR must name the Emscripten Unicorn build");
        println!("cargo:rustc-link-search=native={directory}");
        for name in ["unicorn", "arm-softmmu", "unicorn-common"] {
            println!("cargo:rustc-link-lib=static={name}");
            println!("cargo:rerun-if-changed={directory}/lib{name}.a");
        }
        println!("cargo:rerun-if-env-changed=CAMERA_UNICORN_STATIC_DIR");
        return;
    }
    let source = std::env::current_dir().unwrap().join("../vendor/unicorn");
    if !source.join("CMakeLists.txt").is_file() {
        panic!("the pinned Unicorn source submodule is not initialized");
    }

    let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let compiler = cc::Build::new().get_compiler().path().to_path_buf();
    let compiler = std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
        .map(|directory| directory.join(&compiler))
        .find(|path| path.is_file())
        .unwrap_or(compiler);
    #[cfg(windows)]
    let compiler = require_clang_cl(compiler);
    let mut config;
    if [&source, &out, &compiler].into_iter().any(|path| breaks_configure(path)) {
        let mut hasher = DefaultHasher::new();
        out.hash(&mut hasher);
        let alias = std::env::temp_dir()
            .join(format!("fujimod-unicorn-{:016x}", hasher.finish()));
        assert!(!breaks_configure(&alias), "no space-free directory for the Unicorn build");
        std::fs::create_dir_all(&alias).expect("create Unicorn build alias");
        link(&source, &alias.join("source"));
        link(&compiler, &alias.join("cc"));
        config = cmake::Config::new(alias.join("source"));
        config.out_dir(&alias).define("CMAKE_C_COMPILER", alias.join("cc"));
        // The system may clear the temporary directory between builds.
        println!("cargo:rerun-if-changed={}", alias.join("build/libunicorn.a").display());
    } else {
        config = cmake::Config::new(&source);
    }
    #[cfg(windows)]
    {
        // QEMU's fujimod perf-map hook calls getpid() without including
        // <process.h>, which clang-cl rejects as an implicit declaration.
        config.generator("Ninja");
        config.cflag("/FIprocess.h");
    }
    config
        // A compiler-path change makes CMake discard its cache and rerun
        // configuration without our -D options. Start fresh so the selected
        // architectures survive toolchain changes in a reused Cargo directory.
        .configure_arg("--fresh")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("UNICORN_LEGACY_STATIC_ARCHIVE", "ON")
        .define("UNICORN_ARCH", "arm;aarch64")
        .define("UNICORN_TCG_INTERPRETER", "OFF")
        .define("UNICORN_BUILD_TESTS", "OFF")
        .define("UNICORN_INSTALL", "OFF")
        .define("UNICORN_FUZZ", "OFF")
        .build_target("unicorn_archive");
    let output = config.build();
    let cache = std::fs::read_to_string(output.join("build/CMakeCache.txt"))
        .expect("read configured Unicorn architecture");
    assert!(
        cache.lines().any(|line| {
            line.starts_with("UNICORN_ARCH:")
                && line
                    .split_once('=')
                    .is_some_and(|(_, value)| value == "arm;aarch64")
        }),
        "Unicorn must be configured for ARM and AArch64"
    );

    println!(
        "cargo:rustc-link-search=native={}",
        output.join("build").display()
    );
    println!("cargo:rustc-link-lib=static=unicorn");
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_env != "msvc" {
        println!("cargo:rustc-link-lib=m");
        println!("cargo:rustc-link-lib=pthread");
    }
    // GCC lowers Unicorn's 16-byte atomics to libatomic calls on Linux.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux") {
        println!("cargo:rustc-link-lib=atomic");
    }
    println!("cargo:rerun-if-changed=../vendor/unicorn");
}
