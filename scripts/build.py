# This builds the crate and moves around the libs to a folder called pxsb (pixelscript build)
from glob import glob
import os
import shutil
from pathlib import Path
import subprocess


# Config
CRATE_NAME = "pixelscript"
LIB_CRATES = ["mlua", CRATE_NAME]
SOURCE = "pxsb"
# TODO: Figure out how WASM will work since it needs to use other libs for it to work, gotta have to link something.
VALID_EXTENSIONS = ["lib", "a", "so", "dylib"]


def convert_path(path:str) -> str:
    """Convert a Windows path"""
    return path.replace('\\', '/')


def get_ext(path) -> str:
    return path.split('.')[-1]


def move(old):
    old = convert_path(old)
    ext = get_ext(old)
    file_name = old.split('.')[0].split('/')[-1]
    shutil.copy(old, f"{SOURCE}/{file_name}.{ext}")


def collect_libs(folder, rule="/**/*"):
    for ext in VALID_EXTENSIONS:
        libs = glob(f"{folder}{rule}.{ext}", recursive=True)
        for lib in libs:
            move(lib)


# Build in release mode
subprocess.call(["cargo", "build", "--release"])

Path(SOURCE).mkdir(exist_ok=True)
collect_libs("target/release", rule="/*")

build_dir = Path("target/release/build")

for path in os.listdir(build_dir):
    for lib in LIB_CRATES:
        if path.startswith(lib):
            full_path = f"{build_dir}/{path}"
            print(full_path)
            # Search through contents
            collect_libs(full_path)
