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

/// Report error message during runtime.
/// In most cases, it means SDK can't fix the issue and application should take action.
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
