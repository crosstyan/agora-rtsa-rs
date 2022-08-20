#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// https://stackoverflow.com/questions/24145823/how-do-i-convert-a-c-string-into-a-rust-string-and-back-via-ffi
// https://doc.rust-lang.org/reference/items/extern-crates.htm

pub mod agoraRTC {
    use super::*;
    use num_derive::FromPrimitive;
    use num_enum::IntoPrimitive;
    use std::ffi::CStr;
    pub enum ReturnCode {
        Ok,
        GeneralError(String),
    }

    #[derive(Copy, Clone, FromPrimitive, IntoPrimitive)]
    #[repr(u32)]
    pub enum AreaCode {
        DEFAULT = 0x00000000,
        CN = 0x00000001,
        NA = 0x00000002,
        EU = 0x00000004,
        AS = 0x00000008,
        JP = 0x00000010,
        IN = 0x00000020,
        OC = 0x00000040,
        SA = 0x00000080,
        AF = 0x00000100,
        KR = 0x00000200,
        OVS = 0xFFFFFFFE,
        GLOB = (0xFFFFFFFF),
    }

    #[derive(Copy, Clone, FromPrimitive, IntoPrimitive)]
    #[repr(u32)]
    pub enum LogLevel {
        DEFAULT = 0, // the same as RTC_LOG_NOTICE
        EMERG,       // system is unusable
        ALERT,       // action must be taken immediately
        CRIT,        // critical conditions
        ERROR,       // error conditions
        WARNING,     // warning conditions
        NOTICE,      // normal but significant condition, default level
        INFO,        // informational
        DEBUG,       // debug-level messages
    }

    #[derive(Clone)]
    pub struct RtcServiceOption {
        area_code: AreaCode,
        product_id: String,
        log_cfg: LogConfig,
        license_value: String,
    }

    #[derive(Clone)]
    pub struct LogConfig {
        log_disable: bool,
        log_disable_desensitize: bool,
        log_level: LogLevel,
        log_path: String,
    }

    impl From<LogConfig> for log_config_t {
        fn from(config: LogConfig) -> Self {
            log_config_t {
                log_disable: config.log_disable,
                log_disable_desensitize: config.log_disable_desensitize,
                log_level: config.log_level.into(),
                log_path: config.log_path.as_ptr(),
            }
        }
    }

    impl From<RtcServiceOption> for rtc_service_option_t {
        fn from(opt: RtcServiceOption) -> Self {
            rtc_service_option_t {
                area_code: opt.area_code.into(),
                product_id: opt
                    .product_id
                    .as_bytes()
                    .try_into()
                    .expect("Product ID is too long!"),
                log_cfg: opt.log_cfg.into(),
                license_value: opt
                    .license_value
                    .as_bytes()
                    .try_into()
                    .expect("License Value is too long!"),
            }
        }
    }

    /// Get SDK version
    /// ```
    /// extern crate agora_rtsa_rs;
    /// let v = agora_rtsa_rs::agoraRTC::get_version();
    /// assert_eq!(v, "1.8.0");
    /// ```
    pub fn get_version() -> String {
        unsafe {
            let pVersion = agora_rtc_get_version();
            let version = CStr::from_ptr(pVersion);
            version.to_str().unwrap().to_owned()
        }
    }

    /// verify license without credential
    pub fn license_verify(certificate_str: &String) -> ReturnCode {
        unsafe {
            let code = agora_rtc_license_verify(
                certificate_str.as_ptr(),
                certificate_str.len().try_into().unwrap(),
                std::ptr::null(),
                0,
            );
            let reason: String = if code != 0 {
                let pVersion = agora_rtc_err_2_str(code);
                CStr::from_ptr(pVersion).to_str().unwrap().to_owned()
            } else {
                "".to_owned() // I should set it to undefined. There's some overhead.
            };
            // TODO: add more ReturnCode and avoid GeneralError
            match code {
                0 => ReturnCode::Ok,
                _ => ReturnCode::GeneralError(reason),
            }
        }
    }

    // TODO: build a interface
    // let a = agora_rtc_event_handler_t {
    //     on_join_channel_success: todo!(),
    //     on_connection_lost: todo!(),
    //     on_rejoin_channel_success: todo!(),
    //     on_error: todo!(),
    //     on_user_joined: todo!(),
    //     on_user_offline: todo!(),
    //     on_user_mute_audio: todo!(),
    //     on_user_mute_video: todo!(),
    //     on_audio_data: todo!(),
    //     on_mixed_audio_data: todo!(),
    //     on_video_data: todo!(),
    //     on_target_bitrate_changed: todo!(),
    //     on_key_frame_gen_req: todo!(),
    //     on_token_privilege_will_expire: todo!(),
    // };
    /// init
    // https://stackoverflow.com/questions/70840454/passing-a-safe-rust-function-pointer-to-c
    // https://adventures.michaelfbryan.com/posts/rust-closures-in-ffi/
    pub fn init(app_id: &String, opt: &RtcServiceOption, handlers: agora_rtc_event_handler_t) {
        // this should keeps living during the programming running (heap allocation?)
        let mut opt_t: rtc_service_option_t = opt.clone().into();
        let p_handler = &handlers as * const agora_rtc_event_handler_t;
        unsafe {
            agora_rtc_init(
                app_id.as_ptr(),
                p_handler,
                std::ptr::addr_of_mut!(opt_t),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn try_get_version() {
        let v = agoraRTC::get_version();
        assert_eq!(v, "1.8.0");
    }
}
