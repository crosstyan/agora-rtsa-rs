extern crate bindgen;

use std::env;
use std::path::PathBuf;

// from https://rust-lang.github.io/rust-bindgen/tutorial-3.html
fn main() {
    let header = "agora_sdk/include/agora_rtc_api.h";
    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search=agora_sdk/lib/aarch64");

    // Tell cargo to tell rustc to link the system library
    println!("cargo:rustc-link-lib=agora-rtc-sdk");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=agora_sdk/include/agora_rtc_api.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(header)
        // https://rust-lang.github.io/rust-bindgen/nocopy.html
        .no_copy("log_config_t")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}