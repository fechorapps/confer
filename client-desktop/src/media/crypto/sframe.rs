use std::collections::HashMap;
use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes128Gcm, Aes256Gcm, Key, Nonce};
use hkdf::Hkdf;
use sha2::Sha256;
use thiserror::Error;

/// Supported SFrame AEAD Cipher Suites.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CipherSuite {
    /// AES-128-GCM with 128-bit key, 96-bit IV, and 128-bit authentication tag.
    Aes128Gcm,
    /// AES-256-GCM with 256-bit key, 96-bit IV, and 128-bit authentication tag.
    Aes256Gcm,
}

impl CipherSuite {
    /// Returns the required key length in bytes for this cipher suite.
    pub const fn key_len(&self) -> usize {
        match self {
            Self::Aes128Gcm => 16,
            Self::Aes256Gcm => 32,
        }
    }

    /// Returns the base IV length in bytes (standard 96-bit / 12-byte IV for GCM).
    pub const fn iv_len(&self) -> usize {
        12
    }

    /// Returns the authentication tag length in bytes (standard 128-bit / 16-byte tag for GCM).
    pub const fn tag_len(&self) -> usize {
        16
    }
}

/// SFrame Engine errors.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum SFrameError {
    #[error("Buffer too short: expected at least {expected} bytes, got {actual}")]
    BufferTooShort { expected: usize, actual: usize },

    #[error("Invalid SFrame header: {0}")]
    InvalidHeader(String),

    #[error("SFrame key not found for Key ID: {0}")]
    KeyNotFound(u64),

    #[error("Authentication failed: invalid tag or ciphertext corrupted")]
    AuthenticationFailed,

    #[error("Replay attack detected: frame counter {counter} already seen or outside window (max: {max_counter})")]
    ReplayDetected { counter: u64, max_counter: u64 },

    #[error("Invalid key length: expected {expected} bytes, got {actual}")]
    InvalidKeyLength { expected: usize, actual: usize },

    #[error("Invalid IV length: expected {expected} bytes, got {actual}")]
    InvalidIvLength { expected: usize, actual: usize },

    #[error("Frame counter exhausted: reached maximum counter value")]
    CounterExhausted,

    #[error("Key ratcheting failed: {0}")]
    RatchetFailed(String),

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
}

/// SFrame Header as defined in RFC / draft-ietf-sframe-enc §4.2.
///
/// Encodes the Key ID (`KID`) and the monotonically increasing frame Counter (`CTR`).
/// The serialized SFrame Header is included as Additional Authenticated Data (AAD)
/// in the AEAD encryption to prevent header tampering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SFrameHeader {
    /// Key Identifier corresponding to the sender's current epoch key.
    pub key_id: u64,
    /// Monotonically increasing frame counter for nonce computation and replay protection.
    pub counter: u64,
}

impl SFrameHeader {
    /// Creates a new SFrame header.
    pub const fn new(key_id: u64, counter: u64) -> Self {
        Self { key_id, counter }
    }

    /// Computes the minimal number of bytes required to encode the counter value.
    pub fn counter_byte_len(counter: u64) -> usize {
        if counter == 0 {
            1
        } else {
            let bits = 64 - counter.leading_zeros() as usize;
            bits.div_ceil(8).min(8)
        }
    }

    /// Computes the minimal number of bytes required to encode the Key ID value.
    pub fn key_id_byte_len(key_id: u64) -> usize {
        if key_id <= 15 {
            // Short Key IDs (0..=15) are packed into the config byte directly.
            0
        } else {
            let bits = 64 - key_id.leading_zeros() as usize;
            bits.div_ceil(8).min(8)
        }
    }

    /// Computes the serialized header size in bytes.
    pub fn serialized_size(&self) -> usize {
        let ctr_len = Self::counter_byte_len(self.counter);
        if self.key_id <= 15 {
            1 + ctr_len
        } else {
            let kid_len = Self::key_id_byte_len(self.key_id);
            1 + kid_len + ctr_len
        }
    }

