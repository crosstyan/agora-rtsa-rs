#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod C {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

// https://stackoverflow.com/questions/24145823/how-do-i-convert-a-c-string-into-a-rust-string-and-back-via-ffi
// https://doc.rust-lang.org/reference/items/extern-crates.htm

pub mod agoraRTC {
    use super::defaultCallbacks::*;
    use super::C::*;
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

    #[derive(Clone)]
    pub struct LogConfig {
        pub log_disable: bool,
        pub log_disable_desensitize: bool,
        pub log_level: LogLevel,
    }

    impl From<LogConfig> for log_config_t {
        /// TODO: figure out why String isn't work
        fn from(config: LogConfig) -> Self {
            log_config_t {
                log_disable: config.log_disable,
                log_disable_desensitize: config.log_disable_desensitize,
                log_level: config.log_level.into(),
                log_path: null(),
            }
        }
    }

    impl From<&RtcServiceOption> for rtc_service_option_t {
        fn from(opt: &RtcServiceOption) -> Self {
            rtc_service_option_t {
                area_code: opt.area_code.into(),
                product_id: opt.product_id,
                log_cfg: opt.log_cfg.clone().into(),
                license_value: opt.license_value,
            }
        }
    }

    impl RtcServiceOption {
        /// set [log_path_c] as [std::ptr::null()] will set the log path to pwd.
        /// 用于存放 Agora SDK 日志的目录。如果 log_path 设为 NULL，则日志位于当前应用程序的 pwd 目录。
        pub fn to_c_type(&self, log_path_c: *const u8) -> rtc_service_option_t {
            let mut opt_t: rtc_service_option_t = self.into();
            // You have to set logs before deallocation
            opt_t.log_cfg.log_path = log_path_c;
            opt_t
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

    fn err_2_result(code: ErrorCode) -> Result<(), ErrorCode> {
        match code {
            0 => Result::Ok(()),
            _ => Result::Err(code),
        }
    }

    pub fn err_2_reason(code: ErrorCode) -> String {
        unsafe {
            let pVersion = agora_rtc_err_2_str(code);
            CStr::from_ptr(pVersion).to_str().unwrap().to_owned()
        }
    }

    /// verify license without credential
    /// &str is okay here since it has length parameter
    pub fn license_verify(certificate_str: &str) -> Result<(), ErrorCode> {
        let code = unsafe {
            agora_rtc_license_verify(
                certificate_str.as_ptr(),
                certificate_str.len().try_into().unwrap(),
                null(),
                0,
            )
        };
        err_2_result(code)
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

    // TODO: use a object wrapper
    // https://stackoverflow.com/questions/70840454/passing-a-safe-rust-function-pointer-to-c
    // https://adventures.michaelfbryan.com/posts/rust-closures-in-ffi/
    /// init SDK
    pub fn init(
        app_id: &str,
        opt: rtc_service_option_t,
        handlers: agora_rtc_event_handler_t,
    ) -> Result<(), ErrorCode> {
        // opt_t should keeps living during the programming running (Static lifetime?)
        // I will use move for safty
        let mut opt_t: rtc_service_option_t = opt.into();
        let p_handler = &handlers as *const agora_rtc_event_handler_t;
        // You have to use CString to handle null-terminated string since rust String/&str is not zero terminated
        let c_app_id = CString::new(app_id).expect("CString::new failed");
        let code =
            unsafe { agora_rtc_init(c_app_id.as_ptr(), p_handler, std::ptr::addr_of_mut!(opt_t)) };

        err_2_result(code)
    }

    /// deinit SDK
    pub fn deinit() -> Result<(), i32> {
        let code = unsafe { agora_rtc_fini() };
        err_2_result(code)
    }

    pub fn create_connection() -> Result<u32, ErrorCode> {
        // https://doc.rust-lang.org/reference/expressions/operator-expr.html#type-cast-expressions
        // WARNING: this value will be mutated
        let conn_id: u32 = 0;
        let ptr = &conn_id as *const u32 as *mut u32;
        let code = unsafe { agora_rtc_create_connection(ptr) };
        unsafe {
            match code {
                0 => Result::Ok(*ptr),
                _ => Result::Err(code),
            }
        }
    }
    pub fn destroy_connection(conn_id: u32) -> Result<(), ErrorCode> {
        let code = unsafe { agora_rtc_destroy_connection(conn_id) };
        err_2_result(code)
    }

    pub fn join_channel(
        conn_id: u32,
        channel_name: &str,
        uid: Option<u32>,
        token: *const u8,
        options: rtc_channel_options_t,
    ) -> Result<(), ErrorCode> {
        let p_o: *mut rtc_channel_options_t =
            &options as *const rtc_channel_options_t as *mut rtc_channel_options_t;
        let c_chan_name = CString::new(channel_name).unwrap();
        let code = unsafe {
            // I believe this function won't modify token or options
            agora_rtc_join_channel(conn_id, c_chan_name.as_ptr(), uid.unwrap_or(0), token, p_o)
        };
        err_2_result(code)
    }

    pub fn leave_channel(conn_id: u32) -> Result<(), ErrorCode> {
        let code = unsafe { agora_rtc_leave_channel(conn_id) };
        err_2_result(code)
    }

    pub fn mute_local_audio(conn_id: u32, is_muted: bool) -> Result<(), ErrorCode> {
        let code = unsafe { agora_rtc_mute_local_audio(conn_id, is_muted) };
        err_2_result(code)
    }
    // TODO: a safe interface
    // https://stackoverflow.com/questions/53183070/what-is-the-defacto-bytes-type-in-rust
    pub fn send_video_data(
        conn_id: u32,
        buf: &[u8],
        info: &video_frame_info_t,
    ) -> Result<(), ErrorCode> {
        let ptr = buf.as_ptr();
        let len: size_t = buf
            .len()
            .try_into()
            .expect("error when converting buffer len in send_video_data");
        // don't think this function will actually mutate info
        let p_i: *mut video_frame_info_t =
            info as *const video_frame_info_t as *mut video_frame_info_t;
        let code =
            unsafe { agora_rtc_send_video_data(conn_id, std::mem::transmute(ptr), len, p_i) };
        err_2_result(code)
    }
}

pub mod defaultCallbacks {
    use std::ffi::{c_void, CStr};

    use super::C::*;
    use log::{error, info, warn};
    /// Occurs when local user joins channel successfully.
    /// * `conn_id` -  Connection identification
    /// * `uid`     -   local uid
    /// * `elapsed_ms` - Time elapsed (ms) since channel is established
    pub extern "C" fn on_join_channel_success(conn_id: u32, uid: u32, elapsed_ms: i32) {
        info!(
            "join_channel_success, conn_id: {}, uid: {}, elapsed_ms: {}",
            conn_id, uid, elapsed_ms
        );
    }
    pub extern "C" fn on_connection_lost(conn_id: u32) {
        error!("connection_lost, conn_id: {}", conn_id);
    }
    pub extern "C" fn on_rejoin_channel_success(conn_id: u32, uid: u32, elapsed_ms: i32) {
        info!(
            "rejoin_channel_success, conn_id: {}, uid: {}, elapsed_ms: {}",
            conn_id, uid, elapsed_ms
        );
    }

    ///Report error message during runtime.
    ///In most cases, it means SDK can't fix the issue and application should take action.
    /// * `conn_id` Connection identification
    /// * `code`    Error code, see #agora_err_code_e
    /// * `msg`     Error message
    pub extern "C" fn on_error(conn_id: u32, code: i32, msg: *const u8) {
        let m = unsafe { CStr::from_ptr(msg) }.to_str();
        let message = match code
            .try_into()
            .expect("Error converting on_error error code")
        {
            agora_err_code_e_ERR_INVALID_APP_ID => "Invalid App ID. Please double check.",
            agora_err_code_e_ERR_INVALID_CHANNEL_NAME => "Invalid channel name",
            agora_err_code_e_ERR_INVALID_TOKEN => "Invalid token",
            agora_err_code_e_ERR_DYNAMIC_TOKEN_BUT_USE_STATIC_KEY => {
                "Dynamic token is enabled but is not provided."
            }
            _ => "Other Error",
        };
        error!(
            "ERROR!, conn_id: {}, {:?}, {:?}, Code: {}",
            conn_id, message, m, code
        );
    }

    pub extern "C" fn on_user_joined(conn_id: u32, uid: u32, elapsed_ms: i32) {
        info!(
            "user_join, conn_id: {}, uid: {}, elapsed_ms: {}",
            conn_id, uid, elapsed_ms
        );
    }

    pub extern "C" fn on_user_offline(conn_id: u32, uid: u32, reason: i32) {
        warn!(
            "user_offline, conn_id: {}, uid: {}, user_offline_reason_e: {}",
            conn_id, uid, reason
        );
    }

    pub extern "C" fn on_user_mute_audio(conn_id: u32, uid: u32, muted: bool) {
        info!(
            "user_mute_audio, conn_id: {}, uid: {}, is_muted: {}",
            conn_id, uid, muted
        );
    }

    pub extern "C" fn on_user_mute_video(conn_id: u32, uid: u32, muted: bool) {
        info!(
            "user_mute_video, conn_id: {}, uid: {}, is_muted: {}",
            conn_id, uid, muted
        );
    }

    pub extern "C" fn on_audio_data(
        _conn_id: u32,
        _uid: u32,
        _sent_ts: u16,
        _data_ptr: *const c_void,
        _data_length: u64,
        _info_ptr: *const audio_frame_info_t,
    ) {
        // won't do anything default
    }

    /// Occurs every 20ms.
    pub extern "C" fn on_mixed_audio_data(
        _conn_id: u32,
        _data_ptr: *const c_void,
        _data_length: u64,
        _info_ptr: *const audio_frame_info_t,
    ) {
        // won't do anything default
    }

    pub extern "C" fn on_video_data(
        _conn_id: u32,
        _uid: u32,
        _sent_ts: u16,
        _data_ptr: *const c_void,
        _data_length: u64,
        _info_ptr: *const video_frame_info_t,
    ) {
        // won't do anything default
    }

    pub extern "C" fn on_target_bitrate_changed(conn_id: u32, target_bps: u32) {
        info!(
            "target_bitrate_changed, conn_id: {}, target_bps: {}",
            conn_id, target_bps
        );
    }

    pub extern "C" fn on_key_frame_gen_req(conn_id: u32, uid: u32, stream_type: u32) {
        info!(
            "user_mute_audio, conn_id: {}, uid: {}, video_stream_type_e: {}",
            conn_id, uid, stream_type
        );
    }

    pub extern "C" fn on_token_privilege_will_expire(conn_id: u32, token: *const u8) {
        let t = unsafe { CStr::from_ptr(token) }.to_str();
        info!(
            "user_mute_audio, conn_id: {}, The token will expire: {:?}",
            conn_id, t
        );
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
