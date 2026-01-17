extern crate cbindgen;

use std::env;
use std::path::PathBuf;

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

/// Build PocketPy library
#[cfg(feature = "python")]
fn build_pocketpy() {
    let mut build = cc::Build::new();

    // Add sorce
    build.file("libs/pocketpy/pocketpy.c");
    // Add header location
    build.include("libs/pocketpy");

    // Set c11
    build.std("c11");

    // When MSVC, gotta set some stuff
    let builder = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if builder == "msvc" {
        build.flag("/utf-8");
        build.flag("/experimental:c11atomics");
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

/// Create PocketPy Rust bindings
#[cfg(feature = "python")]
fn build_pocketpy_bindings() {
    let bindings = bindgen::Builder::default()
        .header("libs/pocketpy/pocketpy.h")
        .clang_arg("-Ilibs/pocketpy")
        .size_t_is_usize(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_function("py_.*")
        .allowlist_type("py_.*")
        .allowlist_var("py_.*")
        .generate()
        .expect("Unable to build Pocketpy rust bindings");

    // Write bindings
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("pocketpy_bindings.rs"))
        .expect("Couldn't write bindings!");    
} 

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    build_pixelscript_h();
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");

    #[cfg(feature = "python")] 
    {
        build_pocketpy();
        build_pocketpy_bindings();
        println!("cargo:rerun-if-changed=libs/pocketpy/pocketpy.c");
        println!("cargo:rerun-if-changed=libs/pocketpy/pocketpy.h");
    }
}