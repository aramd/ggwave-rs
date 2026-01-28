use std::env;
use std::path::PathBuf;

fn main() {
    let vendored = env::var_os("CARGO_FEATURE_VENDORED").is_some();
    let system = env::var_os("CARGO_FEATURE_SYSTEM").is_some();

    if !vendored && !system {
        panic!("Either the 'vendored' or 'system' feature must be enabled");
    }

    if system {
        if vendored {
            println!(
                "cargo:warning=Both 'vendored' and 'system' features enabled; using system library."
            );
        }
        pkg_config::Config::new()
            .probe("ggwave")
            .expect("Failed to find system ggwave via pkg-config");
        return;
    }

    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR missing"));
    let ggwave_dir = manifest_dir.join("vendor/ggwave");
    let src_dir = ggwave_dir.join("src");
    let include_dir = ggwave_dir.join("include");

    println!("cargo:rerun-if-changed={}", src_dir.join("ggwave.cpp").display());
    println!(
        "cargo:rerun-if-changed={}",
        include_dir.join("ggwave/ggwave.h").display()
    );
    println!("cargo:rerun-if-changed={}", src_dir.join("fft.h").display());
    println!(
        "cargo:rerun-if-changed={}",
        src_dir.join("reed-solomon/rs.hpp").display()
    );

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .file(src_dir.join("ggwave.cpp"))
        .include(&src_dir)
        .include(&include_dir)
        .flag_if_supported("-std=c++11")
        .flag_if_supported("-fPIC")
        .warnings(false)
        .compile("ggwave");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_os == "macos" || target_os == "ios" {
        println!("cargo:rustc-link-lib=c++");
    } else if target_env == "gnu" {
        println!("cargo:rustc-link-lib=stdc++");
    }
}

