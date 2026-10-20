#[allow(unused)]
use pixelscript::*;

// Main for wasm, do nothing
#[cfg(target_arch = "wasm32")]
fn main() {}

// Main for none wasm, do lots of stuff.
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    
}