    /// Serializes this SFrame header into the provided byte vector.
    pub fn serialize_into(&self, buf: &mut Vec<u8>) {
        let ctr_len = Self::counter_byte_len(self.counter);
        let ctr_len_field = (ctr_len & 0x07) as u8;

        if self.key_id <= 15 {
            // S=0 (Short Key ID, 0..15)
            // Byte 0: [S=0 (1b) | CTR_LEN (3b) | KID (4b)]
            let byte0 = (ctr_len_field << 4) | ((self.key_id as u8) & 0x0F);
            buf.push(byte0);
        } else {
            // S=1 (Extended Key ID)
            // K field is (kid_len - 1)
            let kid_len = Self::key_id_byte_len(self.key_id);
            let k_field = ((kid_len.saturating_sub(1)) & 0x0F) as u8;
            // Byte 0: [S=1 (1b) | CTR_LEN (3b) | K (4b)]
            let byte0 = 0x80 | (ctr_len_field << 4) | k_field;
            buf.push(byte0);

            // Append KID bytes in big-endian
            let kid_be = self.key_id.to_be_bytes();
            let start = 8 - kid_len;
            buf.extend_from_slice(&kid_be[start..]);
        }

        // Append Counter bytes in big-endian
        let ctr_be = self.counter.to_be_bytes();
        let start = 8 - ctr_len;
        buf.extend_from_slice(&ctr_be[start..]);
    }

    /// Serializes this SFrame header into a new byte vector.
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.serialized_size());
        self.serialize_into(&mut buf);
        buf
    }

    /// Parses an SFrame header from the start of a byte buffer.
    /// Returns `Ok((header, bytes_consumed))` on success.
    pub fn parse(buf: &[u8]) -> Result<(Self, usize), SFrameError> {
        if buf.is_empty() {
            return Err(SFrameError::BufferTooShort {
                expected: 1,
                actual: 0,
            });
        }

        let byte0 = buf[0];
        let is_extended_kid = (byte0 & 0x80) != 0;
        let mut ctr_len = ((byte0 >> 4) & 0x07) as usize;
        if ctr_len == 0 {
            ctr_len = 8; // If 0 in 3-bit field, represents 8 bytes
        }

        let mut offset = 1;

        let key_id = if !is_extended_kid {
            (byte0 & 0x0F) as u64
        } else {
            let kid_len = ((byte0 & 0x0F) as usize) + 1;
            if buf.len() < offset + kid_len {
                return Err(SFrameError::BufferTooShort {
                    expected: offset + kid_len,
                    actual: buf.len(),
                });
            }

            let mut kid_val = 0u64;
            for &b in &buf[offset..offset + kid_len] {
                kid_val = (kid_val << 8) | (b as u64);
            }
            offset += kid_len;
            kid_val
        };

        if buf.len() < offset + ctr_len {
            return Err(SFrameError::BufferTooShort {
                expected: offset + ctr_len,
                actual: buf.len(),
            });
        }

        let mut counter = 0u64;
        for &b in &buf[offset..offset + ctr_len] {
            counter = (counter << 8) | (b as u64);
        }
        offset += ctr_len;

        Ok((Self { key_id, counter }, offset))
    }
}

/// SFrame Key representation containing AEAD key material, cipher suite, and Base IV.
#[derive(Clone, PartialEq, Eq)]
pub struct SFrameKey {
    pub cipher_suite: CipherSuite,
    pub key: Vec<u8>,
    pub base_iv: [u8; 12],
}

impl std::fmt::Debug for SFrameKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SFrameKey")
            .field("cipher_suite", &self.cipher_suite)
            .field("key_len", &self.key.len())
            .field("base_iv", &self.base_iv)
            .finish()
    }
}

impl SFrameKey {
    /// Creates a new `SFrameKey` directly from raw key bytes and 12-byte Base IV.
    pub fn new(cipher_suite: CipherSuite, key_bytes: &[u8], base_iv: [u8; 12]) -> Result<Self, SFrameError> {
        if key_bytes.len() != cipher_suite.key_len() {
            return Err(SFrameError::InvalidKeyLength {
                expected: cipher_suite.key_len(),
                actual: key_bytes.len(),
            });
        }
        Ok(Self {
            cipher_suite,
            key: key_bytes.to_vec(),
            base_iv,
        })
    }

