#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// https://stackoverflow.com/questions/24145823/how-do-i-convert-a-c-string-into-a-rust-string-and-back-via-ffi
// https://doc.rust-lang.org/reference/items/extern-crates.htm

pub mod agoraRTC{
    use super::*;
    use std::ffi::CStr;
    /// Get SDK version
    /// ```
    /// extern crate agora_rtsa_rs;
    /// let v = agora_rtsa_rs::agoraRTC::get_version();
    /// assert_eq!(v, "1.8.0");
    /// ```
    pub fn get_version() -> String{
        unsafe{
            let pVersion = agora_rtc_get_version();
            let version = CStr::from_ptr(pVersion);
            version.to_str().unwrap().to_owned()
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn try_get_version(){
        let v = agoraRTC::get_version();
        assert_eq!(v, "1.8.0");
    }
}

