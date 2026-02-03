//! # SBE Type Helpers
//!
//! Helper types for SBE encoding/decoding of complex types.

use super::error::{SbeError, SbeResult};
use rust_decimal::Decimal;
use uuid::Uuid;

/// SBE UUID representation (16 bytes: high u64 + low u64).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SbeUuid {
    /// High 64 bits.
    pub high: u64,
    /// Low 64 bits.
    pub low: u64,
}

#[allow(clippy::indexing_slicing)]
impl SbeUuid {
    /// Size in bytes.
    pub const SIZE: usize = 16;

    /// Creates from a UUID.
    #[must_use]
    pub fn from_uuid(uuid: Uuid) -> Self {
        let bytes = uuid.as_bytes();
        let high = u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        let low = u64::from_le_bytes([
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        ]);
        Self { high, low }
    }

    /// Converts to a UUID.
    #[must_use]
    pub fn to_uuid(self) -> Uuid {
        let mut bytes = [0u8; 16];
        bytes[0..8].copy_from_slice(&self.high.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.low.to_le_bytes());
        Uuid::from_bytes(bytes)
    }

    /// Encodes to a buffer.
    ///
    /// # Errors
    ///
    /// Returns error if buffer is too small.
    pub fn encode(&self, buffer: &mut [u8]) -> SbeResult<()> {
        if buffer.len() < Self::SIZE {
            return Err(SbeError::BufferTooSmall {
                needed: Self::SIZE,
                available: buffer.len(),
            });
        }
        buffer[0..8].copy_from_slice(&self.high.to_le_bytes());
        buffer[8..16].copy_from_slice(&self.low.to_le_bytes());
        Ok(())
    }

    /// Decodes from a buffer.
    ///
    /// # Errors
    ///
    /// Returns error if buffer is too small.
    pub fn decode(buffer: &[u8]) -> SbeResult<Self> {
        if buffer.len() < Self::SIZE {
            return Err(SbeError::BufferTooSmall {
                needed: Self::SIZE,
                available: buffer.len(),
            });
        }
        let high = u64::from_le_bytes([
            buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7],
        ]);
        let low = u64::from_le_bytes([
            buffer[8], buffer[9], buffer[10], buffer[11], buffer[12], buffer[13], buffer[14],
            buffer[15],
        ]);
        Ok(Self { high, low })
    }
}

/// SBE Decimal representation (mantissa i64 + exponent i8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SbeDecimal {
    /// Mantissa (significand).
    pub mantissa: i64,
    /// Exponent (scale, negative for decimal places).
    pub exponent: i8,
}

#[allow(clippy::indexing_slicing)]
impl SbeDecimal {
    /// Size in bytes (8 + 1 = 9).
    pub const SIZE: usize = 9;

    /// Creates from a Decimal.
    #[must_use]
    pub fn from_decimal(decimal: Decimal) -> Self {
        // Decimal stores scale as u32, we need to convert to negative exponent
        let scale = decimal.scale();
        let mantissa_i128 = decimal.mantissa();
        let mantissa = mantissa_i128 as i64;
        // Scale is the number of decimal places, so exponent = -scale
        let exponent = -(scale as i8);
        Self { mantissa, exponent }
    }

    /// Converts to a Decimal.
    ///
    /// # Errors
    ///
    /// Returns error if the decimal cannot be represented.
    pub fn to_decimal(self) -> SbeResult<Decimal> {
        // Convert exponent back to scale
        if self.exponent > 0 {
            // Positive exponent means multiply by 10^exponent
            let multiplier = 10i64
                .checked_pow(self.exponent as u32)
                .ok_or_else(|| SbeError::InvalidDecimal("exponent overflow".to_string()))?;
            let mantissa = self
                .mantissa
                .checked_mul(multiplier)
                .ok_or_else(|| SbeError::InvalidDecimal("mantissa overflow".to_string()))?;
            Ok(Decimal::new(mantissa, 0))
        } else {
            // Negative exponent is the scale
            let scale = (-self.exponent) as u32;
            Ok(Decimal::new(self.mantissa, scale))
        }
    }

    /// Encodes to a buffer.
    ///
    /// # Errors
    ///
    /// Returns error if buffer is too small.
    pub fn encode(&self, buffer: &mut [u8]) -> SbeResult<()> {
        if buffer.len() < Self::SIZE {
            return Err(SbeError::BufferTooSmall {
                needed: Self::SIZE,
                available: buffer.len(),
            });
        }
        buffer[0..8].copy_from_slice(&self.mantissa.to_le_bytes());
        buffer[8] = self.exponent as u8;
        Ok(())
    }

