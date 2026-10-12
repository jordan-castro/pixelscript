# This builds the crate and moves around the libs to a folder called pxsb (pixelscript build)

from glob import glob
import os
import shutil
from pathlib import Path
import subprocess
import sys


# Config
CRATE_NAME = "pixelscript"
LIB_CRATES = [CRATE_NAME]
SOURCE = "pxsb"
VALID_EXTENSIONS = ["lib", "a", "so", "dylib", "wasm", "dll", "js"]
VALID_LANGS = ["python", "lua", "js"]
VALID_CORES = ["http", "zip", "mem", "json", "fs", "os", "shell", "api", "pxs"]
full_lib_size = 0


def convert_path(path:str) -> str:
    """Convert a Windows path"""
    return path.replace('\\', '/')


def get_ext(path) -> str:
    return path.split('.')[-1]


def move(old):
    global full_lib_size
    full_lib_size += os.path.getsize(old)
    old = convert_path(old)
    ext = get_ext(old)
    file_name = old.split('.')[0].split('/')[-1]
    # The wasm build produces pxs.js and pxs.wasm
    if file_name == 'pxs' and ext in ['wasm', 'js']:
        file_name = "pixelscript"
    shutil.copy(old, f"{SOURCE}/{file_name}.{ext}")


def collect_libs(folder, rule="/**/*"):
    for ext in VALID_EXTENSIONS:
        libs = glob(f"{folder}{rule}.{ext}", recursive=True)
        for lib in libs:
            move(lib)


# Get the args for target and features
argv = []
if len(sys.argv) > 0:
    argv = sys.argv[1:]

target = ""
rtarget = ""
features = []
defaults = True
debug = False
run_clear = False
use_zig = False
two_wasm = False
binary = False
langs = []
cores = []
android_target = ""
android_api = '36'

for arg in argv:
    if "target" in arg:
        # Split target
        rtarget = arg.split("=")[-1]
        target =  "--target=" + rtarget
    elif "features" in arg:
        # Split features
        features = arg.split("=")[-1].split(",")
    elif "defaults" in arg:
        if arg.split("=")[-1].lower() == 'n':
            defaults = False
    elif "langs=" in arg:
        langs = arg.split("=")[-1].split(",")
        for l in langs:
            if l not in VALID_LANGS:
                print(f"{l} is not a valid lang.")
                exit(1)
    elif "cores=" in arg:
        cores = arg.split("=")[-1].split(",")
        for c in cores:
            if c not in VALID_CORES:
                print(f"{c} is not a valid core.")
                exit(1)
    elif "debug" in arg:
        debug = True
    elif arg == "clear":
        run_clear = True
    elif arg == "zig":
        use_zig = True
    elif arg == '2wasm':
        two_wasm = True
    elif arg == 'bin':
        binary = True
    elif "android=" in arg:
        android_target = arg.split("=")[-1].strip()
    elif "android_api=" in arg:
        android_api = arg.split("=")[-1].strip()
    elif arg == "help":
        print("""PixelScript script/build.py usage
Arguments:
- target=<valid rust targets>
- features=<valid pxs feature comma delimited>
- langs=<valid pxs backends comma delimited>
- cores=<valid pxs cores, comma delimited>
- defaults=<n/y>
- android=<valid rust android target>
- android_api=<valid android api level>
- debug; a debug build
- clear; clear the cache
- help; print this message
- zig; use zig to build. This is best for cross platform. Requires `cargo-zigbuild`.
- 2wasm; comple pixelscript library for use in WASM.
- bin; compile the pxs binary.
""")
        exit(0)

build_mode = "release" if not debug else "debug"
build_flag = "--release" if not debug else ""
# Build in release mode
cmd = [
    "cargo", 
    "ndk" if len(android_target) > 0 else "",
    "zigbuild" if use_zig else "build", 
    build_flag, 
    "--lib" if not binary else "--bin pxs"
]

# override target when wasm is passed. 
if two_wasm:
    if not binary:
        print("Can not compile WASM without `bin` enabled.")
        exit(1)
    if use_zig:
        print("Can not compile WASM using zig.")
        exit(1)
    if len(android_target) > 0:
        print("Can not compile WASM and Android at same time.")
        exit(1)
    target = "--target=wasm32-unknown-emscripten"
    rtarget = "wasm32-unknown-emscripten"

# Override target when android is passed
if len(android_target) > 0:
    if binary:
        print("Can not compile Android with `bin` enabled.")
        exit(1)

    target = f"--target={android_target}"
    rtarget = android_target

    # Lets make sure that the ENV are set.
    if not 'ANDROID_NDK_HOME' in os.environ:
        print("ANDROID_NDK_HOME is required in env.")
        exit(1)

    os.environ['CARGO_NDK_ANDROID_PLATFORM'] = android_api
    os.environ['ANDROID_PLATFORM'] = android_api

# Grab target and features if passed
if target:
    cmd += [target]
if not defaults:
    cmd += ["--no-default-features"]
if len(langs) > 0:
    if defaults:
        print("Defaults need to be disabled to use language features. Disable using defaults=n")
        exit(1)
    features += langs
if len(cores) > 0:
    if defaults:
        print("Defaults need to be disabled to use core features. Disable using defaults=n")
    features += list(map(lambda c: f"pxs_{c}", cores))

if len(features) > 0:
    cmd += ["--features", f'"{','.join(features)}"']

if run_clear:
    # Run cargo clean before building
    subprocess.call(["cargo", "clean"])

print(" ".join(cmd))
os.system(" ".join(cmd))
# subprocess.call(cmd)

# Find build directory
path_to_build = f"target/{build_mode}/build"
path_to_release = f"target/{build_mode}"
if target:
    path_to_build = f"target/{rtarget}/{build_mode}/build"
    path_to_release = f"target/{rtarget}/{build_mode}"

# Create source
source = Path(SOURCE)
# If exists, clear it
if source.exists() and source.is_dir():
    shutil.rmtree(source)
source.mkdir(exist_ok=True)

# Collect the pixelscript lib
collect_libs(path_to_release, rule="/*")

build_dir = Path(path_to_build)

for path in os.listdir(build_dir):
    for lib in LIB_CRATES:
        if path.startswith(lib):
            full_path = f"{build_dir}/{path}"
            print(full_path)
            # Search through contents
            collect_libs(full_path)

# Replace pxs.wasm to pixelscript.wasm in pixelscript.js
if two_wasm:
    files = glob(f"{SOURCE}/*.js")
    if len(files) > 0:
        with open(files[0], "r+") as f:
            contents = f.read()
            contents = contents.replace("pxs.wasm", "pixelscript.wasm")
            f.seek(0)
            f.truncate(0)
            f.write(contents)


print(f"Full size of pixelscript: {full_lib_size // 1000000}mb")
