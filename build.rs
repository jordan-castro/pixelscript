extern crate cbindgen;

use std::path::PathBuf;
use std::{env, fs};

/// Adds emsdk includes. This is only required for emscripten backend.
macro_rules! add_emsdk_include {
    ($bindings:expr, $target:expr) => {{
        if $target == "emscripten" {
            let emsdk = env::var("EMSDK").expect("EMSDK is not setup.");
            let emsdk_sysroot = format!("{emsdk}/upstream/emscripten/cache/sysroot");
            let emsdk_include_path = format!("{emsdk_sysroot}/include");

            $bindings = $bindings.clang_arg(format!("--sysroot={emsdk_sysroot}"));
            $bindings = $bindings.clang_arg(format!("-I{emsdk_include_path}"));
            $bindings = $bindings.clang_arg("-D__EMSCRIPTEN__");
            $bindings = $bindings.clang_arg("-target").clang_arg("wasm32-unknown-emscripten");
        }
    }};
}

/// Adds builder flags for emscripten backend
macro_rules! add_emscripten_flags {
    ($builder:expr, $target:expr) => {{
        if $target == "emscripten" {
            $builder.flag("-fwasm-exceptions")
                    .flag("-sSUPPORT_LONGJMP=wasm")
                    .flag("-sPTHREAD_POOL_SIZE=4")
                    .flag("-sALLOW_MEMORY_GROWTH=1")
                    .flag("-sALLOW_TABLE_GROWTH=1");
        }
    }};
}

/// Read dir
fn read_dir(path: PathBuf) -> Vec<String> {
    let paths = fs::read_dir(path).unwrap();
    paths
        .map(|v| v.unwrap().path().to_str().unwrap().to_string())
        .collect()
}

/// Build the emscripten flags for pixelscript
fn build_emscripten_flags() {
    println!("cargo:rustc-link-arg=-sEXPORTED_RUNTIME_METHODS=['ccall','cwrap','UTF8ToString']");
    println!("cargo:rustc-link-arg=-sSUPPORT_LONGJMP=wasm");
    println!("cargo:rustc-link-arg=-fwasm-exceptions");
    println!("cargo:rustc-link-arg=-sENVIRONMENT=web");
    println!("cargo:rustc-link-arg=-sALLOW_MEMORY_GROWTH=1");
    println!("cargo:rustc-link-arg=-sASSERTIONS=1");
    println!("cargo:rustc-link-arg=-sPTHREAD_POOL_SIZE=4");
    println!("cargo:rustc-link-arg=-sSTACK_SIZE=5242880");
    println!("cargo:rustc-link-arg=-sSTACK_OVERFLOW_CHECK=2");
    println!("cargo:rustc-link-arg=-sALLOW_TABLE_GROWTH=1");
}

/// Build the pixelscript.h C bindings
fn build_pixelscript_h(target_os: &str) {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let package_name = env::var("CARGO_PKG_NAME").unwrap();
    let output_file = PathBuf::from(&crate_dir).join(format!("{}.h", package_name));

    cbindgen::generate(crate_dir)
        .expect("Unable to generate bindings")
        .write_to_file(output_file);

    if target_os == "emscripten" {
        build_emscripten_flags();
    }
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

    add_emscripten_flags!(build, target_os);

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

    add_emscripten_flags!(build, _target_os);

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

    add_emscripten_flags!(build, _target_os);

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

// #[cfg(feature="pxs_zip")]
// /// Build pxs_zip
// fn build_pxs_zip(_target_os: &str, target_env: &str, taget_arch: &str) {
//     let mut build = cc::Build::new();
//     build.warnings(false);
//     build.cpp(true);

//     // Include pixelscript.h
//     build.include("./");
//     // Include zip dir
//     build.include("core/pxs/zip");

//     // Compile source
//     build.file("core/pxs/zip/zip.cpp");

//     if target_env == "msvc" {
//         build.static_crt(true);
//         build.flag("/EHsc");
//     } else {
//         build.flag("-fpermissive");
//         build.flag("-include");
//         build.flag("cstring");
//     }

//     build.std("c++17");
//     build.compile("pxs_zip");
// }

/// Create QuickJS-NG Rust bindings
#[cfg(feature = "js")]
fn build_quickjsng_bindings(target_os: &str) {
    let mut bindings = bindgen::Builder::default()
        .header("libs/quickjs-ng/quickjs.h")
        .allowlist_function("js_.*")
        .allowlist_function("JS_.*")
        .allowlist_type("js_.*")
        .allowlist_type("JS_.*")
        .allowlist_var("JS_.*")
        .layout_tests(false)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));
        // .generate()
        // .expect("Could not generate QuickJS-NG bindings");

    add_emsdk_include!(bindings, target_os);
    let bindings = bindings.generate().expect("Could not generate QuickJS-NG bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("quickjsng_bindings.rs"))
        .expect("Couldn't write QuickJS-NG bindings!");
}

#[cfg(feature = "lua")]
fn build_lua_bindings(target_os: &str) {
    let mut bindings = bindgen::Builder::default()
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
        .layout_tests(false)
        .allowlist_function("pxslua_.*");

    add_emsdk_include!(bindings, target_os);
    let bindings = bindings.generate().expect("Could not generate Lua-5.5.0 bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("lua_bindings.rs"))
        .expect("Couldn't write Lua-5.5.0 bindings!");
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    // let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    build_pixelscript_h(&target_os);

    // Compile lua
    #[cfg(feature = "lua")]
    {
        build_lua(&target_os, &target_env);
        build_lua_bindings(&target_os);
        println!("cargo:rerun-if-changed=libs/lua-5.5.0");
        println!("cargo:rerun-if-changed=libs/pxs_lua/pxs_lua.c");
        println!("cargo:rerun-if-changed=libs/pxs_lua/pxs_lua.h");
        println!("cargo:rerun-if-changed=libs/pxs_utils");
    }

    // Compile pocketpy
    #[cfg(feature = "python")]
    {
        build_pocketpy(&target_os, &target_env);
        println!("cargo:rerun-if-changed=libs/pocketpy/pocketpy.c");
        println!("cargo:rerun-if-changed=libs/pocketpy/pocketpy.h");
        println!("cargo:rerun-if-changed=libs/pxs_python");
        println!("cargo:rerun-if-changed=libs/pxs_utils");
    }

    // Compile quickjs-ng
    #[cfg(feature = "js")]
    {
        build_quickjsng(&target_os, &target_env);
        build_quickjsng_bindings(&target_os);
        println!("cargo:rerun-if-changed=libs/quickjs-ng/quickjs-amalgam.c");
        println!("cargo:rerun-if-changed=libs/quickjs-ng/quickjs.h");
    }

    // #[cfg(feature="pxs_zip")]
    // {
    //     build_pxs_zip(&target_os, &target_env, &target_arch);
    //     println!("cargo:rerun-if-changed=core/pxs/zip");
    // }

    // #[cfg(feature="pxs_http")]
    // {
    //     build_pxs_http(&target_os, &target_env);
    //     println!("cargo:rerun-if-changed=src/pxs_core/http");
    // }
}
