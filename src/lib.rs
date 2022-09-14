#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod C {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

// https://stackoverflow.com/questions/24145823/how-do-i-convert-a-c-string-into-a-rust-string-and-back-via-ffi
// https://doc.rust-lang.org/reference/items/extern-crates.htm

// https://stackoverflow.com/questions/66915951/rust-use-vs-mod
mod callbacks;
mod utils;
pub mod agoraRTC {
    use super::callbacks::*;
    use super::C::*;
    use log::warn;
    use num_derive::FromPrimitive;
    use num_enum::IntoPrimitive;
    use std::ffi::{c_void, CStr, CString};
    use std::option::Option;
    use std::ptr::{null, null_mut};

    #[derive(Copy, Clone, FromPrimitive, IntoPrimitive)]
    #[repr(u32)]
    pub enum VideoDataType {
        YUV420 = 0,
        H264 = 2,
        H265 = 3,
        GENERIC = 6,
        GENERIC_JPEG = 20,
    }

    #[derive(Copy, Clone, FromPrimitive, IntoPrimitive)]
    #[repr(u32)]
    pub enum VideoFrameType {
        AUTO = 0,
        KEY = 3,
        DELTA = 4,
    }

    #[derive(Copy, Clone, FromPrimitive, IntoPrimitive)]
    #[repr(u32)]
    pub enum VideoFrameRate {
        FPS_1 = 1,
        FPS_7 = 7,
        FPS_10 = 10,
        FPS_15 = 15,
        FPS_24 = 24,
        FPS_30 = 30,
        /* 60: 60 fps. Applies to Windows and macOS only. */
        FPS_60 = 60,
    }

    #[derive(Copy, Clone, FromPrimitive, IntoPrimitive)]
    #[repr(u32)]
    pub enum VideoStreamQuality {
        HIGH = 0,
        LOW = 1,
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

    pub type ErrorCode = i32;

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
        pub area_code: AreaCode,
        pub product_id: [u8; 64],
        pub log_cfg: LogConfig,
        pub license_value: [u8; 33],
    }

    impl RtcServiceOption {
        pub fn new(log_path: &str, log_level: LogLevel) -> Self {
            let log_cfg = LogConfig {
                log_disable: false,
                log_disable_desensitize: true,
                log_level,
                log_path: log_path.to_owned(),
            };

            RtcServiceOption {
                area_code: AreaCode::CN,
                product_id: [0; 64],
                log_cfg: log_cfg,
                license_value: [0; 33],
            }
        }
    }
    #[derive(Clone)]
    pub struct LogConfig {
        pub log_disable: bool,
        pub log_disable_desensitize: bool,
        pub log_level: LogLevel,
        pub log_path: String,
    }

    impl From<LogConfig> for log_config_t {
        fn from(config: LogConfig) -> Self {
            let cs = CString::new(config.log_path).unwrap();
            // use heap
            // hope it will be freed by C (don't think so but don't think that's a problem)
            let b = Box::new(cs);
            log_config_t {
                log_disable: config.log_disable,
                log_disable_desensitize: config.log_disable_desensitize,
                log_level: config.log_level.into(),
                log_path: b.as_bytes().as_ptr(),
            }
        }
    }

