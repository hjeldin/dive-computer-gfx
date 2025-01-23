#[cfg(feature = "simulator")]
fn main() {
    // Tell cargo to tell rustc to link the system shared library
    // named `libfoo`.
    println!("cargo:rustc-link-search=native=/nix/store/6a4s1c1d3xcl0i9lacj23ziz8l27zn5b-SDL2-2.30.6/lib");
}

#[cfg(not(feature = "simulator"))]
fn main() {
    
}