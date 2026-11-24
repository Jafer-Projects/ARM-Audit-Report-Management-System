use serde_json::Value;
use std::{ffi::c_void, io};

const TARGET: &str = "ARM-Audit-Report-Management-System/cloud-session/v1";
const MAX_SECRET_BYTES: usize = 2400;

fn validate_session(raw: &str) -> Result<(), String> {
    if raw.len() > MAX_SECRET_BYTES { return Err("Cloud session exceeds the credential-vault size limit".into()); }
    let value: Value = serde_json::from_str(raw).map_err(|_| "Cloud session is not valid JSON".to_string())?;
    for field in ["accessToken", "refreshToken"] {
        if value.get(field).and_then(Value::as_str).filter(|token| !token.is_empty()).is_none() {
            return Err(format!("Cloud session is missing {field}"));
        }
    }
    Ok(())
}

#[cfg(windows)]
mod platform {
    use super::*;
    use std::{os::windows::ffi::OsStrExt, ptr};

    #[repr(C)] struct FileTime { low: u32, high: u32 }
    #[repr(C)] struct CredentialW {
        flags: u32, credential_type: u32, target_name: *mut u16, comment: *mut u16,
        last_written: FileTime, credential_blob_size: u32, credential_blob: *mut u8,
        persist: u32, attribute_count: u32, attributes: *mut c_void,
        target_alias: *mut u16, user_name: *mut u16,
    }
    #[link(name = "Advapi32")]
    extern "system" {
        fn CredWriteW(credential: *const CredentialW, flags: u32) -> i32;
        fn CredReadW(target: *const u16, credential_type: u32, flags: u32, credential: *mut *mut CredentialW) -> i32;
        fn CredDeleteW(target: *const u16, credential_type: u32, flags: u32) -> i32;
        fn CredFree(buffer: *mut c_void);
    }
    fn wide(value: &str) -> Vec<u16> { std::ffi::OsStr::new(value).encode_wide().chain(std::iter::once(0)).collect() }

    pub fn store(raw: &str) -> Result<(), String> {
        let target = wide(TARGET); let username = wide("ARM desktop cloud session");
        let mut bytes = raw.as_bytes().to_vec();
        let credential = CredentialW {
            flags: 0, credential_type: 1, target_name: target.as_ptr() as *mut u16,
            comment: ptr::null_mut(), last_written: FileTime { low: 0, high: 0 },
            credential_blob_size: bytes.len() as u32, credential_blob: bytes.as_mut_ptr(),
            persist: 2, attribute_count: 0, attributes: ptr::null_mut(),
            target_alias: ptr::null_mut(), user_name: username.as_ptr() as *mut u16,
        };
        if unsafe { CredWriteW(&credential, 0) } == 0 {
            return Err(format!("Windows Credential Manager write failed: {}", io::Error::last_os_error()));
        }
        Ok(())
    }

    pub fn load() -> Result<Option<String>, String> {
        let target = wide(TARGET); let mut pointer = ptr::null_mut();
        if unsafe { CredReadW(target.as_ptr(), 1, 0, &mut pointer) } == 0 {
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(1168) { return Ok(None); }
            return Err(format!("Windows Credential Manager read failed: {error}"));
        }
        if pointer.is_null() { return Err("Windows Credential Manager returned an empty credential".into()); }
        let credential = unsafe { &*pointer };
        let raw = if credential.credential_blob_size as usize > MAX_SECRET_BYTES {
            Err("Stored cloud session exceeds the credential-vault size limit".to_string())
        } else if credential.credential_blob_size == 0 || credential.credential_blob.is_null() {
            Err("Stored cloud session is corrupt".to_string())
        } else {
            let bytes = unsafe { std::slice::from_raw_parts(credential.credential_blob, credential.credential_blob_size as usize) };
            String::from_utf8(bytes.to_vec()).map_err(|_| "Stored cloud session is not valid UTF-8".to_string())
        };
        unsafe { CredFree(pointer as *mut c_void); }
        let value = raw?; super::validate_session(&value)?; Ok(Some(value))
    }

    pub fn clear() -> Result<(), String> {
        let target = wide(TARGET);
        if unsafe { CredDeleteW(target.as_ptr(), 1, 0) } == 0 {
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(1168) { return Ok(()); }
            return Err(format!("Windows Credential Manager delete failed: {error}"));
        }
        Ok(())
    }
}

#[cfg(not(windows))]
mod platform {
    pub fn store(_: &str) -> Result<(), String> { Err("Secure cloud-session storage is supported only on Windows Credential Manager".into()) }
    pub fn load() -> Result<Option<String>, String> { Err("Secure cloud-session storage is supported only on Windows Credential Manager".into()) }
    pub fn clear() -> Result<(), String> { Err("Secure cloud-session storage is supported only on Windows Credential Manager".into()) }
}

#[tauri::command]
pub fn store_cloud_session(session_json: String) -> Result<(), String> { validate_session(&session_json)?; platform::store(&session_json) }
#[tauri::command]
pub fn load_cloud_session() -> Result<Option<String>, String> { platform::load() }
#[tauri::command]
pub fn clear_cloud_session() -> Result<(), String> { platform::clear() }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_invalid_or_incomplete_session_before_touching_os_vault() {
        assert!(validate_session("not-json").is_err());
        assert!(validate_session(r#"{"accessToken":"a"}"#).is_err());
        assert!(validate_session(r#"{"accessToken":"a","refreshToken":"b"}"#).is_ok());
    }
    #[test]
    fn rejects_oversized_session() {
        let oversized = format!(r#"{{"accessToken":"{}","refreshToken":"b"}}"#, "x".repeat(MAX_SECRET_BYTES));
        assert!(validate_session(&oversized).is_err());
    }
}
