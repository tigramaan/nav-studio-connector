use crate::domain::{ConnectorError, EndpointInspection, ErrorCode, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::ffi::c_void;
use std::ptr::{null, null_mut};

const TRUST_TARGET: &str = "CurrentUser\\Root";
const X509_ASN_ENCODING: u32 = 0x0000_0001;
const PKCS_7_ASN_ENCODING: u32 = 0x0001_0000;
const CERT_STORE_ADD_REPLACE_EXISTING: u32 = 3;
const CERT_FIND_SHA1_HASH: u32 = 0x0001_0000;

type CertificateStore = *mut c_void;
type CertificateContext = c_void;

#[repr(C)]
struct CryptHashBlob {
    size: u32,
    data: *mut u8,
}

#[link(name = "Crypt32")]
unsafe extern "system" {
    fn CertOpenSystemStoreW(provider: usize, subsystem_protocol: *const u16) -> CertificateStore;
    fn CertAddEncodedCertificateToStore(
        store: CertificateStore,
        encoding_type: u32,
        encoded: *const u8,
        encoded_size: u32,
        add_disposition: u32,
        context: *mut *const CertificateContext,
    ) -> i32;
    fn CertFindCertificateInStore(
        store: CertificateStore,
        encoding_type: u32,
        find_flags: u32,
        find_type: u32,
        find_parameter: *const c_void,
        previous_context: *const CertificateContext,
    ) -> *const CertificateContext;
    fn CertDeleteCertificateFromStore(context: *const CertificateContext) -> i32;
    fn CertCloseStore(store: CertificateStore, flags: u32) -> i32;
}

pub fn install_certificate(inspection: &EndpointInspection) -> Result<String> {
    let certificate_der = decode_pem(&inspection.certificate_pem)?;
    let encoded_size = certificate_der.len().try_into().map_err(|_| {
        ConnectorError::new(
            ErrorCode::InvalidInput,
            "Public certificate exceeds the Windows API size limit",
            false,
        )
    })?;
    let store = open_current_user_root()?;
    let added = unsafe {
        CertAddEncodedCertificateToStore(
            store,
            X509_ASN_ENCODING | PKCS_7_ASN_ENCODING,
            certificate_der.as_ptr(),
            encoded_size,
            CERT_STORE_ADD_REPLACE_EXISTING,
            null_mut(),
        )
    };
    let api_error = std::io::Error::last_os_error();
    unsafe { CertCloseStore(store, 0) };
    if added == 0 {
        return Err(store_error(
            "Windows could not add the certificate",
            api_error,
        ));
    }
    Ok(TRUST_TARGET.to_string())
}

pub fn remove_certificate(
    _fingerprint_sha256: &str,
    fingerprint_sha1: &str,
    trust_target: &str,
) -> Result<()> {
    if trust_target != TRUST_TARGET
        || fingerprint_sha1.len() != 40
        || !fingerprint_sha1.chars().all(|ch| ch.is_ascii_hexdigit())
    {
        return Err(ConnectorError::new(
            ErrorCode::InvalidInput,
            "Certificate removal target is not connector-owned",
            false,
        ));
    }
    let mut hash = hex::decode(fingerprint_sha1).map_err(|_| {
        ConnectorError::new(
            ErrorCode::InvalidInput,
            "Certificate SHA-1 is malformed",
            false,
        )
    })?;
    let blob = CryptHashBlob {
        size: hash.len() as u32,
        data: hash.as_mut_ptr(),
    };
    let store = open_current_user_root()?;
    let context = unsafe {
        CertFindCertificateInStore(
            store,
            X509_ASN_ENCODING | PKCS_7_ASN_ENCODING,
            0,
            CERT_FIND_SHA1_HASH,
            (&blob as *const CryptHashBlob).cast(),
            null(),
        )
    };
    if context.is_null() {
        let api_error = std::io::Error::last_os_error();
        unsafe { CertCloseStore(store, 0) };
        return Err(store_error(
            "Connector-owned certificate was not found",
            api_error,
        ));
    }
    let deleted = unsafe { CertDeleteCertificateFromStore(context) };
    let api_error = std::io::Error::last_os_error();
    unsafe { CertCloseStore(store, 0) };
    if deleted == 0 {
        return Err(store_error(
            "Windows could not remove the certificate",
            api_error,
        ));
    }
    Ok(())
}

fn open_current_user_root() -> Result<CertificateStore> {
    let name: Vec<u16> = "ROOT".encode_utf16().chain(std::iter::once(0)).collect();
    let store = unsafe { CertOpenSystemStoreW(0, name.as_ptr()) };
    if store.is_null() {
        return Err(store_error(
            "Windows could not open CurrentUser Root",
            std::io::Error::last_os_error(),
        ));
    }
    Ok(store)
}

fn decode_pem(pem: &str) -> Result<Vec<u8>> {
    let payload: String = pem
        .lines()
        .filter(|line| !line.starts_with("-----"))
        .collect();
    if payload.is_empty() || payload.len() > 2 * 1024 * 1024 {
        return Err(ConnectorError::new(
            ErrorCode::InvalidInput,
            "Public certificate PEM is malformed",
            false,
        ));
    }
    STANDARD.decode(payload).map_err(|_| {
        ConnectorError::new(
            ErrorCode::InvalidInput,
            "Public certificate PEM is malformed",
            false,
        )
    })
}

fn store_error(message: &str, source: std::io::Error) -> ConnectorError {
    let denied = source.raw_os_error() == Some(5);
    ConnectorError::new(
        if denied {
            ErrorCode::PermissionDenied
        } else {
            ErrorCode::TrustStoreFailed
        },
        message,
        false,
    )
    .with_detail("os_error", source.raw_os_error().unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_malformed_pem() {
        assert!(decode_pem("not a certificate").is_err());
    }
}
