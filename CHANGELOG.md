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