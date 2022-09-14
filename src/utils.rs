use std::ffi::{CString};

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