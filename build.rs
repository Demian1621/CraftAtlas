use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn find_windres() -> Option<PathBuf> {
    if let Ok(path_var) = env::var("PATH") {
        for dir in env::split_paths(&path_var) {
            let candidate = dir.join("windres.exe");
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    if let Ok(profile) = env::var("USERPROFILE") {
        let fallback = PathBuf::from(profile).join(".gemini/w64devkit/bin/windres.exe");
        if fallback.exists() {
            return Some(fallback);
        }
    }
    None
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/app.rc");
    println!("cargo:rerun-if-changed=assets/app.ico");

    // Compile Windows resource (app icon) dynamically via PATH or w64devkit
    #[cfg(target_os = "windows")]
    {
        if Path::new("assets/app.rc").exists() {
            if let Some(windres) = find_windres() {
                let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
                let res_out = out_dir.join("app.res");
                let status = Command::new(&windres)
                    .args([
                        "assets/app.rc",
                        "-O",
                        "coff",
                        "-o",
                        res_out.to_str().expect("invalid OUT_DIR path"),
                    ])
                    .status();
                if let Ok(st) = status {
                    if st.success() {
                        println!("cargo:rustc-link-arg={}", res_out.display());
                    } else {
                        println!("cargo:warning=windres failed with exit status: {}", st);
                    }
                }
            } else {
                println!("cargo:warning=windres.exe not found; building without embedded resource icon");
            }
        }
    }

    // Compile Cubiomes C library + FFI bridge
    let cubiomes_dir = Path::new("third_party/cubiomes");
    let c_sources = [
        "noise.c",
        "biomes.c",
        "layers.c",
        "biomenoise.c",
        "generator.c",
        "finders.c",
        "util.c",
        "quadbase.c",
        "bridge.c",
    ];
    for file in &c_sources {
        println!("cargo:rerun-if-changed=third_party/cubiomes/{}", file);
    }
    println!("cargo:rerun-if-changed=third_party/cubiomes/bridge.h");

    let mut build = cc::Build::new();
    build.include(cubiomes_dir);
    for file in &c_sources {
        build.file(cubiomes_dir.join(file));
    }
    build
        .flag("-O3")
        .flag("-fwrapv")
        .warnings(false);

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        build.define("_WIN32", None);
    } else if target_os == "linux" {
        println!("cargo:rustc-link-lib=pthread");
        println!("cargo:rustc-link-lib=m");
    }

    build.compile("cubiomes");
}