    /// Computes the unique 12-byte AEAD Nonce for a given frame counter:
    /// `Nonce = BaseIV XOR Counter_12bytes`.
    #[inline]
    pub fn compute_nonce(&self, counter: u64) -> [u8; 12] {
        let mut nonce = self.base_iv;
        let ctr_be = counter.to_be_bytes(); // 8 bytes
        // XOR the lower 8 bytes (offset 4..12) with counter
        for i in 0..8 {
            nonce[4 + i] ^= ctr_be[i];
        }
        nonce
    }

    /// Derives an `SFrameKey` from master secret material (e.g. room password or MLS epoch secret)
    /// using HKDF-SHA256.
    pub fn derive_from_secret(
        cipher_suite: CipherSuite,
        secret: &[u8],
        salt: Option<&[u8]>,
    ) -> Result<Self, SFrameError> {
        let default_salt = b"confer-sframe-v1-key-derivation-salt";
        let salt_bytes = salt.unwrap_or(default_salt);

        let hk = Hkdf::<Sha256>::new(Some(salt_bytes), secret);

        let mut key = vec![0u8; cipher_suite.key_len()];
        hk.expand(b"sframe-aead-key", &mut key)
            .map_err(|e| SFrameError::RatchetFailed(format!("HKDF key expansion failed: {:?}", e)))?;

        let mut base_iv = [0u8; 12];
        hk.expand(b"sframe-base-iv", &mut base_iv)
            .map_err(|e| SFrameError::RatchetFailed(format!("HKDF base IV expansion failed: {:?}", e)))?;

        Ok(Self {
            cipher_suite,
            key,
            base_iv,
        })
    }

    /// Ratchets this key forward to produce the next epoch key using HKDF-SHA256.
    /// Provides forward secrecy across meeting epochs or participant transitions.
    pub fn ratchet(&self, next_epoch: u64) -> Result<Self, SFrameError> {
        let epoch_salt = format!("confer-sframe-epoch-ratchet-{}", next_epoch);
        let hk = Hkdf::<Sha256>::new(Some(epoch_salt.as_bytes()), &self.key);

        let mut next_key = vec![0u8; self.cipher_suite.key_len()];
        let key_info = format!("sframe-ratchet-key-epoch-{}", next_epoch);
        hk.expand(key_info.as_bytes(), &mut next_key)
            .map_err(|e| SFrameError::RatchetFailed(format!("HKDF ratchet key expansion failed: {:?}", e)))?;

        let mut next_iv = [0u8; 12];
        let iv_info = format!("sframe-ratchet-iv-epoch-{}", next_epoch);
        hk.expand(iv_info.as_bytes(), &mut next_iv)
            .map_err(|e| SFrameError::RatchetFailed(format!("HKDF ratchet IV expansion failed: {:?}", e)))?;

        // Mix the previous Base IV with the newly derived IV
        for (dst, src) in next_iv.iter_mut().zip(self.base_iv.iter()) {
            *dst ^= src;
        }

        Ok(Self {
            cipher_suite: self.cipher_suite,
            key: next_key,
            base_iv: next_iv,
        })
    }

    /// Encrypts raw plaintext frame payload with SFrame header as AAD.
    pub fn encrypt(&self, header: &SFrameHeader, plaintext: &[u8]) -> Result<Vec<u8>, SFrameError> {
        let header_bytes = header.serialize();
        let nonce_bytes = self.compute_nonce(header.counter);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let payload = Payload {
            msg: plaintext,
            aad: &header_bytes,
        };

        let ciphertext_with_tag = match self.cipher_suite {
            CipherSuite::Aes128Gcm => {
                let cipher = Aes128Gcm::new(Key::<Aes128Gcm>::from_slice(&self.key));
                cipher
                    .encrypt(nonce, payload)
                    .map_err(|e| SFrameError::EncryptionFailed(format!("AES-128-GCM error: {:?}", e)))?
            }
            CipherSuite::Aes256Gcm => {
                let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
                cipher
                    .encrypt(nonce, payload)
                    .map_err(|e| SFrameError::EncryptionFailed(format!("AES-256-GCM error: {:?}", e)))?
            }
        };

        let mut output = Vec::with_capacity(header_bytes.len() + ciphertext_with_tag.len());
        output.extend_from_slice(&header_bytes);
        output.extend_from_slice(&ciphertext_with_tag);
        Ok(output)
    }

