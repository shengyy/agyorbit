//! Windows Credential Manager access, matching go-keyring's layout:
//! a generic credential whose target is `service:account` and whose blob is
//! the raw UTF-8 secret.

use std::ptr;

use windows_sys::Win32::Foundation::{ERROR_NOT_FOUND, GetLastError};
use windows_sys::Win32::Security::Credentials::{
    CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC, CREDENTIALW, CredDeleteW, CredFree, CredReadW,
    CredWriteW,
};

use crate::error::{Error, Result};

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn target(service: &str, account: &str) -> Vec<u16> {
    wide(&format!("{service}:{account}"))
}

pub fn get(service: &str, account: &str) -> Result<Option<String>> {
    let target = target(service, account);
    let mut credential: *mut CREDENTIALW = ptr::null_mut();
    // SAFETY: `target` is a NUL-terminated UTF-16 string that outlives the
    // call, and `credential` is freed with `CredFree` below.
    let ok = unsafe { CredReadW(target.as_ptr(), CRED_TYPE_GENERIC, 0, &mut credential) };
    if ok == 0 {
        let code = unsafe { GetLastError() };
        if code == ERROR_NOT_FOUND {
            return Ok(None);
        }
        return Err(Error::SecretStore(format!("CredReadW failed with {code}")));
    }
    // SAFETY: on success `credential` points to a valid CREDENTIALW whose blob
    // is `CredentialBlobSize` bytes long.
    let bytes = unsafe {
        let cred = &*credential;
        std::slice::from_raw_parts(cred.CredentialBlob, cred.CredentialBlobSize as usize).to_vec()
    };
    unsafe { CredFree(credential as *const core::ffi::c_void) };
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| Error::SecretStore("credential blob is not UTF-8".into()))
}

pub fn set(service: &str, account: &str, secret: &str) -> Result<()> {
    let mut target = target(service, account);
    let mut user = wide(account);
    let mut blob = secret.as_bytes().to_vec();
    let credential = CREDENTIALW {
        Type: CRED_TYPE_GENERIC,
        TargetName: target.as_mut_ptr(),
        CredentialBlobSize: blob.len() as u32,
        CredentialBlob: blob.as_mut_ptr(),
        Persist: CRED_PERSIST_LOCAL_MACHINE,
        UserName: user.as_mut_ptr(),
        ..Default::default()
    };
    // SAFETY: every pointer in `credential` borrows a buffer that lives until
    // the call returns.
    let ok = unsafe { CredWriteW(&credential, 0) };
    if ok == 0 {
        let code = unsafe { GetLastError() };
        return Err(Error::SecretStore(format!("CredWriteW failed with {code}")));
    }
    Ok(())
}

pub fn delete(service: &str, account: &str) -> Result<()> {
    let target = target(service, account);
    // SAFETY: `target` is a NUL-terminated UTF-16 string.
    let ok = unsafe { CredDeleteW(target.as_ptr(), CRED_TYPE_GENERIC, 0) };
    if ok == 0 {
        let code = unsafe { GetLastError() };
        if code != ERROR_NOT_FOUND {
            return Err(Error::SecretStore(format!(
                "CredDeleteW failed with {code}"
            )));
        }
    }
    Ok(())
}
