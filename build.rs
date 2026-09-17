extern crate cbindgen;

use std::path::PathBuf;
use std::{env, fs};

/// Read dir
fn read_dir(path: PathBuf) -> Vec<String> {
    let paths = fs::read_dir(path).unwrap();
    paths
        .map(|v| v.unwrap().path().to_str().unwrap().to_string())
        .collect()
}

/// Build the pixelscript.h C bindings
fn build_pixelscript_h() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let package_name = env::var("CARGO_PKG_NAME").unwrap();
    let output_file = PathBuf::from(&crate_dir).join(format!("{}.h", package_name));

    cbindgen::generate(crate_dir)
        .expect("Unable to generate bindings")
        .write_to_file(output_file);
}

#[cfg(feature = "lua")]
fn build_lua(target_os: &str, target_env: &str) {
    let mut build = cc::Build::new();
    build.warnings(false);

    // Add sources
    let paths = read_dir(PathBuf::from("libs/lua-5.5.0"));

    for file in paths {
        if !file.ends_with(".c") {
            continue;
        }
        if file.contains("lua.c") || file.contains("luac.c") {
            continue;
        }
        build.file(file);
    }
    build.file("libs/pxs_lua/pxs_lua.c");

    build.include("libs/lua-5.5.0");
    build.include("libs/pxs_lua");
    build.include("libs/pxs_utils");

    if target_env == "msvc" {
        build.static_crt(true);
        build.flag("/utf-8");
        build.std("c11");
    } else {
        build.flag("-O3");
        build.flag("-fPIC");
        build.std("c99");
    }

    if target_os == "linux" {
        build.define("LUA_USE_LINUX", None);
    }

    build.compile("lua");
}

/// Build PocketPy library
#[cfg(feature = "python")]
fn build_pocketpy(_target_os: &str, target_env: &str) {
    let mut build = cc::Build::new();
    build.warnings(false);

    // Add sorce
    build.file("libs/pocketpy/pocketpy.c");
    build.file("libs/pxs_python/pxs_python.c");
    // Add header location
    build.include("libs/pocketpy");
    build.include("libs/pxs_python");
    build.include("libs/pxs_utils");

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
        // Set NDEBUG macro for performance (https://pocketpy.dev/quick-start/#compile-flags)
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

#[cfg(feature="pxs_zip")]
/// Build pxs_zip
fn build_pxs_zip(_target_os: &str, target_env: &str) {
    let mut build = cc::Build::new();
    build.warnings(false);
    build.cpp(true);

    // Include pixelscript.h
    build.include("./");
    // Include zip dir
    build.include("core/pxs/zip");

    // Compile source
    build.file("core/pxs/zip/zip.cpp");

    if target_env == "msvc" {
        build.static_crt(true);
        build.flag("/EHsc");
    } else {
        build.flag("-fpermissive");
        build.flag("-include");
        build.flag("cstring");
    }

    build.std("c++17");
    build.compile("pxs_zip");
}

// #[cfg(feature="pxs_http")]
// /// Build pxs_http
// fn build_pxs_http(target_os: &str, _target_env: &str) {
//     // Specific impls
//     if target_os == "windows" {
//         // build.file("core/pxs/http/http_windows.cpp");
//     } else if target_os == "macos" || target_os == "ios" {
//         // build.file("core/pxs/http/http_apple.mm");
//         println!("cargo:rustc-link-lib=framework=Foundation");
//     } else if target_os == "linux" {
//         // build.file("core/pxs/http/http_linux.cpp");
//         println!("cargo:rustc-link-lib=curl");
//     }

//     // Compile source
//     // build.file("core/pxs/http/http.cpp");

//     // build.std("c++17");
//     // build.compile("pxs_http");
// }

/// Create PocketPy Rust bindings
#[cfg(feature = "python")]
fn build_pocketpy_bindings() {
    // This might be a problem when using GCC on windows.
    // I don't, but if anyone requires this please apply a solution if it does not work currently.

    let builder = bindgen::Builder::default()
        .header("libs/pocketpy/pocketpy.h")
        .header("libs/pxs_python/pxs_python.h")
        .clang_arg("-DPK_IS_PUBLIC_INCLUDE")
        .clang_arg("-Ilibs/pocketpy")
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .size_t_is_usize(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_function("py_.*")
        .allowlist_type("py_.*")
        .allowlist_var("py_.*")
        .allowlist_function("pxspython_.*")
        .allowlist_var("PXSPYTHON_.*");

    let bindings = builder
        .generate()
        .expect("Unable to build Pocketpy rust bindings");

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
    bindings
        .write_to_file(out_path.join("quickjsng_bindings.rs"))
        .expect("Couldn't write QuickJS-NG bindings!");
}

#[cfg(feature = "lua")]
fn build_lua_bindings() {
    let bindings = bindgen::Builder::default()
        .header("libs/lua-5.5.0/lua.h")
        .clang_args(vec![
            "-include",
            "libs/lua-5.5.0/lualib.h",
            "-include",
            "libs/lua-5.5.0/lauxlib.h",
            "-include",
            "libs/pxs_lua/pxs_lua.h",
            "-Ilibs/lua-5.5.0",
        ])
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .size_t_is_usize(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_function("lua_.*")
        .allowlist_function("luaL_.*")
        .allowlist_type("lua_.*")
        .allowlist_type("luaL_.*")
        .allowlist_var("LUA_.*")
        .allowlist_function("pxslua_.*")
        .generate()
        .expect("Could not generate Lua-5.5.0 bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("lua_bindings.rs"))
        .expect("Couldn't write Lua-5.5.0 bindings!");
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    build_pixelscript_h();
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    // Compile lua
    #[cfg(feature = "lua")]
    {
        build_lua(&target_os, &target_env);
        build_lua_bindings();
        println!("cargo:rerun-if-changed=libs/lua-5.5.0");
        println!("cargo:rerun-if-changed=libs/pxs_lua/pxs_lua.c");
        println!("cargo:rerun-if-changed=libs/pxs_lua/pxs_lua.h");
        println!("cargo:rerun-if-changed=libs/pxs_utils");
    }

    // Compile pocketpy
    #[cfg(feature = "python")]
    {
        build_pocketpy(&target_os, &target_env);
        build_pocketpy_bindings();
        println!("cargo:rerun-if-changed=libs/pocketpy/pocketpy.c");
        println!("cargo:rerun-if-changed=libs/pocketpy/pocketpy.h");
        println!("cargo:rerun-if-changed=libs/pxs_python");
        println!("cargo:rerun-if-changed=libs/pxs_utils");
    }

    // Compile quickjs-ng
    #[cfg(feature = "js")]
    {
        build_quickjsng(&target_os, &target_env);
        build_quickjsng_bindings();
        println!("cargo:rerun-if-changed=libs/quickjs-ng/quickjs-amalgam.c");
        println!("cargo:rerun-if-changed=libs/quickjs-ng/quickjs.h");
    }

    #[cfg(feature="pxs_zip")]
    {
        build_pxs_zip(&target_os, &target_env);
        println!("cargo:rerun-if-changed=core/pxs/zip");
    }

    // #[cfg(feature="pxs_http")]
    // {
    //     build_pxs_http(&target_os, &target_env);
    //     println!("cargo:rerun-if-changed=src/pxs_core/http");
    // }
}
