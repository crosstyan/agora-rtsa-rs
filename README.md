# Rust bindings for [Agora RTSA](https://docs.agora.io/cn/RTSA/downloads?platform=All%20Platforms)

See also [Agora RTSA C API Reference for Linux](https://docs.agora.io/cn/RTSA/API%20Reference/rtsa_c/index.html)

Link against `aarch64` by default, you may wish to modify `build.rs` to link your arch.

SDK library file is not included. You have to move it to the directory manually. Downlod the SDK from [here](https://docs.agora.io/cn/RTSA/downloads?platform=All%20Platforms).

Current support version is `1.9.0`

`ffi.rs` is the `bindgen` whose name was `bindings.rs`. 
