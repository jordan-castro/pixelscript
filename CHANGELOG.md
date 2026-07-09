# Changelog (since 0.6.0)

## 0.6.0
- Removed runtime dependencies (anyhow, mlua).
- Use C lua bindings via ffi.
- Each thread gets its own language state (including pixelscript state).
- Remove (os, io, debug) from lua runtime.

## 0.6.1
- Remove lua module loaders (2,3,4)
    - 2 path searcher
    - 3 c searcher
    - 4 all searcher
- Added VFS loader for lua files.

## 0.6.2
- Update `pxs_tostring` to catch `pxs_VarNull` without routing it.
- Cleaned `scripts/build.py`
- Created `pixelscript_cpp.hpp` header.
- Updated `pxs_LoadFileFn` to return `pxs_VarT`

## 0.6.3
- Fixed `LUA_*` being added to `pixelscript.h`
- Created `pxs_utils` in libs
- Created `pxs_python` in libs.
- Added `pxspython_importfile` and `pxspython_import` for safe rust interop.
- Include `pixelscript_m.h` in `pixelscript_cpp.hpp`
- Free the error message in `consume_error`.
- Add `HWrapper` to `pixelscript_cpp.hpp` wrapper.
- Added `PXSVariant` to `pixelscript_cpp.hpp` wrapper.
- Added internal `etffi` dep.
- Added `pxs_getidx`.
- Added `pxs_evalnamed`.
- Added `require` function to JS backend for commonJS support.
- Added `js_commonjs` feature.
- Wrapped `pxs_json` in feature tag in JS backend.

## 0.6.4
- Added `pxs_newtype` to combat UB in `pxs_HostObject` retrival.
- Added `pxs_gettype` that retrieves `pxs_HostObject` and only returns if the type matches.
- Change luas VFS module loader to be raw string. This lets the host handle `/` or `.` as they like.

## 0.6.5
- Added `pxs_addfuncs` method to add the same function with different names. This is useful for adding tostring methods.
- Added benchmarks.
- Added `scripts/bench.py` to run and display benchmarks.
- Made it so that `pxs_getstring` won't panic.
- renamed `core` to `pxs_core`.
- added `pxs_arg` to be shorthand for `pxs_listget(args, i - 1)`
- Added `pxs_getrt` to be shorthand for `pxs_listget(args, 0)`
- Automatically init `pxs_mem` if enabled.
- Added memory helpers:
    - `pxs_newbytes` which creates a list of u8s from a void* of size and element size.
    - `pxs_copybytes` copies a list of items as good as possible (this is unsafe) because a list can have a float and bool
        anywyas, it copies the bytes into a void*
    - `pxs_varsize` get the size (in bytes) of a `pxs_VarT` 
    - `pxs_copystring` copy a `pxs_String` into a *char without allocations.
    - `pxs_smart_getstring` get a *char from a `pxs_VarT` IF IT is not a string it will call `pxs_tostring` and handle the memory automatically.
    - `pxs_smart_copystring` does the same as `pxs_smart_getstring` but does not allocate the final *char.
- Added `yoyo` core modules. (behind feature flag.)
    - `yoyo`
    - `yoyo.os`
    - `yoyo.net`
    - `yoyo.shell`
    - `yoyo.zip`
    - `yoyo.fs`
- Added `test_bytes.rs`
- Added `test_str.rs`
- Added `test_yoyo.rs`