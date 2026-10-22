//! High-performance hexadecimal encoding and decoding utilities.

use crate::error::CoreError;

const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

/// Encodes binary bytes into a lowercase hexadecimal string without extra allocations.
pub fn encode_hex(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        encoded.push(HEX_CHARS[(byte >> 4) as usize] as char);
        encoded.push(HEX_CHARS[(byte & 0x0f) as usize] as char);
    }
    encoded
}

/// Decodes an even-length hexadecimal string into raw binary bytes.
pub fn decode_hex(encoded: &str) -> Result<Vec<u8>, CoreError> {
    let bytes = encoded.as_bytes();
    if bytes.len() % 2 != 0 {
        return Err(CoreError::Serialization(
            "hexadecimal string must contain an even number of characters".into(),
        ));
    }

    bytes
        .chunks_exact(2)
        .map(|pair| {
            let high = decode_nibble(pair[0])?;
            let low = decode_nibble(pair[1])?;
            Ok((high << 4) | low)
        })
        .collect()
}

/// Helper function to decode a single hexadecimal ASCII character into a 4-bit nibble.
pub fn decode_nibble(value: u8) -> Result<u8, CoreError> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => Err(CoreError::Serialization(
            "string contains non-hexadecimal characters".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_and_decode_hex() {
        let original = b"FamShield Safe Protection!";
        let encoded = encode_hex(original);
        let decoded = decode_hex(&encoded).expect("valid hex decode");
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_decode_hex_invalid() {
        assert!(decode_hex("abc").is_err());
        assert!(decode_hex("zz").is_err());
    }
}
