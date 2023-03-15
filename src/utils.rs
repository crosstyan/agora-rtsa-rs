use super::ffi::*;
use std::ffi::{CString, CStr};
pub type ErrorCode = i32;

pub trait ToCString {
    fn to_c_string(&self) -> Result<CString, std::ffi::NulError>;
}
impl ToCString for &str {
    fn to_c_string(&self) -> Result<CString, std::ffi::NulError> {
        CString::new(&**self)
    }
}

impl ToCString for String {
    fn to_c_string(&self) -> Result<CString, std::ffi::NulError> {
        self.as_str().to_c_string()
    }
}


pub fn err_2_result(code: ErrorCode) -> Result<(), ErrorCode> {
    match code {
        0 => Result::Ok(()),
        _ => Result::Err(code),
    }
}

pub fn err_2_reason(code: ErrorCode) -> String {
    unsafe {
        let pReason = agora_rtc_err_2_str(code);
        CStr::from_ptr(pReason).to_str().unwrap().to_owned()
    }
}
