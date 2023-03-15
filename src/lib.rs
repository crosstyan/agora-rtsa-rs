#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

/// This module contains the generated bindings from `bindgen`.
/// Just copy and paste from `bindings.rs` to here.
pub mod ffi;

// https://stackoverflow.com/questions/24145823/how-do-i-convert-a-c-string-into-a-rust-string-and-back-via-ffi
// https://doc.rust-lang.org/reference/items/extern-crates.htm

// https://stackoverflow.com/questions/66915951/rust-use-vs-mod
mod callbacks;
mod utils;
pub mod agoraRTC;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn try_get_version() {
        let v = agoraRTC::get_version();
        assert_eq!(v, "1.8.0");
    }
}