    /// Decrypts ciphertext given the parsed SFrame header and raw header bytes as AAD.
    pub fn decrypt(
        &self,
        header: &SFrameHeader,
        header_bytes: &[u8],
        ciphertext_and_tag: &[u8],
    ) -> Result<Vec<u8>, SFrameError> {
        if ciphertext_and_tag.len() < self.cipher_suite.tag_len() {
            return Err(SFrameError::BufferTooShort {
                expected: self.cipher_suite.tag_len(),
                actual: ciphertext_and_tag.len(),
            });
        }

        let nonce_bytes = self.compute_nonce(header.counter);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let payload = Payload {
            msg: ciphertext_and_tag,
            aad: header_bytes,
        };

        let plaintext = match self.cipher_suite {
            CipherSuite::Aes128Gcm => {
                let cipher = Aes128Gcm::new(Key::<Aes128Gcm>::from_slice(&self.key));
                cipher
                    .decrypt(nonce, payload)
                    .map_err(|_| SFrameError::AuthenticationFailed)?
            }
            CipherSuite::Aes256Gcm => {
                let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
                cipher
                    .decrypt(nonce, payload)
                    .map_err(|_| SFrameError::AuthenticationFailed)?
            }
        };

        Ok(plaintext)
    }
}

/// 128-bit Sliding Window Anti-Replay Protection Filter (RFC 6479).
///
/// Tracks the highest received counter and maintains a 128-packet bitmask
/// to detect and reject replayed media frames while allowing out-of-order delivery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayFilter {
    window_size: u64,
    max_counter: u64,
    bitmap_low: u64,
    bitmap_high: u64,
    initialized: bool,
}

impl Default for ReplayFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplayFilter {
    pub const DEFAULT_WINDOW_SIZE: u64 = 128;

    pub fn new() -> Self {
        Self {
            window_size: Self::DEFAULT_WINDOW_SIZE,
            max_counter: 0,
            bitmap_low: 0,
            bitmap_high: 0,
            initialized: false,
        }
    }

    /// Checks whether the counter is valid (not replayed and not too old) without updating state.
    pub fn check(&self, counter: u64) -> Result<(), SFrameError> {
        if !self.initialized {
            return Ok(());
        }

        if counter > self.max_counter {
            return Ok(());
        }

        let diff = self.max_counter - counter;
        if diff >= self.window_size {
            return Err(SFrameError::ReplayDetected {
                counter,
                max_counter: self.max_counter,
            });
        }

        let is_set = if diff < 64 {
            (self.bitmap_low & (1u64 << diff)) != 0
        } else {
            (self.bitmap_high & (1u64 << (diff - 64))) != 0
        };

        if is_set {
            Err(SFrameError::ReplayDetected {
                counter,
                max_counter: self.max_counter,
            })
        } else {
            Ok(())
        }
    }

    /// Updates the sliding window bitmap with the verified counter.
    pub fn update(&mut self, counter: u64) {
        if !self.initialized {
            self.initialized = true;
            self.max_counter = counter;
            self.bitmap_low = 1;
            self.bitmap_high = 0;
            return;
        }

        if counter > self.max_counter {
            let diff = counter - self.max_counter;
            if diff >= self.window_size {
                self.bitmap_low = 1;
                self.bitmap_high = 0;
            } else if diff >= 64 {
                let shift = diff - 64;
                self.bitmap_high = (self.bitmap_low << shift) | (self.bitmap_high >> (64 - shift));
                self.bitmap_low = 1;
            } else {
                let carry = self.bitmap_low >> (64 - diff);
                self.bitmap_high = (self.bitmap_high << diff) | carry;
                self.bitmap_low = (self.bitmap_low << diff) | 1;
            }
            self.max_counter = counter;
        } else {
            let diff = self.max_counter - counter;
            if diff < 64 {
                self.bitmap_low |= 1u64 << diff;
            } else if diff < 128 {
                self.bitmap_high |= 1u64 << (diff - 64);
            }
        }
    }

    /// Checks and updates the replay filter atomically.
    pub fn check_and_update(&mut self, counter: u64) -> Result<(), SFrameError> {
        self.check(counter)?;
        self.update(counter);
        Ok(())
    }
}

/// SFrame End-to-End Encryption Engine.
///
/// Manages sender/receiver keys across epochs, auto-increments transmission frame counters,
/// and applies sliding-window replay protection.
#[derive(Debug, Clone)]
pub struct SFrameEngine {
    pub cipher_suite: CipherSuite,
    keys: HashMap<u64, SFrameKey>,
    active_key_id: u64,
    tx_counter: u64,
    replay_filters: HashMap<u64, ReplayFilter>,
}