    /// Decodes from a buffer.
    ///
    /// # Errors
    ///
    /// Returns error if buffer is too small.
    pub fn decode(buffer: &[u8]) -> SbeResult<Self> {
        if buffer.len() < Self::SIZE {
            return Err(SbeError::BufferTooSmall {
                needed: Self::SIZE,
                available: buffer.len(),
            });
        }
        let mantissa = i64::from_le_bytes([
            buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7],
        ]);
        let exponent = buffer[8] as i8;
        Ok(Self { mantissa, exponent })
    }
}

/// Encodes a variable-length string to SBE format.
///
/// Format: u32 length + UTF-8 bytes
///
/// # Errors
///
/// Returns error if buffer is too small.
#[allow(clippy::indexing_slicing)]
pub fn encode_var_string(s: &str, buffer: &mut [u8]) -> SbeResult<usize> {
    let bytes = s.as_bytes();
    let total_size = 4 + bytes.len();
    if buffer.len() < total_size {
        return Err(SbeError::BufferTooSmall {
            needed: total_size,
            available: buffer.len(),
        });
    }
    buffer[0..4].copy_from_slice(&(bytes.len() as u32).to_le_bytes());
    buffer[4..total_size].copy_from_slice(bytes);
    Ok(total_size)
}

/// Decodes a variable-length string from SBE format.
///
/// # Errors
///
/// Returns error if buffer is too small or string is invalid UTF-8.
#[allow(clippy::indexing_slicing)]
pub fn decode_var_string(buffer: &[u8]) -> SbeResult<(String, usize)> {
    if buffer.len() < 4 {
        return Err(SbeError::BufferTooSmall {
            needed: 4,
            available: buffer.len(),
        });
    }
    let len = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]) as usize;
    let total_size = 4 + len;
    if buffer.len() < total_size {
        return Err(SbeError::BufferTooSmall {
            needed: total_size,
            available: buffer.len(),
        });
    }
    let s = String::from_utf8(buffer[4..total_size].to_vec())
        .map_err(|e| SbeError::InvalidString(e.to_string()))?;
    Ok((s, total_size))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    mod sbe_uuid {
        use super::*;

        #[test]
        fn roundtrip() {
            let uuid = Uuid::new_v4();
            let sbe = SbeUuid::from_uuid(uuid);
            let back = sbe.to_uuid();
            assert_eq!(uuid, back);
        }

        #[test]
        fn encode_decode() {
            let uuid = Uuid::new_v4();
            let sbe = SbeUuid::from_uuid(uuid);
            let mut buffer = [0u8; 16];
            sbe.encode(&mut buffer).unwrap();
            let decoded = SbeUuid::decode(&buffer).unwrap();
            assert_eq!(sbe, decoded);
        }
    }

    mod sbe_decimal {
        use super::*;

        #[test]
        fn roundtrip_positive() {
            let decimal = Decimal::new(12345, 2); // 123.45
            let sbe = SbeDecimal::from_decimal(decimal);
            let back = sbe.to_decimal().unwrap();
            assert_eq!(decimal, back);
        }

        #[test]
        fn roundtrip_zero() {
            let decimal = Decimal::ZERO;
            let sbe = SbeDecimal::from_decimal(decimal);
            let back = sbe.to_decimal().unwrap();
            assert_eq!(decimal, back);
        }

        #[test]
        fn encode_decode() {
            let decimal = Decimal::new(99999, 4);
            let sbe = SbeDecimal::from_decimal(decimal);
            let mut buffer = [0u8; 9];
            sbe.encode(&mut buffer).unwrap();
            let decoded = SbeDecimal::decode(&buffer).unwrap();
            assert_eq!(sbe, decoded);
        }
    }

    mod var_string {
        use super::*;

        #[test]
        fn roundtrip_empty() {
            let s = "";
            let mut buffer = [0u8; 100];
            let size = encode_var_string(s, &mut buffer).unwrap();
            assert_eq!(size, 4);
            let (decoded, decoded_size) = decode_var_string(&buffer).unwrap();
            assert_eq!(decoded, s);
            assert_eq!(decoded_size, size);
        }

        #[test]
        fn roundtrip_ascii() {
            let s = "hello-world";
            let mut buffer = [0u8; 100];
            let size = encode_var_string(s, &mut buffer).unwrap();
            let (decoded, decoded_size) = decode_var_string(&buffer).unwrap();
            assert_eq!(decoded, s);
            assert_eq!(decoded_size, size);
        }

        #[test]
        fn roundtrip_unicode() {
            let s = "héllo-wörld-日本語";
            let mut buffer = [0u8; 100];
            let size = encode_var_string(s, &mut buffer).unwrap();
            let (decoded, decoded_size) = decode_var_string(&buffer).unwrap();
            assert_eq!(decoded, s);
            assert_eq!(decoded_size, size);
        }
    }
}
