#[cfg(feature = "simulator")]
fn main() {
    // Let the linker find SDL2 when it is not on the default search path.
    // The Nix dev shell sets SDL2_LIB_DIR; other setups can set it or rely on system paths.
    println!("cargo:rerun-if-env-changed=SDL2_LIB_DIR");
    if let Ok(dir) = std::env::var("SDL2_LIB_DIR") {
        println!("cargo:rustc-link-search=native={dir}");
    }
}

#[cfg(not(feature = "simulator"))]
fn main() {}