impl SFrameEngine {
    /// Creates a new SFrameEngine with the specified cipher suite.
    pub fn new(cipher_suite: CipherSuite) -> Self {
        Self {
            cipher_suite,
            keys: HashMap::new(),
            active_key_id: 0,
            tx_counter: 0,
            replay_filters: HashMap::new(),
        }
    }

    /// Creates an engine initialized with a master secret (passphrase or MLS epoch secret)
    /// assigned to Key ID 0.
    pub fn from_shared_secret(cipher_suite: CipherSuite, secret: &[u8]) -> Result<Self, SFrameError> {
        let key = SFrameKey::derive_from_secret(cipher_suite, secret, None)?;
        let mut engine = Self::new(cipher_suite);
        engine.set_key(0, key);
        engine.set_active_key_id(0);
        Ok(engine)
    }

    /// Registers a key for a given Key ID / epoch.
    pub fn set_key(&mut self, key_id: u64, key: SFrameKey) {
        self.keys.insert(key_id, key);
    }

    /// Sets the active Key ID used for outgoing frame encryptions.
    pub fn set_active_key_id(&mut self, key_id: u64) {
        self.active_key_id = key_id;
    }

    /// Returns the currently active Key ID.
    pub fn active_key_id(&self) -> u64 {
        self.active_key_id
    }

    /// Retrieves the key for a specific Key ID if present.
    pub fn get_key(&self, key_id: u64) -> Option<&SFrameKey> {
        self.keys.get(&key_id)
    }

    /// Returns the current transmit counter value.
    pub fn current_counter(&self) -> u64 {
        self.tx_counter
    }

    /// Increments and returns the next transmit counter value.
    pub fn next_counter(&mut self) -> Result<u64, SFrameError> {
        if self.tx_counter == u64::MAX {
            return Err(SFrameError::CounterExhausted);
        }
        let ctr = self.tx_counter;
        self.tx_counter += 1;
        Ok(ctr)
    }

    /// Ratchets the active key to a new epoch, sets it as active key, and resets the counter.
    pub fn rotate_epoch(&mut self, next_epoch: u64) -> Result<u64, SFrameError> {
        let current_key = self
            .keys
            .get(&self.active_key_id)
            .ok_or(SFrameError::KeyNotFound(self.active_key_id))?;

        let next_key = current_key.ratchet(next_epoch)?;
        self.set_key(next_epoch, next_key);
        self.set_active_key_id(next_epoch);
        self.tx_counter = 0;
        Ok(next_epoch)
    }

    /// Encrypts raw media payload using the active key and auto-incremented counter.
    pub fn encrypt_frame(&mut self, raw_frame_payload: &[u8]) -> Result<Vec<u8>, SFrameError> {
        let counter = self.next_counter()?;
        self.encrypt_frame_with(self.active_key_id, raw_frame_payload, counter)
    }

    /// Encrypts raw media payload with an explicit Key ID and counter.
    pub fn encrypt_frame_with(
        &self,
        key_id: u64,
        raw_frame_payload: &[u8],
        counter: u64,
    ) -> Result<Vec<u8>, SFrameError> {
        let key = self.keys.get(&key_id).ok_or(SFrameError::KeyNotFound(key_id))?;
        let header = SFrameHeader::new(key_id, counter);
        key.encrypt(&header, raw_frame_payload)
    }

    /// Decrypts an incoming SFrame-encrypted payload.
    ///
    /// Parses the header, verifies replay protection, verifies the AEAD auth tag,
    /// decrypts the ciphertext, and updates replay window on success.
    pub fn decrypt_frame(&mut self, encrypted_payload: &[u8]) -> Result<Vec<u8>, SFrameError> {
        let (header, header_len) = SFrameHeader::parse(encrypted_payload)?;
        let key = self
            .keys
            .get(&header.key_id)
            .ok_or(SFrameError::KeyNotFound(header.key_id))?;

        // Replay check before decryption
        let filter = self
            .replay_filters
            .entry(header.key_id)
            .or_default();
        filter.check(header.counter)?;

        let header_bytes = &encrypted_payload[..header_len];
        let ciphertext_and_tag = &encrypted_payload[header_len..];

        let plaintext = key.decrypt(&header, header_bytes, ciphertext_and_tag)?;

        // Update replay filter only after successful authentication
        filter.update(header.counter);

        Ok(plaintext)
    }

