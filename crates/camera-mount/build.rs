use std::{env, ffi::OsString, fs, path::PathBuf, process::Command};
fn run(command: &mut Command) {
    let result = command.output().expect("run payload tool");
    assert!(
        result.status.success(),
        "{command:?}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
}
/// The command that runs `tool`.
///
/// On Windows a bare name runs through `cmd`, so a `.cmd` shim early on PATH
/// (mise, conda) resolves exactly as it does in a shell; a name with a path
/// separator runs directly.
fn tool(tool: &OsString) -> Command {
    #[cfg(windows)]
    {
        let text = tool.to_string_lossy();
        let shell = !text.is_empty()
            && (text.ends_with(".cmd")
                || text.ends_with(".bat")
                || (!text.contains('/') && !text.contains('\\')));
        if shell {
            let mut command = Command::new("cmd");
            command.arg("/c").arg(tool);
            return command;
        }
    }
    Command::new(tool)
}
/// Candidate spellings of `name`: on Windows the bare name first, so a `.cmd`
/// shim resolves, then every PATH directory with and without `.exe`.
fn candidates(name: &str) -> Vec<OsString> {
    let mut found = Vec::new();
    if cfg!(windows) {
        found.push(OsString::from(name));
    }
    let suffixes: &[&str] = if cfg!(windows) {
        &["", ".exe", ".cmd", ".bat"]
    } else {
        &[""]
    };
    for directory in env::split_paths(&env::var_os("PATH").unwrap_or_default()) {
        for suffix in suffixes {
            found.push(directory.join(format!("{name}{suffix}")).into());
        }
    }
    found
}
fn find(name: &str, works: impl Fn(&OsString) -> bool, message: &str) -> OsString {
    candidates(name)
        .into_iter()
        .find(|candidate| works(candidate))
        .unwrap_or_else(|| panic!("{message}"))
}
/// The clang that compiles the ARM payloads: `CLANG`, else Apple clang on
/// macOS, else the first `clang` on PATH whose registered targets include ARM.
/// A PATH can put a WebAssembly-only clang (Emscripten's) first.
fn arm_clang() -> OsString {
    if let Some(clang) = env::var_os("CLANG").filter(|clang| !clang.is_empty()) {
        return clang;
    }
    if cfg!(target_os = "macos") {
        return "/usr/bin/clang".into();
    }
    println!("cargo:rerun-if-env-changed=PATH");
    find(
        "clang",
        |clang| {
            tool(clang)
                .arg("-print-targets")
                .output()
                .is_ok_and(|output| {
                    String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .any(|line| line.trim_start().starts_with("arm "))
                })
        },
        "no clang on PATH targets ARM; set CLANG",
    )
}
/// The linker that turns the payload objects into raw images: `LD_LLD`, else
/// the first `ld.lld` on PATH that runs.
fn arm_linker() -> OsString {
    if let Some(linker) = env::var_os("LD_LLD").filter(|linker| !linker.is_empty()) {
        return linker;
    }
    println!("cargo:rerun-if-env-changed=PATH");
    find(
        "ld.lld",
        |linker| {
            tool(linker)
                .arg("--version")
                .output()
                .is_ok_and(|output| output.status.success())
        },
        "no ld.lld on PATH runs; set LD_LLD",
    )
}

fn main() {
    println!("cargo:rerun-if-env-changed=CLANG");
    println!("cargo:rerun-if-env-changed=LD_LLD");
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let script = out.join("payload.ld");
    fs::write(&script,"SECTIONS { . = 0; .text : { *(.text.entry) *(.text*) } .rodata : { *(.rodata*) } .data : { *(.data*) *(.bss*) } /DISCARD/ : { *(.ARM.exidx*) *(.ARM.extab*) *(.comment*) *(.note*) *(.eh_frame*) } }").unwrap();
    let clang = arm_clang();
    let linker = arm_linker();
    for name in ["call", "card"] {
        let source = format!("payload/{name}.c");
        println!("cargo:rerun-if-changed={source}");
        let object = out.join(format!("{name}.o"));
        let image = out.join(format!("{name}.bin"));
        run(tool(&clang)
            .args([
                "--target=armv7-none-eabi",
                "-mcpu=cortex-a9",
                "-marm",
                "-mfloat-abi=soft",
                "-Oz",
                "-fpie",
                "-fvisibility=hidden",
                "-ffreestanding",
                "-fno-builtin",
                "-fno-stack-protector",
                "-c",
                &source,
                "-o",
            ])
            .arg(&object));
        run(tool(&linker)
            .args(["-m", "armelf", "-Bsymbolic", "--oformat=binary", "-T"])
            .arg(&script)
            .arg(&object)
            .arg("-o")
            .arg(&image));
    }
}
