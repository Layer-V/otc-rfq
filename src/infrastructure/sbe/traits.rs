//! # SBE Encoding/Decoding Traits
//!
//! Traits for SBE binary serialization.

use super::error::SbeResult;

/// Trait for types that can be encoded to SBE binary format.
pub trait SbeEncode {
    /// Returns the encoded size in bytes (including header).
    #[must_use]
    fn encoded_size(&self) -> usize;

    /// Encodes the message to a byte buffer.
    ///
    /// # Errors
    ///
    /// Returns `SbeError::BufferTooSmall` if the buffer is too small.
    fn encode(&self, buffer: &mut [u8]) -> SbeResult<usize>;

    /// Encodes the message to a new Vec.
    ///
    /// # Errors
    ///
    /// Returns an error if encoding fails.
    fn encode_to_vec(&self) -> SbeResult<Vec<u8>> {
        let size = self.encoded_size();
        let mut buffer = vec![0u8; size];
        self.encode(&mut buffer)?;
        Ok(buffer)
    }
}

/// Trait for types that can be decoded from SBE binary format.
pub trait SbeDecode: Sized {
    /// Decodes a message from a byte buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is invalid or too small.
    fn decode(buffer: &[u8]) -> SbeResult<Self>;
}