    /// Decrypts an incoming SFrame-encrypted payload with an expected Key ID.
    pub fn decrypt_frame_with_key_id(
        &mut self,
        key_id: u64,
        encrypted_payload: &[u8],
    ) -> Result<Vec<u8>, SFrameError> {
        let (header, header_len) = SFrameHeader::parse(encrypted_payload)?;
        if header.key_id != key_id {
            return Err(SFrameError::InvalidHeader(format!(
                "Key ID mismatch in header: expected {}, got {}",
                key_id, header.key_id
            )));
        }
        let key = self.keys.get(&key_id).ok_or(SFrameError::KeyNotFound(key_id))?;

        let filter = self
            .replay_filters
            .entry(key_id)
            .or_default();
        filter.check(header.counter)?;

        let header_bytes = &encrypted_payload[..header_len];
        let ciphertext_and_tag = &encrypted_payload[header_len..];

        let plaintext = key.decrypt(&header, header_bytes, ciphertext_and_tag)?;

        filter.update(header.counter);

        Ok(plaintext)
    }

    /// Decrypts an incoming frame without modifying the replay filter state (read-only inspection).
    pub fn decrypt_frame_stateless(&self, encrypted_payload: &[u8]) -> Result<(SFrameHeader, Vec<u8>), SFrameError> {
        let (header, header_len) = SFrameHeader::parse(encrypted_payload)?;
        let key = self
            .keys
            .get(&header.key_id)
            .ok_or(SFrameError::KeyNotFound(header.key_id))?;

        let header_bytes = &encrypted_payload[..header_len];
        let ciphertext_and_tag = &encrypted_payload[header_len..];

        let plaintext = key.decrypt(&header, header_bytes, ciphertext_and_tag)?;
        Ok((header, plaintext))
    }
}

/// Standalone helper to encrypt a raw frame payload given an `SFrameKey`, `key_id`, and `counter`.
pub fn encrypt_frame(
    key: &SFrameKey,
    key_id: u64,
    raw_frame_payload: &[u8],
    counter: u64,
) -> Result<Vec<u8>, SFrameError> {
    let header = SFrameHeader::new(key_id, counter);
    key.encrypt(&header, raw_frame_payload)
}

/// Standalone helper to decrypt an encrypted payload given an `SFrameKey`.
pub fn decrypt_frame(
    key: &SFrameKey,
    encrypted_payload: &[u8],
) -> Result<Vec<u8>, SFrameError> {
    let (header, header_len) = SFrameHeader::parse(encrypted_payload)?;
    let header_bytes = &encrypted_payload[..header_len];
    let ciphertext_and_tag = &encrypted_payload[header_len..];
    key.decrypt(&header, header_bytes, ciphertext_and_tag)
}

/// Standalone helper to ratchet an `SFrameKey` forward to produce the next epoch key.
pub fn ratchet_key(current_key: &SFrameKey, next_epoch: u64) -> Result<SFrameKey, SFrameError> {
    current_key.ratchet(next_epoch)
}

