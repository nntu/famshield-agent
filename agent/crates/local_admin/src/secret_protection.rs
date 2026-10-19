use crate::auth::AuthError;

const DPAPI_PREFIX: &str = "dpapi-machine-v1:";
const TEST_PREFIX: &str = "test-only-v1:";

pub(crate) struct UnprotectedSecret {
    pub plaintext: String,
    pub needs_migration: bool,
}

pub(crate) fn protect_secret(plaintext: &str) -> Result<String, AuthError> {
    #[cfg(windows)]
    {
        protect_with_dpapi(plaintext.as_bytes()).map(|ciphertext| {
            let mut encoded = String::with_capacity(DPAPI_PREFIX.len() + ciphertext.len() * 2);
            encoded.push_str(DPAPI_PREFIX);
            encoded.push_str(&encode_hex(&ciphertext));
            encoded
        })
    }

    #[cfg(not(windows))]
    {
        // FamShield ships only on Windows. This reversible codec exists solely so
        // cross-platform CI can exercise the persistence and migration paths.
        Ok(format!("{TEST_PREFIX}{}", encode_hex(plaintext.as_bytes())))
    }
}

pub(crate) fn unprotect_secret(stored: &str) -> Result<UnprotectedSecret, AuthError> {
    if let Some(encoded) = stored.strip_prefix(DPAPI_PREFIX) {
        #[cfg(windows)]
        {
            let ciphertext = decode_hex(encoded)?;
            let plaintext = unprotect_with_dpapi(&ciphertext)?;
            return String::from_utf8(plaintext)
                .map(|plaintext| UnprotectedSecret {
                    plaintext,
                    needs_migration: false,
                })
                .map_err(|_| AuthError::Protection("DPAPI plaintext is not valid UTF-8".into()));
        }

        #[cfg(not(windows))]
        {
            let _ = encoded;
            return Err(AuthError::Protection(
                "DPAPI-protected secret can only be opened on Windows".into(),
            ));
        }
    }

    if let Some(encoded) = stored.strip_prefix(TEST_PREFIX) {
        #[cfg(not(windows))]
        {
            let plaintext = decode_hex(encoded)?;
            return String::from_utf8(plaintext)
                .map(|plaintext| UnprotectedSecret {
                    plaintext,
                    needs_migration: false,
                })
                .map_err(|_| AuthError::Protection("test secret is not valid UTF-8".into()));
        }

        #[cfg(windows)]
        {
            let _ = encoded;
            return Err(AuthError::Protection(
                "test-only secret encoding is not accepted by Windows builds".into(),
            ));
        }
    }

    // Existing installations stored Argon2 PHC strings directly. Accept them
    // once and rewrite them through DPAPI after successful authentication.
    Ok(UnprotectedSecret {
        plaintext: stored.to_owned(),
        needs_migration: true,
    })
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn decode_hex(encoded: &str) -> Result<Vec<u8>, AuthError> {
    if !encoded.as_bytes().chunks_exact(2).remainder().is_empty() {
        return Err(AuthError::Protection(
            "invalid protected-secret encoding".into(),
        ));
    }

    encoded
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = decode_nibble(pair[0])?;
            let low = decode_nibble(pair[1])?;
            Ok((high << 4) | low)
        })
        .collect()
}

fn decode_nibble(value: u8) -> Result<u8, AuthError> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => Err(AuthError::Protection(
            "invalid protected-secret encoding".into(),
        )),
    }
}

#[cfg(windows)]
fn protect_with_dpapi(plaintext: &[u8]) -> Result<Vec<u8>, AuthError> {
    use windows::core::w;
    use windows::Win32::Foundation::{LocalFree, HLOCAL};
    use windows::Win32::Security::Cryptography::{
        CryptProtectData, CRYPTPROTECT_LOCAL_MACHINE, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    let input = CRYPT_INTEGER_BLOB {
        cbData: plaintext
            .len()
            .try_into()
            .map_err(|_| AuthError::Protection("secret is too large for DPAPI".into()))?,
        pbData: plaintext.as_ptr().cast_mut(),
    };
    let mut output = CRYPT_INTEGER_BLOB::default();
    unsafe {
        CryptProtectData(
            &input,
            w!("FamShield machine secret"),
            None,
            None,
            None,
            CRYPTPROTECT_LOCAL_MACHINE | CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
        .map_err(|error| AuthError::Protection(format!("CryptProtectData failed: {error}")))?;

        let ciphertext = std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec();
        let _ = LocalFree(HLOCAL(output.pbData.cast()));
        Ok(ciphertext)
    }
}

#[cfg(windows)]
fn unprotect_with_dpapi(ciphertext: &[u8]) -> Result<Vec<u8>, AuthError> {
    use windows::Win32::Foundation::{LocalFree, HLOCAL};
    use windows::Win32::Security::Cryptography::{
        CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    let input = CRYPT_INTEGER_BLOB {
        cbData: ciphertext
            .len()
            .try_into()
            .map_err(|_| AuthError::Protection("secret is too large for DPAPI".into()))?,
        pbData: ciphertext.as_ptr().cast_mut(),
    };
    let mut output = CRYPT_INTEGER_BLOB::default();
    unsafe {
        CryptUnprotectData(
            &input,
            None,
            None,
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
        .map_err(|error| AuthError::Protection(format!("CryptUnprotectData failed: {error}")))?;

        let plaintext = std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec();
        let _ = LocalFree(HLOCAL(output.pbData.cast()));
        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protected_secret_round_trip() {
        let protected = protect_secret("sensitive-value").unwrap();
        assert_ne!(protected, "sensitive-value");
        let opened = unprotect_secret(&protected).unwrap();
        assert_eq!(opened.plaintext, "sensitive-value");
        assert!(!opened.needs_migration);
    }

    #[test]
    fn legacy_secret_is_marked_for_migration() {
        let opened = unprotect_secret("$argon2id$legacy").unwrap();
        assert_eq!(opened.plaintext, "$argon2id$legacy");
        assert!(opened.needs_migration);
    }
}
