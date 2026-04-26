extern crate cbindgen;

use std::env;
use std::path::PathBuf;
use std::process::Command;

/// Build the pixelscript.h C bindings
fn build_pixelscript_h() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let package_name = env::var("CARGO_PKG_NAME").unwrap();
    let output_file = PathBuf::from(&crate_dir)
        .join(format!("{}.h", package_name));

    cbindgen::generate(crate_dir)
        .expect("Unable to generate bindings")
        .write_to_file(output_file);
}

/// Build PH7 library
#[cfg(feature = "php")]
fn build_ph7() {
    let mut build = cc::Build::new();
    build.warnings(false);

    // Add source
    build.file("libs/ph7/ph7.c");
    // Add header location
    build.include("libs/ph7");

    let builder = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    if builder == "msvc" {
        build.flag("/Ox");
        build.flag("/fp:fast");
        // Compile as a static lib
        build.static_crt(true);
    } else {
        // GCC/Clang
        build.flag("-Wunused");
        build.flag("-Ofast");
    }

    build.compile("ph7");
}

/// Build bindings for ph7
#[cfg(feature = "php")]
fn build_ph7_bindings() {
    let mut builder = bindgen::Builder::default()
        .header("libs/ph7/ph7.h")
        .clang_arg("-libs/ph7")
        .default_enum_style(bindgen::EnumVariation::Rust { non_exhaustive: false })
        .size_t_is_usize(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    // add GNU libs
    for arg in find_gnu_include_path() {
        builder = builder.clang_arg(arg);
    }

    let bindings = builder.generate().expect("Unable to build Pocketpy rust bindings");
 
    // Write bindings
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("ph7_bindings.rs"))
        .expect("Couldn't write ph7 bindings!");    
}

/// Build PocketPy library
#[cfg(feature = "python")]
fn build_pocketpy(_target_os: &str, target_env: &str) {
    let mut build = cc::Build::new();
    build.warnings(false);

    // Add sorce
    build.file("libs/pocketpy/pocketpy.c");
    // Add header location
    build.include("libs/pocketpy");

    // Set c11
    build.std("c11");

    // When MSVC, gotta set some stuff
    if target_env == "msvc" {
        build.flag("/utf-8");
        build.flag("/experimental:c11atomics");
        // Compile as a static lib
        build.static_crt(true);
    } else {
        build.flag("-O3");
        build.flag("-fPIC");
    }

    // Check if release or debug mode
    let target_mode = std::env::var("PROFILE").unwrap_or_default();
    if target_mode == "release" {
        // Override NDEBUG macro for performance
        build.define("NDEBUG", None);
    }

    // Remove PK_ENABLE_THREADS since PixelScript is single threaded (in theory at least)
    build.define("PK_ENABLE_THREADS", "0");

    // Now we can compile pocketpy.
    build.compile("pocketpy");
}

/// Build QuickJS-NG Library
#[cfg(feature = "js")]
fn build_quickjsng(_target_os: &str, target_env: &str) {
    let mut build = cc::Build::new();
    build.warnings(false);
    build.file("libs/quickjs-ng/quickjs-amalgam.c");
    build.include("libs/quickjs-ng");
    build.std("c11");
    build.define("_GNU_SOURCE", None);

    if target_env == "msvc" {
        build.flag("/experimental:c11atomics");
        build.static_crt(true);
    } else {
        build.flag("-fPIC");
        build.flag("-funsigned-char");
        build.flag("-fno-exceptions");
        build.flag("-fno-asynchronous-unwind-tables");
    }

    build.compile("quickjs");
}

// TODO: remove this
fn _find_gnu_include_path() -> Vec<String> {
    // Get current os
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        return vec![];
    }

    // Check for current toolchain
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_env != "gnu" {
        return vec![];
    }
    // Ok here we need to find out headers...
    let output = Command::new("gcc")
        .arg("-v")
        .arg("-E")
        .arg("-")
        .stdin(std::process::Stdio::null())
        .output()
        .ok();
    let mut includes = vec![];

    if let Some(output) = output {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let mut scanning = false;

        for line in stderr.lines() {
            if line.contains("#include <...> search starts here") {
                scanning = true;
                continue;
            }
            if line.contains("End of search list") {
                break;
            }

            if scanning {
                let path = line.trim();
                if !path.is_empty() {
                    includes.push(format!("-isystem{}", path.replace("\\", "/")));
                }
            }            
        }
    }

    includes
}

/// Create PocketPy Rust bindings
#[cfg(feature = "python")]
fn build_pocketpy_bindings() {
    // If using gcc on windows, we might need to find the gcc include paths
    // let include_paths = 

    let builder = bindgen::Builder::default()
        .header("libs/pocketpy/pocketpy.h")
        .clang_arg("-DPK_IS_PUBLIC_INCLUDE")
        .clang_arg("-Ilibs/pocketpy")
        .default_enum_style(bindgen::EnumVariation::Rust { non_exhaustive: false })
        .size_t_is_usize(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_function("py_.*")
        .allowlist_type("py_.*")
        .allowlist_var("py_.*");

    // for arg in find_gnu_include_path() {
    //     builder = builder.clang_arg(arg);
    // }

    let bindings = builder.generate().expect("Unable to build Pocketpy rust bindings");
 
    // Write bindings
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("pocketpy_bindings.rs"))
        .expect("Couldn't write PocketPy bindings!");    
} 

/// Create QuickJS-NG Rust bindings
#[cfg(feature = "js")]
fn build_quickjsng_bindings() {
    let bindings = bindgen::Builder::default()
        .header("libs/quickjs-ng/quickjs.h")
        .allowlist_function("js_.*")
        .allowlist_function("JS_.*")
        .allowlist_type("js_.*")
        .allowlist_type("JS_.*")
        .allowlist_var("JS_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Could not generate QuickJS-NG bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("quickjsng_bindings.rs")).expect("Couldn't write QuickJS-NG bindings!");
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    build_pixelscript_h();
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    // Compile pocketpy
    #[cfg(feature = "python")] 
    {
        build_pocketpy(&target_os, &target_env);
        build_pocketpy_bindings();
        println!("cargo:rerun-if-changed=libs/pocketpy/pocketpy.c");
        println!("cargo:rerun-if-changed=libs/pocketpy/pocketpy.h");
    }

    // Compile quickjs-ng
    #[cfg(feature = "js")]
    {
        build_quickjsng(&target_os, &target_env);
        build_quickjsng_bindings();
        println!("cargo:rerun-if-changed=libs/quickjs-ng/quickjs-amalgam.c");
        println!("cargo:rerun-if-changed=libs/quickjs-ng/quickjs.h");
    }

    // Compile PH7
    #[cfg(feature = "php")]
    {
        build_ph7();
        build_ph7_bindings();
        println!("cargo:rerun-if-changed=libs/ph7/ph7.c");
        println!("cargo:rerun-if-changed=libs/ph7/ph7.h");
    }
}