    impl From<RtcServiceOption> for rtc_service_option_t {
        fn from(opt: RtcServiceOption) -> Self {
            rtc_service_option_t {
                area_code: opt.area_code.into(),
                product_id: opt.product_id,
                log_cfg: opt.log_cfg.clone().into(),
                license_value: opt.license_value,
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

    // See https://adventures.michaelfbryan.com/posts/rust-closures-in-ffi/
    // https://rust-lang.github.io/unsafe-code-guidelines/layout/function-pointers.html
    // https://doc.rust-lang.org/reference/expressions/operator-expr.html#type-cast-expressions
    // https://users.rust-lang.org/t/rust-and-c-interoperability-c-lambdas/67136/3
    // ** only for closures that do not capture (close over) any local variables
    impl agora_rtc_event_handler_t {
        /// default impl of event_handler
        pub fn new() -> Self {
            agora_rtc_event_handler_t {
                on_join_channel_success: Some(on_join_channel_success),
                on_connection_lost: Some(on_connection_lost),
                on_rejoin_channel_success: Some(on_rejoin_channel_success),
                on_error: Some(on_error),
                on_user_joined: Some(on_user_joined),
                on_user_offline: Some(on_user_offline),
                on_user_mute_audio: Some(on_user_mute_audio),
                on_user_mute_video: Some(on_user_mute_video),
                on_audio_data: Some(on_audio_data),
                on_mixed_audio_data: Some(on_mixed_audio_data),
                on_video_data: Some(on_video_data),
                on_target_bitrate_changed: Some(on_target_bitrate_changed),
                on_key_frame_gen_req: Some(on_key_frame_gen_req),
                on_token_privilege_will_expire: Some(on_token_privilege_will_expire),
            }
        }
    }

    impl rtc_channel_options_t {
        /// I don't need audio by default!
        pub fn new() -> Self {
            let codec_opt = audio_codec_option_t {
                audio_codec_type: audio_codec_type_e_AUDIO_CODEC_DISABLED,
                /// Pcm sample rate. Ignored if audio coded is diabled
                pcm_sample_rate: 0,
                /// Pcm channel number. Ignored if audio coded is diabled
                pcm_channel_num: 0,
            };
            rtc_channel_options_t {
                auto_subscribe_audio: false,
                auto_subscribe_video: false,
                subscribe_local_user: false,
                enable_audio_jitter_buffer: false,
                enable_audio_mixer: false,
                audio_codec_opt: codec_opt,
                enable_aut_encryption: false,
            }
        }
    }

    // I don't get intellisense in other file for some reason
    pub struct AgoraApp {
        uid: u32,
        conn_id: Option<u32>,
        c_app_id: CString,
        c_channel_name: CString,
        c_app_token: CString,
        handlers: agora_rtc_event_handler_t,
        service_option: Option<rtc_service_option_t>,
        channel_option: Option<rtc_channel_options_t>,
        default_video_info: Option<video_frame_info_t>,
    }

    // maybe derive Default?
    // https://stackoverflow.com/questions/41510424/most-idiomatic-way-to-create-a-default-struct
    use super::utils::ToCString;
    impl AgoraApp {
        /// using default handler
        pub fn new(app_id: &str) -> Self {
            Self {
                c_app_id: app_id.to_c_string().unwrap(),
                c_channel_name: "".to_c_string().unwrap(),
                c_app_token: "".to_c_string().unwrap(),
                uid: 0,
                conn_id: None,
                handlers: agora_rtc_event_handler_t::new(),
                service_option: None,
                channel_option: None,
                default_video_info: None,
            }
        }

        pub fn set_handlers(&mut self, handlers: agora_rtc_event_handler_t) {
            self.handlers = handlers;
        }

        /// verify license without credential
        /// &str is okay here since it has length parameter
        pub fn license_verify(certificate_str: &str) -> Result<(), ErrorCode> {
            let code = unsafe {
                agora_rtc_license_verify(
                    certificate_str.as_ptr(),
                    certificate_str.len().try_into().unwrap(),
                    std::ptr::null(),
                    0,
                )
            };
            err_2_result(code)
        }

        // TODO: use a object wrapper
        // https://stackoverflow.com/questions/70840454/passing-a-safe-rust-function-pointer-to-c
        // https://adventures.michaelfbryan.com/posts/rust-closures-in-ffi/
        /// init SDK
        /// * `app_id` - You have to use CString to handle null-terminated string since rust String/&str is not zero terminated
        ///    `app_id` should out live the AgoraSDK
        pub fn init(&mut self, option: RtcServiceOption) -> Result<(), ErrorCode> {
            self.service_option = Some(option.into());
            // opt_t should keeps living during the programming running (Static lifetime?)
            // I will use move for safty
            let mut opt = self.service_option.expect("No Service Option");
            let code = unsafe {
                agora_rtc_init(
                    self.c_app_id.as_ptr(),
                    std::ptr::addr_of!(self.handlers),
                    std::ptr::addr_of_mut!(opt),
                )
            };

            err_2_result(code)
        }

        pub fn join_channel(
            &mut self,
            channel_name: &str,
            uid: Option<u32>,
            token: &str,
            option: rtc_channel_options_t,
        ) -> Result<(), ErrorCode> {
            self.channel_option = Some(option);
            self.c_channel_name = channel_name.to_c_string().unwrap();
            self.c_app_token = token.to_c_string().unwrap();
            self.uid = uid.unwrap_or(0);
            // I guess this temporary variable will be alive during the lifetime of AgoraApp
            let mut opt = self.channel_option.expect("No Channel Option");
            let p_opt = std::ptr::addr_of_mut!(opt);

            let code = unsafe {
                // I believe this function won't modify token or options
                agora_rtc_join_channel(
                    self.conn_id.expect("no connection id"),
                    channel_name.as_ptr(),
                    uid.unwrap_or(0),
                    token.as_ptr(),
                    p_opt,
                )
            };
            err_2_result(code)
        }

        // TODO: better error handling
        // https://stackoverflow.com/questions/53183070/what-is-the-defacto-bytes-type-in-rust
        pub fn send_video_data(
            &self,
            buf: &[u8],
            info: &mut video_frame_info_t,
        ) -> Result<(), ErrorCode> {
            let ptr = buf.as_ptr();
            let len: size_t = buf
                .len()
                .try_into()
                .expect("error when converting buffer len in send_video_data");
            // don't think this function will actually mutate info
            let p_i: *mut video_frame_info_t = std::ptr::addr_of_mut!(*info);
            let code = unsafe {
                agora_rtc_send_video_data(
                    self.conn_id.expect("No connection id"),
                    std::mem::transmute(ptr),
                    len,
                    p_i,
                )
            };
            err_2_result(code)
        }

        pub fn set_video_info(&mut self, info: video_frame_info_t) {
            self.default_video_info = Some(info);
        }

        pub fn send_video_data_default(&mut self, buf: &[u8]) -> Result<(), ErrorCode> {
            let mut i = self.default_video_info.expect("No Video Info");
            self.send_video_data(buf, &mut i)
        }

        /// deinit SDK
        pub fn deinit() -> Result<(), i32> {
            let code = unsafe { agora_rtc_fini() };
            err_2_result(code)
        }

        /// set connection id and return the new one
        pub fn create_connection(&mut self) -> Result<u32, ErrorCode> {
            // https://doc.rust-lang.org/reference/expressions/operator-expr.html#type-cast-expressions
            let heap = Box::new(0);
            let ptr = Box::into_raw(heap);
            let code = unsafe { agora_rtc_create_connection(ptr) };
            unsafe {
                match code {
                    0 => {
                        self.conn_id = Some(*ptr);
                        Result::Ok(*ptr)
                    }
                    _ => Result::Err(code),
                }
            }
        }

        pub fn destroy_connection(&mut self) -> Result<(), ErrorCode> {
            match self.conn_id {
                Some(id) => {
                    let code = unsafe { agora_rtc_destroy_connection(id) };
                    self.conn_id = None;
                    err_2_result(code)
                }
                None => {
                    warn!("No connection id");
                    Result::Ok(())
                }
            }
        }

        pub fn leave_channel(&self) -> Result<(), ErrorCode> {
            match self.conn_id {
                Some(id) => {
                    let code = unsafe { agora_rtc_leave_channel(id) };
                    err_2_result(code)
                }
                None => {
                    warn!("No connection id");
                    Result::Ok(())
                }
            }
        }

        pub fn mute_local_audio(&self, is_muted: bool) -> Result<(), ErrorCode> {
            let code = unsafe {
                agora_rtc_mute_local_audio(self.conn_id.expect("No connection id"), is_muted)
            };
            err_2_result(code)
        }
    }
    impl Drop for AgoraApp {
        // https://doc.rust-lang.org/nomicon/destructors.html
        // After drop is run, Rust will recursively try to drop all of the fields of self.
        fn drop(&mut self) {
            // don't actually need to check if it's None
            self.leave_channel();
            self.destroy_connection();
            AgoraApp::deinit();
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
