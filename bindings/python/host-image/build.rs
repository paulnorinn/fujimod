fn main() {
    build_jpeg(std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("emscripten"));
    println!("cargo:rerun-if-changed=build.rs");
}
fn build_jpeg(wasm: bool) {
    let mut jpeg = cc::Build::new();
    jpeg.file("src/jpeg_scanlines.c").warnings(true).pic(true);
    if wasm {
        jpeg.flag("-sMAIN_MODULE=2")
            .flag("-sUSE_LIBJPEG=1")
            .flag("-fwasm-exceptions")
            .flag("-sSUPPORT_LONGJMP=wasm");
    } else {
        for path in pkg_config::Config::new()
            .cargo_metadata(false)
            .probe("libjpeg")
            .expect("libjpeg development library")
            .include_paths
        {
            jpeg.include(path);
        }
    }
    jpeg.compile("fujimod-jpeg-scanlines");
    if wasm {
        let archive = std::process::Command::new("emcc")
            .args([
                "-sMAIN_MODULE=2",
                "-sUSE_LIBJPEG=1",
                "--print-file-name=libjpeg.a",
            ])
            .output()
            .expect("Emscripten libjpeg archive");
        assert!(archive.status.success());
        let archive = std::path::PathBuf::from(String::from_utf8(archive.stdout).unwrap().trim());
        println!(
            "cargo:rustc-link-search=native={}",
            archive
                .parent()
                .expect("libjpeg archive directory")
                .display()
        );
        println!("cargo:rustc-link-lib=static=jpeg");
    } else {
        link_jpeg();
    }
    println!("cargo:rerun-if-changed=src/jpeg_scanlines.c");
}

/// Link libjpeg.
///
/// Python searches only an extension's own directory for its dependencies, so a
/// dynamic libjpeg on Windows would have to be copied beside every installed
/// extension. A static archive the probe can find keeps it self-contained;
/// otherwise the probe's own dynamic link is used.
fn link_jpeg() {
    let library = pkg_config::Config::new()
        .cargo_metadata(false)
        .probe("libjpeg")
        .expect("libjpeg development library");
    let archives = ["jpeg-static.lib", "libjpeg-static.a", "libjpeg.a"];
    let static_archive = cfg!(windows)
        && library
            .link_paths
            .iter()
            .any(|directory| archives.iter().any(|name| directory.join(name).is_file()));
    if static_archive {
        for directory in &library.link_paths {
            println!("cargo:rustc-link-search=native={}", directory.display());
        }
        println!("cargo:rustc-link-lib=static=jpeg-static");
        return;
    }
    pkg_config::Config::new()
        .probe("libjpeg")
        .expect("libjpeg development library");
}