/// Standalone helper to rotate the epoch key in an `SFrameEngine`.
pub fn rotate_epoch_key(engine: &mut SFrameEngine, next_epoch: u64) -> Result<u64, SFrameError> {
    engine.rotate_epoch(next_epoch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sframe_header_serialization_and_parsing_short_kid() {
        // Short Key ID (0..15)
        for kid in [0u64, 1, 7, 15] {
            for counter in [0u64, 1, 255, 65535, 16777215, 0x1234567890ABCDEF] {
                let header = SFrameHeader::new(kid, counter);
                let serialized = header.serialize();
                assert_eq!(serialized.len(), header.serialized_size());

                let (parsed, consumed) = SFrameHeader::parse(&serialized).expect("Parse short KID header");
                assert_eq!(parsed, header);
                assert_eq!(consumed, serialized.len());
            }
        }
    }

    #[test]
    fn test_sframe_header_serialization_and_parsing_extended_kid() {
        // Extended Key ID (> 15)
        for kid in [16u64, 100, 1000, 0xABCDEF, 0xFEDCBA9876543210] {
            for counter in [0u64, 1, 1024, 0x11223344] {
                let header = SFrameHeader::new(kid, counter);
                let serialized = header.serialize();
                assert_eq!(serialized.len(), header.serialized_size());

                let (parsed, consumed) = SFrameHeader::parse(&serialized).expect("Parse extended KID header");
                assert_eq!(parsed, header);
                assert_eq!(consumed, serialized.len());
            }
        }
    }

    #[test]
    fn test_aes_128_gcm_encryption_decryption_roundtrip() {
        let mut engine = SFrameEngine::new(CipherSuite::Aes128Gcm);
        let raw_key = [0x42u8; 16];
        let base_iv = [0x11u8; 12];
        let sframe_key = SFrameKey::new(CipherSuite::Aes128Gcm, &raw_key, base_iv).unwrap();

        engine.set_key(1, sframe_key);
        engine.set_active_key_id(1);

        let plaintext = b"Hello, Confer E2EE Secure SFrame WebRTC!";
        let encrypted = engine.encrypt_frame(plaintext).expect("Encryption failed");

        assert!(encrypted.len() > plaintext.len());

        let mut rx_engine = SFrameEngine::new(CipherSuite::Aes128Gcm);
        let rx_key = SFrameKey::new(CipherSuite::Aes128Gcm, &raw_key, base_iv).unwrap();
        rx_engine.set_key(1, rx_key);

        let decrypted = rx_engine.decrypt_frame(&encrypted).expect("Decryption failed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_aes_256_gcm_encryption_decryption_roundtrip() {
        let mut engine = SFrameEngine::new(CipherSuite::Aes256Gcm);
        let raw_key = [0x99u8; 32];
        let base_iv = [0x22u8; 12];
        let sframe_key = SFrameKey::new(CipherSuite::Aes256Gcm, &raw_key, base_iv).unwrap();

        engine.set_key(5, sframe_key);
        engine.set_active_key_id(5);

        let plaintext = b"Top secret ultra-high definition VP8 video keyframe byte stream";
        let encrypted = engine.encrypt_frame(plaintext).expect("Encryption failed");

        let mut rx_engine = SFrameEngine::new(CipherSuite::Aes256Gcm);
        let rx_key = SFrameKey::new(CipherSuite::Aes256Gcm, &raw_key, base_iv).unwrap();
        rx_engine.set_key(5, rx_key);

        let decrypted = rx_engine.decrypt_frame(&encrypted).expect("Decryption failed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_corrupted_ciphertext_fails_auth_tag() {
        let mut tx_engine = SFrameEngine::new(CipherSuite::Aes128Gcm);
        let raw_key = [0x55u8; 16];
        let base_iv = [0x77u8; 12];
        let key = SFrameKey::new(CipherSuite::Aes128Gcm, &raw_key, base_iv).unwrap();
        tx_engine.set_key(2, key.clone());
        tx_engine.set_active_key_id(2);

        let plaintext = b"Sensitive in-call audio payload";
        let mut encrypted = tx_engine.encrypt_frame(plaintext).unwrap();

        // Corrupt a byte in the ciphertext body
        let last_idx = encrypted.len() - 1;
        encrypted[last_idx] ^= 0xFF;

        let mut rx_engine = SFrameEngine::new(CipherSuite::Aes128Gcm);
        rx_engine.set_key(2, key);

        let result = rx_engine.decrypt_frame(&encrypted);
        assert_eq!(result, Err(SFrameError::AuthenticationFailed));
    }

    #[test]
    fn test_tampered_header_fails_aad_auth() {
        let mut tx_engine = SFrameEngine::new(CipherSuite::Aes128Gcm);
        let raw_key = [0x66u8; 16];
        let base_iv = [0x88u8; 12];
        let key = SFrameKey::new(CipherSuite::Aes128Gcm, &raw_key, base_iv).unwrap();
        tx_engine.set_key(1, key.clone());
        tx_engine.set_active_key_id(1);

        let plaintext = b"Header authenticated data test";
        let mut encrypted = tx_engine.encrypt_frame(plaintext).unwrap();

        // Tamper with Byte 0 of SFrame header
        encrypted[0] ^= 0x01;

        let mut rx_engine = SFrameEngine::new(CipherSuite::Aes128Gcm);
        rx_engine.set_key(1, key);

        let result = rx_engine.decrypt_frame(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_replay_protection_rejects_duplicates_and_old_counters() {
        let mut tx_engine = SFrameEngine::new(CipherSuite::Aes128Gcm);
        let raw_key = [0x33u8; 16];
        let base_iv = [0x44u8; 12];
        let key = SFrameKey::new(CipherSuite::Aes128Gcm, &raw_key, base_iv).unwrap();
        tx_engine.set_key(1, key.clone());
        tx_engine.set_active_key_id(1);

        let mut rx_engine = SFrameEngine::new(CipherSuite::Aes128Gcm);
        rx_engine.set_key(1, key);

        let p1 = tx_engine.encrypt_frame(b"Frame 0").unwrap();
        let p2 = tx_engine.encrypt_frame(b"Frame 1").unwrap();
        let p3 = tx_engine.encrypt_frame(b"Frame 2").unwrap();

        // Normal in-order receipt
        assert_eq!(rx_engine.decrypt_frame(&p1).unwrap(), b"Frame 0");
        assert_eq!(rx_engine.decrypt_frame(&p2).unwrap(), b"Frame 1");

        // Replay of p1 should fail
        let replay_res = rx_engine.decrypt_frame(&p1);
        assert!(matches!(replay_res, Err(SFrameError::ReplayDetected { .. })));

        // p3 succeeds
        assert_eq!(rx_engine.decrypt_frame(&p3).unwrap(), b"Frame 2");

        // Replay of p2 or p3 fails
        assert!(matches!(rx_engine.decrypt_frame(&p2), Err(SFrameError::ReplayDetected { .. })));
        assert!(matches!(rx_engine.decrypt_frame(&p3), Err(SFrameError::ReplayDetected { .. })));
    }

    #[test]
    fn test_key_ratcheting_and_epoch_rotation() {
        let mut tx_engine = SFrameEngine::from_shared_secret(CipherSuite::Aes128Gcm, b"room_passphrase_secret_123").unwrap();
        let mut rx_engine = SFrameEngine::from_shared_secret(CipherSuite::Aes128Gcm, b"room_passphrase_secret_123").unwrap();

        let frame_epoch0 = tx_engine.encrypt_frame(b"Message in Epoch 0").unwrap();
        assert_eq!(rx_engine.decrypt_frame(&frame_epoch0).unwrap(), b"Message in Epoch 0");

        // Rotate to Epoch 1
        tx_engine.rotate_epoch(1).unwrap();
        rx_engine.rotate_epoch(1).unwrap();

        let frame_epoch1 = tx_engine.encrypt_frame(b"Message in Epoch 1").unwrap();
        assert_eq!(rx_engine.decrypt_frame(&frame_epoch1).unwrap(), b"Message in Epoch 1");

        // Rotate to Epoch 2
        tx_engine.rotate_epoch(2).unwrap();
        rx_engine.rotate_epoch(2).unwrap();

        let frame_epoch2 = tx_engine.encrypt_frame(b"Message in Epoch 2").unwrap();
        assert_eq!(rx_engine.decrypt_frame(&frame_epoch2).unwrap(), b"Message in Epoch 2");
    }

    #[test]
    fn test_unknown_key_id_rejected() {
        let mut tx_engine = SFrameEngine::from_shared_secret(CipherSuite::Aes128Gcm, b"secret_A").unwrap();
        tx_engine.set_active_key_id(99); // No key for 99

        let err = tx_engine.encrypt_frame(b"Test").unwrap_err();
        assert_eq!(err, SFrameError::KeyNotFound(99));

        let mut rx_engine = SFrameEngine::new(CipherSuite::Aes128Gcm);
        let key = SFrameKey::new(CipherSuite::Aes128Gcm, &[0x11; 16], [0x22; 12]).unwrap();
        tx_engine.set_key(10, key);
        tx_engine.set_active_key_id(10);
        let encrypted = tx_engine.encrypt_frame(b"Valid cipher").unwrap();

        // rx_engine does not have key 10
        let rx_err = rx_engine.decrypt_frame(&encrypted).unwrap_err();
        assert_eq!(rx_err, SFrameError::KeyNotFound(10));
    }
}
