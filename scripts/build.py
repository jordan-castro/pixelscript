# This builds the crate and moves around the libs to a folder called pxsb (pixelscript build)
from glob import glob
import os
import shutil
from pathlib import Path
import subprocess
import sys


# Config
CRATE_NAME = "pixelscript"
LIB_CRATES = ["mlua", CRATE_NAME]
SOURCE = "pxsb"
# TODO: Figure out how WASM will work since it needs to use other libs for it to work, gotta have to link something.
VALID_EXTENSIONS = ["lib", "a", "so", "dylib"]
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
features = ["--no-default-features"]

for arg in argv:
    if "target" in arg:
        # Split target
        rtarget = arg.split("=")[-1]
        target =  "--target=" + rtarget
    elif "features" in arg:
        # Split features
        features_ = arg.split("=")[-1]
        features += ["--features", f'{features_}']

# Build in release mode
cmd = ["cargo", "build", "--release"]
# Grab target and features if passed
if target:
    cmd += [target]
if len(features) > 1:
    cmd += features
print(" ".join(cmd))
subprocess.call(cmd)

# Find build directory
path_to_build = "target/release/build"
path_to_release = "target/release"
if target:
    path_to_build = f"target/{rtarget}/release/build"
    path_to_release = f"target/{rtarget}/release"

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

print(f"Full size of pixelscript: {full_lib_size // 1000000}mb")