# This builds the crate and moves around the libs to a folder called pixel_script
# Disclaimer, this was written by Gemini. I have not yet gone through it to clean.

import glob
import os
import shutil
from pathlib import Path

# 1. Configuration
ALLOWED_CRATES = ["mlua", "rustpython"] # External dependencies
MAIN_CRATE_NAME = "pixel_script"         # Your actual project
VALID_EXTENSIONS = {".lib", ".a", ".so", ".dylib", ".wasm"}

def collect_libs():
    dist_dir = Path("pixel_script")
    dist_dir.mkdir(exist_ok=True)
    build_dir = Path("target/release/build")
    release_dir = Path("target/release")

    print(f"Searching for libraries in {build_dir}...")

    # --- Step 1: Collect Internal Dependencies (Strict Folder Check) ---
    if build_dir.exists():
        # Iterate through every folder in the build directory
        for folder in build_dir.iterdir():
            if not folder.is_dir():
                continue
            
            # Check if this folder starts with one of our allowed crate names
            # e.g., folder.name is "mlua-sys-928374..."
            matched_crate = next((c for c in ALLOWED_CRATES if folder.name.startswith(c)), None)
            
            if matched_crate:
                # Only search inside the /out/ directory of THIS specific crate
                pattern = os.path.join(folder, "out", "**", "*")
                for f in glob.glob(pattern, recursive=True):
                    path = Path(f)
                    if path.suffix.lower() in VALID_EXTENSIONS:
                        dest = dist_dir / path.name
                        shutil.copy2(path, dest)
                        print(f" -> Collected Dependency: {path.name} (from {matched_crate})")

    # --- Step 2: Collect Your Main Crate ---
    # We look directly in release/, NOT in the build/ subfolders to avoid the noise
    for ext in VALID_EXTENSIONS:
        main_lib = release_dir / f"{MAIN_CRATE_NAME}{ext}"
        if main_lib.exists():
            dest = dist_dir / main_lib.name
            shutil.copy2(main_lib, dest)
            print(f" -> Collected Main Crate: {main_lib.name}")

if __name__ == "__main__":
    collect_libs()