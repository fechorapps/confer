use egui::ColorImage;
use thiserror::Error;

/// RTP / Media Transport error types.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum RtpError {
    #[error("RTP packet buffer too short: expected at least {expected} bytes, got {actual}")]
    BufferTooShort { expected: usize, actual: usize },

    #[error("Unsupported RTP version: {0} (expected 2)")]
    InvalidVersion(u8),

    #[error("Invalid RTP extension header length: {0}")]
    InvalidExtensionLength(usize),

    #[error("Invalid VP8 payload descriptor: {0}")]
    InvalidVp8Descriptor(String),

    #[error("Missing RTP payload")]
    EmptyPayload,

    #[error("VP8 packet fragmentation sequence gap: expected sequence {expected}, got {actual}")]
    SequenceGap { expected: u16, actual: u16 },

    #[error("Incomplete VP8 video frame")]
    IncompleteFrame,
}

/// Standard RTP Fixed Header as defined in RFC 3550 §5.1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtpHeader {
    /// RTP version (must be 2).
    pub version: u8,
    /// Padding flag indicating whether padding octets exist at the end of the packet.
    pub padding: bool,
    /// Extension flag indicating whether a header extension is present.
    pub extension: bool,
    /// CSRC count (0 to 15).
    pub csrc_count: u8,
    /// Marker bit (e.g., end of video frame in VP8 or talkspurt start in audio).
    pub marker: bool,
    /// Payload type (e.g., 96 for VP8 video, 111 for Opus audio).
    pub payload_type: u8,
    /// Sequence number (monotonically increments by 1 for each RTP packet).
    pub sequence_number: u16,
    /// Timestamp (sampling clock: 90 kHz for video, 48 kHz for Opus audio).
    pub timestamp: u32,
    /// Synchronization source identifier (SSRC).
    pub ssrc: u32,
    /// Contributing source identifiers (CSRC list).
    pub csrc_list: Vec<u32>,
    /// Extension profile ID (e.g., 0xBEDE for RFC 5285 / RFC 8285 one-byte header extensions).
    pub extension_profile: Option<u16>,
    /// Raw extension data bytes (must be a multiple of 4 bytes when serialized).
    pub extension_data: Option<Vec<u8>>,
}

impl Default for RtpHeader {
    fn default() -> Self {
        Self {
            version: 2,
            padding: false,
            extension: false,
            csrc_count: 0,
            marker: false,
            payload_type: 96,
            sequence_number: 0,
            timestamp: 0,
            ssrc: 0,
            csrc_list: Vec::new(),
            extension_profile: None,
            extension_data: None,
        }
    }
}

impl RtpHeader {
    pub const FIXED_HEADER_SIZE: usize = 12;
    pub const ONE_BYTE_EXT_PROFILE: u16 = 0xBEDE;
    pub const TWO_BYTE_EXT_PROFILE: u16 = 0x1000;

    /// Creates a standard new RTP header.
    pub fn new(payload_type: u8, sequence_number: u16, timestamp: u32, ssrc: u32) -> Self {
        Self {
            version: 2,
            padding: false,
            extension: false,
            csrc_count: 0,
            marker: false,
            payload_type,
            sequence_number,
            timestamp,
            ssrc,
            csrc_list: Vec::new(),
            extension_profile: None,
            extension_data: None,
        }
    }

    /// Returns the serialized header length in bytes.
    pub fn serialized_size(&self) -> usize {
        let mut size = Self::FIXED_HEADER_SIZE + (self.csrc_list.len() * 4);
        if self.extension {
            if let Some(ext) = &self.extension_data {
                size += 4 + ext.len(); // 2 bytes profile + 2 bytes length + data
            }
        }
        size
    }

    /// Serializes the RTP header into a byte vector.
    pub fn serialize_into(&self, buf: &mut Vec<u8>) {
        let p_bit = if self.padding { 0x20 } else { 0x00 };
        let x_bit = if self.extension && self.extension_data.is_some() { 0x10 } else { 0x00 };
        let cc = (self.csrc_list.len().min(15) as u8) & 0x0F;

        // Byte 0: V(2) | P(1) | X(1) | CC(4)
        let byte0 = (self.version << 6) | p_bit | x_bit | cc;
        buf.push(byte0);

        // Byte 1: M(1) | PT(7)
        let m_bit = if self.marker { 0x80 } else { 0x00 };
        let byte1 = m_bit | (self.payload_type & 0x7F);
        buf.push(byte1);

        // Bytes 2-3: Sequence number
        buf.extend_from_slice(&self.sequence_number.to_be_bytes());

        // Bytes 4-7: Timestamp
        buf.extend_from_slice(&self.timestamp.to_be_bytes());

        // Bytes 8-11: SSRC
        buf.extend_from_slice(&self.ssrc.to_be_bytes());

        // CSRC list
        for &csrc in self.csrc_list.iter().take(cc as usize) {
            buf.extend_from_slice(&csrc.to_be_bytes());
        }

        // Header Extension (RFC 3550 §5.3.1)
        if self.extension {
            if let Some(ext_data) = &self.extension_data {
                let profile = self.extension_profile.unwrap_or(Self::ONE_BYTE_EXT_PROFILE);
                buf.extend_from_slice(&profile.to_be_bytes());
                // Length is number of 32-bit words (excluding the 4-byte extension header itself)
                let length_in_words = ((ext_data.len() + 3) / 4) as u16;
                buf.extend_from_slice(&length_in_words.to_be_bytes());
                buf.extend_from_slice(ext_data);

                // Pad extension data to 32-bit boundary if needed
                let padding_needed = (length_in_words as usize * 4) - ext_data.len();
                for _ in 0..padding_needed {
                    buf.push(0);
                }
            }
        }
    }

    /// Parses an RTP header from raw packet bytes.
    pub fn parse(raw: &[u8]) -> Result<(Self, usize), RtpError> {
        if raw.len() < Self::FIXED_HEADER_SIZE {
            return Err(RtpError::BufferTooShort {
                expected: Self::FIXED_HEADER_SIZE,
                actual: raw.len(),
            });
        }

        let byte0 = raw[0];
        let version = (byte0 >> 6) & 0x03;
        if version != 2 {
            return Err(RtpError::InvalidVersion(version));
        }

        let padding = (byte0 & 0x20) != 0;
        let extension = (byte0 & 0x10) != 0;
        let csrc_count = (byte0 & 0x0F) as usize;

        let byte1 = raw[1];
        let marker = (byte1 & 0x80) != 0;
        let payload_type = byte1 & 0x7F;

        let sequence_number = u16::from_be_bytes([raw[2], raw[3]]);
        let timestamp = u32::from_be_bytes([raw[4], raw[5], raw[6], raw[7]]);
        let ssrc = u32::from_be_bytes([raw[8], raw[9], raw[10], raw[11]]);

        let mut offset = Self::FIXED_HEADER_SIZE;
        let required_csrc_len = offset + (csrc_count * 4);
        if raw.len() < required_csrc_len {
            return Err(RtpError::BufferTooShort {
                expected: required_csrc_len,
                actual: raw.len(),
            });
        }

        let mut csrc_list = Vec::with_capacity(csrc_count);
        for _ in 0..csrc_count {
            let csrc = u32::from_be_bytes([raw[offset], raw[offset + 1], raw[offset + 2], raw[offset + 3]]);
            csrc_list.push(csrc);
            offset += 4;
        }

        let mut extension_profile = None;
        let mut extension_data = None;

        if extension {
            if raw.len() < offset + 4 {
                return Err(RtpError::BufferTooShort {
                    expected: offset + 4,
                    actual: raw.len(),
                });
            }

            let profile = u16::from_be_bytes([raw[offset], raw[offset + 1]]);
            let length_in_words = u16::from_be_bytes([raw[offset + 2], raw[offset + 3]]) as usize;
            offset += 4;

            let ext_data_len = length_in_words * 4;
            if raw.len() < offset + ext_data_len {
                return Err(RtpError::BufferTooShort {
                    expected: offset + ext_data_len,
                    actual: raw.len(),
                });
            }

            extension_profile = Some(profile);
            extension_data = Some(raw[offset..offset + ext_data_len].to_vec());
            offset += ext_data_len;
        }

        Ok((
            Self {
                version,
                padding,
                extension,
                csrc_count: csrc_count as u8,
                marker,
                payload_type,
                sequence_number,
                timestamp,
                ssrc,
                csrc_list,
                extension_profile,
                extension_data,
            },
            offset,
        ))
    }
}

/// A complete RTP Packet (RFC 3550) containing header and payload bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtpPacket {
    pub header: RtpHeader,
    pub payload: Vec<u8>,
    pub padding_len: usize,
}

impl RtpPacket {
    pub fn new(header: RtpHeader, payload: Vec<u8>) -> Self {
        Self {
            header,
            payload,
            padding_len: 0,
        }
    }

    /// Serializes the entire RTP packet to a byte buffer.
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.header.serialized_size() + self.payload.len() + self.padding_len);
        let mut header = self.header.clone();
        if self.padding_len > 0 {
            header.padding = true;
        }
        header.serialize_into(&mut buf);
        buf.extend_from_slice(&self.payload);

        if self.padding_len > 0 {
            // Padding octets ending with the total padding byte count (RFC 3550 §5.1)
            for _ in 0..(self.padding_len - 1) {
                buf.push(0);
            }
            buf.push(self.padding_len as u8);
        }

        buf
    }

    /// Parses an RTP packet from a raw byte slice.
    pub fn parse(raw: &[u8]) -> Result<Self, RtpError> {
        let (header, header_size) = RtpHeader::parse(raw)?;
        let mut payload_slice = &raw[header_size..];

        let mut padding_len = 0;
        if header.padding {
            if payload_slice.is_empty() {
                return Err(RtpError::BufferTooShort {
                    expected: 1,
                    actual: 0,
                });
            }
            let pad_count = *payload_slice.last().unwrap() as usize;
            if pad_count == 0 || pad_count > payload_slice.len() {
                return Err(RtpError::InvalidExtensionLength(pad_count));
            }
            padding_len = pad_count;
            payload_slice = &payload_slice[..payload_slice.len() - pad_count];
        }

        Ok(Self {
            header,
            payload: payload_slice.to_vec(),
            padding_len,
        })
    }
}

/// One-Byte Header Extension element (RFC 5285 / RFC 8285).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OneByteHeaderExtension {
    /// Identifier (1 to 14). ID 0 is padding, ID 15 is reserved/stop.
    pub id: u8,
    /// Extension data payload (1 to 16 bytes).
    pub data: Vec<u8>,
}

impl OneByteHeaderExtension {
    pub fn new(id: u8, data: Vec<u8>) -> Self {
        Self { id, data }
    }

    /// Serializes a slice of One-Byte Header Extensions into a 32-bit aligned byte vector.
    pub fn serialize_list(extensions: &[OneByteHeaderExtension]) -> Vec<u8> {
        let mut buf = Vec::new();
        for ext in extensions {
            if ext.id == 0 || ext.id > 14 || ext.data.is_empty() || ext.data.len() > 16 {
                continue;
            }
            let len_minus_one = (ext.data.len() - 1) as u8;
            let header_byte = (ext.id << 4) | (len_minus_one & 0x0F);
            buf.push(header_byte);
            buf.extend_from_slice(&ext.data);
        }

        // 32-bit boundary alignment with zero padding bytes
        while buf.len() % 4 != 0 {
            buf.push(0);
        }

        buf
    }

    /// Parses One-Byte Header Extensions from raw extension data bytes.
    pub fn parse_list(data: &[u8]) -> Vec<OneByteHeaderExtension> {
        let mut extensions = Vec::new();
        let mut offset = 0;

        while offset < data.len() {
            let byte = data[offset];
            if byte == 0 {
                // 0 is padding byte
                offset += 1;
                continue;
            }

            let id = (byte >> 4) & 0x0F;
            if id == 15 {
                // Stop code
                break;
            }

            let len = ((byte & 0x0F) + 1) as usize;
            offset += 1;

            if offset + len <= data.len() {
                extensions.push(OneByteHeaderExtension {
                    id,
                    data: data[offset..offset + len].to_vec(),
                });
                offset += len;
            } else {
                break;
            }
        }

        extensions
    }
}

/// Client-to-Server Audio Level Header Extension (RFC 6464).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioLevelExtension {
    /// Voice activity detection flag (true = active voice speech detected).
    pub voice_activity: bool,
    /// Audio level in -dBov (0 = loudest / 0 dBov, 127 = silence / -127 dBov).
    pub level_dbov: u8,
}

impl AudioLevelExtension {
    pub const EXTENSION_ID: u8 = 1;

    pub fn new(voice_activity: bool, level_dbov: u8) -> Self {
        Self {
            voice_activity,
            level_dbov: level_dbov.min(127),
        }
    }

    /// Converts this audio level into a One-Byte Header Extension.
    pub fn to_one_byte_extension(&self, id: u8) -> OneByteHeaderExtension {
        let byte = if self.voice_activity {
            0x80 | (self.level_dbov & 0x7F)
        } else {
            self.level_dbov & 0x7F
        };
        OneByteHeaderExtension::new(id, vec![byte])
    }

    /// Extracts an AudioLevelExtension from a One-Byte Header Extension if matching.
    pub fn from_one_byte_extension(ext: &OneByteHeaderExtension) -> Option<Self> {
        if ext.data.is_empty() {
            return None;
        }
        let byte = ext.data[0];
        let voice_activity = (byte & 0x80) != 0;
        let level_dbov = byte & 0x7F;
        Some(Self {
            voice_activity,
            level_dbov,
        })
    }
}

/// VP8 Payload Descriptor as defined in RFC 7741 §4.2.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Vp8PayloadDescriptor {
    /// Extended control bits present (X bit).
    pub extended_control: bool,
    /// Non-reference frame (N bit - 1 if frame can be discarded without affecting other frames).
    pub non_reference: bool,
    /// Start of VP8 partition (S bit - 1 if first packet of a VP8 partition / frame).
    pub start_of_partition: bool,
    /// Partition index (PID - 4 bits, 0 to 15).
    pub partition_index: u8,
    /// Optional PictureID (I bit, 7-bit or 15-bit).
    pub picture_id: Option<u16>,
    /// Optional TL0PICIDX (L bit, Layer 0 Picture Index).
    pub tl0_pic_idx: Option<u8>,
    /// Optional Temporal Layer ID (T bit, 2 bits: 0, 1, 2).
    pub temporal_layer_id: Option<u8>,
    /// Optional Layer Sync (Y bit).
    pub layer_sync: Option<bool>,
    /// Optional Key Index (K bit, 5 bits).
    pub key_index: Option<u8>,
}

impl Vp8PayloadDescriptor {
    /// Serializes the VP8 payload descriptor into a byte vector.
    pub fn write_to(&self, buf: &mut Vec<u8>) {
        let has_picture_id = self.picture_id.is_some();
        let has_tl0 = self.tl0_pic_idx.is_some();
        let has_tid = self.temporal_layer_id.is_some();
        let has_key_idx = self.key_index.is_some();

        let has_extension = self.extended_control || has_picture_id || has_tl0 || has_tid || has_key_idx;

        // Byte 0: X | R | N | S | PartID
        let mut byte0 = self.partition_index & 0x0F;
        if has_extension {
            byte0 |= 0x80;
        }
        if self.non_reference {
            byte0 |= 0x20;
        }
        if self.start_of_partition {
            byte0 |= 0x10;
        }
        buf.push(byte0);

        if has_extension {
            // Extension Byte: I | L | T | K | RSV
            let mut ext_byte = 0u8;
            if has_picture_id {
                ext_byte |= 0x80;
            }
            if has_tl0 {
                ext_byte |= 0x40;
            }
            if has_tid {
                ext_byte |= 0x20;
            }
            if has_key_idx {
                ext_byte |= 0x10;
            }
            buf.push(ext_byte);

            // Optional PictureID
            if let Some(pic_id) = self.picture_id {
                if pic_id > 127 {
                    // 15-bit Picture ID: M bit set to 1 on first octet
                    let high = 0x80 | ((pic_id >> 8) as u8 & 0x7F);
                    let low = (pic_id & 0xFF) as u8;
                    buf.push(high);
                    buf.push(low);
                } else {
                    // 7-bit Picture ID: M bit set to 0
                    buf.push(pic_id as u8 & 0x7F);
                }
            }

            // Optional TL0PICIDX
            if let Some(tl0) = self.tl0_pic_idx {
                buf.push(tl0);
            }

            // Optional TID / KEYIDX
            if has_tid || has_key_idx {
                let tid = self.temporal_layer_id.unwrap_or(0) & 0x03;
                let y = if self.layer_sync.unwrap_or(false) { 0x20 } else { 0x00 };
                let key_idx = self.key_index.unwrap_or(0) & 0x1F;
                let t_k_byte = (tid << 6) | y | key_idx;
                buf.push(t_k_byte);
            }
        }
    }

    /// Parses a VP8 payload descriptor from raw payload bytes. Returns the descriptor and consumed bytes count.
    pub fn parse(data: &[u8]) -> Result<(Self, usize), RtpError> {
        if data.is_empty() {
            return Err(RtpError::EmptyPayload);
        }

        let byte0 = data[0];
        let extended_control = (byte0 & 0x80) != 0;
        let non_reference = (byte0 & 0x20) != 0;
        let start_of_partition = (byte0 & 0x10) != 0;
        let partition_index = byte0 & 0x0F;

        let mut offset = 1;
        let mut picture_id = None;
        let mut tl0_pic_idx = None;
        let mut temporal_layer_id = None;
        let mut layer_sync = None;
        let mut key_index = None;

        if extended_control {
            if data.len() <= offset {
                return Err(RtpError::InvalidVp8Descriptor("Truncated extension byte".to_string()));
            }

            let ext_byte = data[offset];
            offset += 1;

            let has_picture_id = (ext_byte & 0x80) != 0;
            let has_tl0 = (ext_byte & 0x40) != 0;
            let has_tid = (ext_byte & 0x20) != 0;
            let has_key_idx = (ext_byte & 0x10) != 0;

            if has_picture_id {
                if data.len() <= offset {
                    return Err(RtpError::InvalidVp8Descriptor("Truncated PictureID".to_string()));
                }
                let pic_byte0 = data[offset];
                offset += 1;

                if (pic_byte0 & 0x80) != 0 {
                    // 15-bit Picture ID
                    if data.len() <= offset {
                        return Err(RtpError::InvalidVp8Descriptor("Truncated 15-bit PictureID".to_string()));
                    }
                    let pic_byte1 = data[offset];
                    offset += 1;
                    let pic_val = (((pic_byte0 & 0x7F) as u16) << 8) | (pic_byte1 as u16);
                    picture_id = Some(pic_val);
                } else {
                    // 7-bit Picture ID
                    picture_id = Some((pic_byte0 & 0x7F) as u16);
                }
            }

            if has_tl0 {
                if data.len() <= offset {
                    return Err(RtpError::InvalidVp8Descriptor("Truncated TL0PICIDX".to_string()));
                }
                tl0_pic_idx = Some(data[offset]);
                offset += 1;
            }

            if has_tid || has_key_idx {
                if data.len() <= offset {
                    return Err(RtpError::InvalidVp8Descriptor("Truncated TID/KEYIDX".to_string()));
                }
                let tk_byte = data[offset];
                offset += 1;
                if has_tid {
                    temporal_layer_id = Some((tk_byte >> 6) & 0x03);
                    layer_sync = Some((tk_byte & 0x20) != 0);
                }
                if has_key_idx {
                    key_index = Some(tk_byte & 0x1F);
                }
            }
        }

        Ok((
            Self {
                extended_control,
                non_reference,
                start_of_partition,
                partition_index,
                picture_id,
                tl0_pic_idx,
                temporal_layer_id,
                layer_sync,
                key_index,
            },
            offset,
        ))
    }
}

/// VP8 Frame Header parser & builder (RFC 6386 §9.1 / RFC 7741 §4.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vp8FrameHeader {
    pub is_keyframe: bool,
    pub version: u8,
    pub show_frame: bool,
    pub first_partition_size: u32,
    pub width: Option<u16>,
    pub height: Option<u16>,
}

impl Vp8FrameHeader {
    /// Parses the uncompressed VP8 frame header when S=1 and PID=0.
    pub fn parse(payload: &[u8]) -> Option<Self> {
        if payload.len() < 3 {
            return None;
        }

        let b0 = payload[0];
        let is_keyframe = (b0 & 0x01) == 0;
        let version = (b0 >> 1) & 0x07;
        let show_frame = (b0 & 0x10) != 0;
        let first_partition_size = (b0 as u32 >> 5) | ((payload[1] as u32) << 3) | ((payload[2] as u32) << 11);

        let mut width = None;
        let mut height = None;

        if is_keyframe && payload.len() >= 10 {
            // Start code: 0x9D, 0x01, 0x2A
            if payload[3] == 0x9D && payload[4] == 0x01 && payload[5] == 0x2A {
                let w = (payload[6] as u16 | ((payload[7] as u16) << 8)) & 0x3FFF;
                let h = (payload[8] as u16 | ((payload[9] as u16) << 8)) & 0x3FFF;
                width = Some(w);
                height = Some(h);
            }
        }

        Some(Self {
            is_keyframe,
            version,
            show_frame,
            first_partition_size,
            width,
            height,
        })
    }

    /// Constructs a standard 10-byte VP8 keyframe header with start code and dimensions.
    pub fn build_keyframe_header(width: u16, height: u16, partition_size: usize) -> [u8; 10] {
        let part_size = (partition_size as u32) & 0x7FFFF; // 19 bits
        let b0 = ((part_size & 0x07) << 5) as u8 | 0x10; // show_frame = 1, keyframe = 0, ver = 0
        let b1 = ((part_size >> 3) & 0xFF) as u8;
        let b2 = ((part_size >> 11) & 0xFF) as u8;

        let w_bytes = (width & 0x3FFF).to_le_bytes();
        let h_bytes = (height & 0x3FFF).to_le_bytes();

        [
            b0, b1, b2,
            0x9D, 0x01, 0x2A, // VP8 Start Code
            w_bytes[0], w_bytes[1],
            h_bytes[0], h_bytes[1],
        ]
    }
}

/// VP8 Video Packetizer (RFC 7741).
///
/// Handles fragmenting VP8 frames across RTP packets with MTU constraints,
/// maintaining sequence numbers, 90 kHz clock timestamps, and 15-bit Picture IDs.
pub struct Vp8Packetizer {
    pub payload_type: u8,
    pub ssrc: u32,
    pub sequence_number: u16,
    pub picture_id: u16,
    pub max_payload_size: usize,
}

impl Vp8Packetizer {
    pub const DEFAULT_PAYLOAD_TYPE: u8 = 96;
    pub const VIDEO_CLOCK_RATE: u32 = 90_000;
    pub const DEFAULT_MAX_PAYLOAD_SIZE: usize = 1200;

    pub fn new(ssrc: u32) -> Self {
        Self {
            payload_type: Self::DEFAULT_PAYLOAD_TYPE,
            ssrc,
            sequence_number: 1,
            picture_id: 1,
            max_payload_size: Self::DEFAULT_MAX_PAYLOAD_SIZE,
        }
    }

    /// Sets the maximum payload size (MTU) per packet.
    pub fn with_max_payload_size(mut self, max_size: usize) -> Self {
        self.max_payload_size = max_size.max(256);
        self
    }

    /// Packetizes a raw VP8 frame into a sequence of RTP packets.
    pub fn packetize(&mut self, frame: &[u8], timestamp: u32, is_keyframe: bool) -> Vec<RtpPacket> {
        let pic_id = self.picture_id;
        self.picture_id = (self.picture_id + 1) & 0x7FFF; // 15-bit rollover
        self.packetize_with_picture_id(frame, timestamp, is_keyframe, pic_id)
    }

    /// Packetizes a VP8 frame using a specific PictureID.
    pub fn packetize_with_picture_id(
        &mut self,
        frame: &[u8],
        timestamp: u32,
        is_keyframe: bool,
        picture_id: u16,
    ) -> Vec<RtpPacket> {
        if frame.is_empty() {
            return Vec::new();
        }

        let mut packets = Vec::new();
        let total_len = frame.len();
        let mut offset = 0;
        let mut is_first = true;

        while offset < total_len {
            let descriptor = Vp8PayloadDescriptor {
                extended_control: true,
                non_reference: !is_keyframe,
                start_of_partition: is_first,
                partition_index: 0,
                picture_id: Some(picture_id),
                tl0_pic_idx: None,
                temporal_layer_id: None,
                layer_sync: None,
                key_index: None,
            };

            let mut desc_buf = Vec::new();
            descriptor.write_to(&mut desc_buf);

            let available_payload_space = self.max_payload_size.saturating_sub(desc_buf.len());
            let chunk_len = (total_len - offset).min(available_payload_space.max(64));
            let chunk = &frame[offset..offset + chunk_len];
            offset += chunk_len;

            let is_last = offset >= total_len;

            let mut rtp_payload = Vec::with_capacity(desc_buf.len() + chunk.len());
            rtp_payload.extend_from_slice(&desc_buf);
            rtp_payload.extend_from_slice(chunk);

            let mut header = RtpHeader::new(self.payload_type, self.sequence_number, timestamp, self.ssrc);
            header.marker = is_last; // Marker bit set on final packet of video frame (RFC 7741 §4.1)

            self.sequence_number = self.sequence_number.wrapping_add(1);
            packets.push(RtpPacket::new(header, rtp_payload));

            is_first = false;
        }

        packets
    }
}

/// Assembled VP8 video frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vp8Frame {
    pub timestamp: u32,
    pub picture_id: Option<u16>,
    pub is_keyframe: bool,
    pub payload: Vec<u8>,
}

/// VP8 Frame Assembler / Depacketizer.
///
/// Reassembles incoming fragmented RTP packets into complete VP8 video frames.
#[derive(Default)]
pub struct Vp8FrameAssembler {
    pending_packets: Vec<RtpPacket>,
    expected_seq: Option<u16>,
}

impl Vp8FrameAssembler {
    pub fn new() -> Self {
        Self {
            pending_packets: Vec::new(),
            expected_seq: None,
        }
    }

    /// Resets the current assembler state on packet loss or stream disruption.
    pub fn reset(&mut self) {
        self.pending_packets.clear();
        self.expected_seq = None;
    }

    /// Pushes an incoming RTP packet. Returns `Some(Vp8Frame)` when a full frame is assembled.
    pub fn push_packet(&mut self, packet: RtpPacket) -> Result<Option<Vp8Frame>, RtpError> {
        let seq = packet.header.sequence_number;

        // Sequence number continuity check
        if let Some(expected) = self.expected_seq {
            if seq != expected {
                let actual = seq;
                self.reset();
                return Err(RtpError::SequenceGap { expected, actual });
            }
        }
        self.expected_seq = Some(seq.wrapping_add(1));

        let is_marker = packet.header.marker;
        self.pending_packets.push(packet);

        if is_marker {
            let packets = std::mem::take(&mut self.pending_packets);
            let frame = self.assemble_frame(&packets)?;
            return Ok(Some(frame));
        }

        Ok(None)
    }

    fn assemble_frame(&self, packets: &[RtpPacket]) -> Result<Vp8Frame, RtpError> {
        if packets.is_empty() {
            return Err(RtpError::IncompleteFrame);
        }

        let first_packet = &packets[0];
        let timestamp = first_packet.header.timestamp;
        let mut total_payload = Vec::new();
        let mut first_descriptor: Option<Vp8PayloadDescriptor> = None;

        for (idx, pkt) in packets.iter().enumerate() {
            let (descriptor, desc_len) = Vp8PayloadDescriptor::parse(&pkt.payload)?;
            if idx == 0 {
                if !descriptor.start_of_partition {
                    return Err(RtpError::InvalidVp8Descriptor("First packet missing S bit".to_string()));
                }
                first_descriptor = Some(descriptor);
            }
            total_payload.extend_from_slice(&pkt.payload[desc_len..]);
        }

        let descriptor = first_descriptor.unwrap();
        let is_keyframe = if let Some(hdr) = Vp8FrameHeader::parse(&total_payload) {
            hdr.is_keyframe
        } else {
            !descriptor.non_reference
        };

        Ok(Vp8Frame {
            timestamp,
            picture_id: descriptor.picture_id,
            is_keyframe,
            payload: total_payload,
        })
    }
}

/// Opus Audio Packetizer (RFC 7587).
///
/// Encapsulates Opus audio frames with 48 kHz timestamps and RFC 6464 Audio Level header extensions.
pub struct OpusPacketizer {
    pub payload_type: u8,
    pub ssrc: u32,
    pub sequence_number: u16,
    pub timestamp: u32,
    pub frame_duration_ms: u32,
    pub audio_level_ext_id: Option<u8>,
}

impl OpusPacketizer {
    pub const DEFAULT_PAYLOAD_TYPE: u8 = 111;
    pub const AUDIO_CLOCK_RATE: u32 = 48_000;
    pub const DEFAULT_FRAME_DURATION_MS: u32 = 20; // 960 samples @ 48kHz

    pub fn new(ssrc: u32) -> Self {
        Self {
            payload_type: Self::DEFAULT_PAYLOAD_TYPE,
            ssrc,
            sequence_number: 1,
            timestamp: 0,
            frame_duration_ms: Self::DEFAULT_FRAME_DURATION_MS,
            audio_level_ext_id: Some(AudioLevelExtension::EXTENSION_ID),
        }
    }

    /// Computes timestamp ticks increment per audio frame.
    pub fn samples_per_frame(&self) -> u32 {
        (Self::AUDIO_CLOCK_RATE * self.frame_duration_ms) / 1000
    }

    /// Packetizes an Opus audio frame with optional audio level energy.
    pub fn packetize(
        &mut self,
        opus_payload: &[u8],
        audio_level: Option<AudioLevelExtension>,
    ) -> RtpPacket {
        let mut header = RtpHeader::new(self.payload_type, self.sequence_number, self.timestamp, self.ssrc);

        // Include RFC 6464 audio level header extension if configured
        if let (Some(ext_id), Some(level)) = (self.audio_level_ext_id, audio_level) {
            let one_byte_ext = level.to_one_byte_extension(ext_id);
            let ext_bytes = OneByteHeaderExtension::serialize_list(&[one_byte_ext]);
            header.extension = true;
            header.extension_profile = Some(RtpHeader::ONE_BYTE_EXT_PROFILE);
            header.extension_data = Some(ext_bytes);
        }

        self.sequence_number = self.sequence_number.wrapping_add(1);
        self.timestamp = self.timestamp.wrapping_add(self.samples_per_frame());

        RtpPacket::new(header, opus_payload.to_vec())
    }
}

/// Decoded Opus Audio Frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpusAudioFrame {
    pub payload: Vec<u8>,
    pub sequence_number: u16,
    pub timestamp: u32,
    pub audio_level: Option<AudioLevelExtension>,
}

/// Opus Audio Depacketizer.
pub struct OpusDepacketizer;

impl OpusDepacketizer {
    /// Extracts Opus audio payload and optional RFC 6464 audio level from an RTP packet.
    pub fn depacketize(packet: &RtpPacket, audio_level_ext_id: Option<u8>) -> Result<OpusAudioFrame, RtpError> {
        let mut audio_level = None;

        if packet.header.extension {
            if let Some(ext_data) = &packet.header.extension_data {
                let target_id = audio_level_ext_id.unwrap_or(AudioLevelExtension::EXTENSION_ID);
                let extensions = OneByteHeaderExtension::parse_list(ext_data);
                for ext in extensions {
                    if ext.id == target_id {
                        audio_level = AudioLevelExtension::from_one_byte_extension(&ext);
                        break;
                    }
                }
            }
        }

        Ok(OpusAudioFrame {
            payload: packet.payload.clone(),
            sequence_number: packet.header.sequence_number,
            timestamp: packet.header.timestamp,
            audio_level,
        })
    }
}

/// Helper function to compute RMS energy and RFC 6464 -dBov level from 16-bit PCM audio samples.
pub fn compute_pcm_audio_level_dbov(samples: &[i16]) -> (bool, u8) {
    if samples.is_empty() {
        return (false, 127);
    }

    let mut sum_sq = 0u64;
    for &s in samples {
        let val = s as i64;
        sum_sq += (val * val) as u64;
    }

    let mean_sq = sum_sq as f64 / samples.len() as f64;
    let rms = mean_sq.sqrt();

    if rms < 1.0 {
        return (false, 127);
    }

    // 0 dBov is full scale 32767.0
    let dbov = 20.0 * (rms / 32767.0).log10();
    let abs_dbov = (-dbov).clamp(0.0, 127.0).round() as u8;

    // Voice activity threshold (e.g. > -50 dBov => abs_dbov < 50)
    let is_voice = abs_dbov < 50;

    (is_voice, abs_dbov)
}

/// Statistics tracker for an active RTP Media Transport stream.
#[derive(Debug, Clone, Default)]
pub struct RtpStreamStats {
    pub packets_sent: u64,
    pub bytes_sent: u64,
    pub packets_received: u64,
    pub bytes_received: u64,
    pub packets_lost: u32,
    pub fraction_lost: f32,
    pub jitter_samples: f64,
    pub rtt_ms: u32,
}

impl RtpStreamStats {
    pub fn record_sent(&mut self, bytes: usize) {
        self.packets_sent += 1;
        self.bytes_sent += bytes as u64;
    }

    pub fn record_received(&mut self, bytes: usize) {
        self.packets_received += 1;
        self.bytes_received += bytes as u64;
    }
}

// ---------------------------------------------------------------------------
// Hardware & UI Capture Frame Pipeline Integration Helpers
// ---------------------------------------------------------------------------

/// Encapsulates captured video frames (Camera or Screen Share) into RTP packets.
///
/// Builds a VP8 bitstream with keyframe/interframe headers and chunks into MTU-compliant RTP packets.
pub fn capture_frame_to_vp8_rtp_packets(
    frame: &ColorImage,
    packetizer: &mut Vp8Packetizer,
    timestamp: u32,
    is_keyframe: bool,
) -> Vec<RtpPacket> {
    let (width, height) = (frame.size[0] as u16, frame.size[1] as u16);
    let raw_pixels = &frame.pixels;

    // Simulate compressed VP8 partition payload from raw pixel buffer
    let mut payload = Vec::new();
    if is_keyframe {
        let key_hdr = Vp8FrameHeader::build_keyframe_header(width, height, raw_pixels.len().min(4096));
        payload.extend_from_slice(&key_hdr);
    } else {
        // Interframe 3-byte header
        let part_size = (raw_pixels.len().min(4096) as u32) & 0x7FFFF;
        let b0 = ((part_size & 0x07) << 5) as u8 | 0x11; // show_frame = 1, interframe = 1
        let b1 = ((part_size >> 3) & 0xFF) as u8;
        let b2 = ((part_size >> 11) & 0xFF) as u8;
        payload.extend_from_slice(&[b0, b1, b2]);
    }

    // Append downsampled/compressed pixel summary payload
    for chunk in raw_pixels.chunks(16) {
        if let Some(first) = chunk.first() {
            payload.push(first.r());
            payload.push(first.g());
            payload.push(first.b());
        }
    }

    packetizer.packetize(&payload, timestamp, is_keyframe)
}

/// Encapsulates raw PCM microphone audio samples into an Opus RTP packet with RFC 6464 audio level.
pub fn audio_samples_to_opus_rtp_packet(
    samples: &[i16],
    packetizer: &mut OpusPacketizer,
) -> RtpPacket {
    let (is_voice, dbov) = compute_pcm_audio_level_dbov(samples);
    let audio_level = AudioLevelExtension::new(is_voice, dbov);

    // Mock Opus TOC byte + compressed payload
    let mut opus_payload = vec![0xF8]; // Opus Config TOC (CELT/SILK 20ms mono)
    for chunk in samples.chunks(32) {
        if let Some(s) = chunk.first() {
            opus_payload.extend_from_slice(&s.to_be_bytes());
        }
    }

    packetizer.packetize(&opus_payload, Some(audio_level))
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Color32;

    #[test]
    fn test_rtp_fixed_header_serialization_and_deserialization() {
        let mut header = RtpHeader::new(96, 1024, 90000, 0x12345678);
        header.marker = true;

        let mut buf = Vec::new();
        header.serialize_into(&mut buf);

        assert_eq!(buf.len(), 12);
        // Byte 0: V=2 (0x80)
        assert_eq!(buf[0], 0x80);
        // Byte 1: M=1 | PT=96 (0x80 | 96 = 0xE0)
        assert_eq!(buf[1], 0xE0);
        // Seq: 1024 (0x0400)
        assert_eq!(&buf[2..4], &[0x04, 0x00]);
        // SSRC: 0x12345678
        assert_eq!(&buf[8..12], &[0x12, 0x34, 0x56, 0x78]);

        let (parsed, size) = RtpHeader::parse(&buf).expect("Failed to parse RTP header");
        assert_eq!(size, 12);
        assert_eq!(parsed.version, 2);
        assert_eq!(parsed.marker, true);
        assert_eq!(parsed.payload_type, 96);
        assert_eq!(parsed.sequence_number, 1024);
        assert_eq!(parsed.timestamp, 90000);
        assert_eq!(parsed.ssrc, 0x12345678);
    }

    #[test]
    fn test_rtp_header_with_csrc_and_extension() {
        let mut header = RtpHeader::new(111, 42, 48000, 0xDEADBEEF);
        header.csrc_list = vec![0x11111111, 0x22222222];
        header.extension = true;
        header.extension_profile = Some(RtpHeader::ONE_BYTE_EXT_PROFILE);

        let audio_level = AudioLevelExtension::new(true, 30);
        let ext_element = audio_level.to_one_byte_extension(1);
        let ext_bytes = OneByteHeaderExtension::serialize_list(&[ext_element]);
        header.extension_data = Some(ext_bytes);

        let mut buf = Vec::new();
        header.serialize_into(&mut buf);

        let (parsed, size) = RtpHeader::parse(&buf).expect("Failed to parse extended header");
        assert_eq!(size, buf.len());
        assert_eq!(parsed.csrc_list.len(), 2);
        assert_eq!(parsed.csrc_list[0], 0x11111111);
        assert_eq!(parsed.csrc_list[1], 0x22222222);
        assert_eq!(parsed.extension, true);
        assert_eq!(parsed.extension_profile, Some(RtpHeader::ONE_BYTE_EXT_PROFILE));

        let ext_parsed = OneByteHeaderExtension::parse_list(parsed.extension_data.as_ref().unwrap());
        assert_eq!(ext_parsed.len(), 1);
        assert_eq!(ext_parsed[0].id, 1);

        let audio_level_parsed = AudioLevelExtension::from_one_byte_extension(&ext_parsed[0]).unwrap();
        assert_eq!(audio_level_parsed.voice_activity, true);
        assert_eq!(audio_level_parsed.level_dbov, 30);
    }

    #[test]
    fn test_rtp_packet_with_padding_roundtrip() {
        let header = RtpHeader::new(96, 500, 180000, 0xAABBCCDD);
        let payload = vec![1, 2, 3, 4, 5];
        let mut packet = RtpPacket::new(header, payload.clone());
        packet.padding_len = 4;

        let raw = packet.serialize();
        assert_eq!(*raw.last().unwrap(), 4); // RFC 3550 §5.1 padding count at end

        let parsed = RtpPacket::parse(&raw).expect("Failed to parse padded packet");
        assert_eq!(parsed.header.padding, true);
        assert_eq!(parsed.payload, payload);
        assert_eq!(parsed.padding_len, 4);
    }

    #[test]
    fn test_vp8_payload_descriptor_roundtrip() {
        let desc = Vp8PayloadDescriptor {
            extended_control: true,
            non_reference: false,
            start_of_partition: true,
            partition_index: 0,
            picture_id: Some(1500), // 15-bit
            tl0_pic_idx: Some(25),
            temporal_layer_id: Some(2),
            layer_sync: Some(true),
            key_index: Some(7),
        };

        let mut buf = Vec::new();
        desc.write_to(&mut buf);

        let (parsed, consumed) = Vp8PayloadDescriptor::parse(&buf).expect("Failed to parse VP8 descriptor");
        assert_eq!(consumed, buf.len());
        assert_eq!(parsed.start_of_partition, true);
        assert_eq!(parsed.non_reference, false);
        assert_eq!(parsed.picture_id, Some(1500));
        assert_eq!(parsed.tl0_pic_idx, Some(25));
        assert_eq!(parsed.temporal_layer_id, Some(2));
        assert_eq!(parsed.layer_sync, Some(true));
        assert_eq!(parsed.key_index, Some(7));
    }

    #[test]
    fn test_vp8_7bit_picture_id_descriptor() {
        let desc = Vp8PayloadDescriptor {
            extended_control: true,
            non_reference: true,
            start_of_partition: false,
            partition_index: 0,
            picture_id: Some(63), // 7-bit Picture ID (< 128)
            tl0_pic_idx: None,
            temporal_layer_id: None,
            layer_sync: None,
            key_index: None,
        };

        let mut buf = Vec::new();
        desc.write_to(&mut buf);
        assert_eq!(buf.len(), 3); // Byte0 (0x80 | 0x20) + ExtByte (0x80) + PicID (63 with MSB 0)
        assert_eq!(buf[2] & 0x80, 0); // 7-bit has MSB=0

        let (parsed, consumed) = Vp8PayloadDescriptor::parse(&buf).expect("Failed to parse 7-bit PicID");
        assert_eq!(consumed, 3);
        assert_eq!(parsed.picture_id, Some(63));
        assert_eq!(parsed.non_reference, true);
        assert_eq!(parsed.start_of_partition, false);
    }

    #[test]
    fn test_vp8_keyframe_header_builder_and_parser() {
        let width = 1280u16;
        let height = 720u16;
        let key_hdr = Vp8FrameHeader::build_keyframe_header(width, height, 4096);

        let parsed = Vp8FrameHeader::parse(&key_hdr).expect("Failed to parse keyframe header");
        assert_eq!(parsed.is_keyframe, true);
        assert_eq!(parsed.show_frame, true);
        assert_eq!(parsed.width, Some(1280));
        assert_eq!(parsed.height, Some(720));
    }

    #[test]
    fn test_vp8_packetizer_fragmentation_and_assembler() {
        let mut packetizer = Vp8Packetizer::new(0xCAFEBABE).with_max_payload_size(500);

        // Generate synthetic VP8 keyframe (1500 bytes)
        let key_hdr = Vp8FrameHeader::build_keyframe_header(640, 480, 1500);
        let mut frame_data = key_hdr.to_vec();
        frame_data.resize(1500, 0xAB);

        let timestamp = 90000;
        let packets = packetizer.packetize(&frame_data, timestamp, true);

        // Expect frame to be fragmented across 4 packets (500-byte chunks)
        assert!(packets.len() >= 3);

        // Validate RFC 3550 & RFC 7741 headers on all packets
        for (i, pkt) in packets.iter().enumerate() {
            assert_eq!(pkt.header.version, 2);
            assert_eq!(pkt.header.payload_type, 96);
            assert_eq!(pkt.header.timestamp, timestamp);
            assert_eq!(pkt.header.ssrc, 0xCAFEBABE);

            let (desc, _) = Vp8PayloadDescriptor::parse(&pkt.payload).expect("VP8 payload parse failed");
            assert_eq!(desc.picture_id, Some(1));

            if i == 0 {
                assert_eq!(desc.start_of_partition, true);
                assert_eq!(pkt.header.marker, false);
            } else if i == packets.len() - 1 {
                assert_eq!(desc.start_of_partition, false);
                assert_eq!(pkt.header.marker, true); // End of frame marker
            } else {
                assert_eq!(desc.start_of_partition, false);
                assert_eq!(pkt.header.marker, false);
            }
        }

        // Test reassembly with Vp8FrameAssembler
        let mut assembler = Vp8FrameAssembler::new();
        let mut completed_frame = None;

        for pkt in packets {
            if let Some(frame) = assembler.push_packet(pkt).expect("Frame assembly error") {
                completed_frame = Some(frame);
            }
        }

        let frame = completed_frame.expect("Frame failed to complete assembly");
        assert_eq!(frame.timestamp, timestamp);
        assert_eq!(frame.picture_id, Some(1));
        assert_eq!(frame.is_keyframe, true);
        assert_eq!(frame.payload, frame_data);
    }

    #[test]
    fn test_vp8_assembler_detects_sequence_gap() {
        let mut packetizer = Vp8Packetizer::new(0x11223344).with_max_payload_size(256);
        let key_hdr = Vp8FrameHeader::build_keyframe_header(320, 240, 1000);
        let mut frame_data = key_hdr.to_vec();
        frame_data.resize(1000, 0xCD);

        let mut packets = packetizer.packetize(&frame_data, 1000, true);
        assert!(packets.len() >= 3);

        // Drop the second packet to simulate network packet loss
        packets.remove(1);

        let mut assembler = Vp8FrameAssembler::new();
        let res1 = assembler.push_packet(packets.remove(0));
        assert!(res1.is_ok());

        // Pushing the next packet should return SequenceGap error
        let res2 = assembler.push_packet(packets.remove(0));
        assert!(matches!(res2, Err(RtpError::SequenceGap { .. })));
    }

    #[test]
    fn test_multi_frame_progression_sequence_and_timestamps() {
        let mut packetizer = Vp8Packetizer::new(0x99887766).with_max_payload_size(500);

        let mut current_ts = 90000u32;
        let ts_step = 3000u32; // 30 fps (90000 / 30 = 3000)

        for frame_idx in 0..5 {
            let is_key = frame_idx == 0;
            let mut payload = vec![0x10; 800];
            if is_key {
                let hdr = Vp8FrameHeader::build_keyframe_header(640, 360, 800);
                payload[..10].copy_from_slice(&hdr);
            }

            let pkts = packetizer.packetize(&payload, current_ts, is_key);
            assert_eq!(pkts.first().unwrap().header.timestamp, current_ts);
            assert_eq!(pkts.last().unwrap().header.marker, true);

            current_ts += ts_step;
        }

        // Sequence numbers must be strictly increasing
        assert_eq!(packetizer.sequence_number, 11); // 5 frames * 2 packets/frame = 10 packets (seq 1 to 10)
        assert_eq!(packetizer.picture_id, 6);
    }

    #[test]
    fn test_opus_packetizer_and_depacketizer() {
        let mut packetizer = OpusPacketizer::new(0x55667788);
        let synthetic_opus_bytes = vec![0xFC, 0x01, 0x02, 0x03, 0x04];
        let audio_level = AudioLevelExtension::new(true, 18);

        let packet = packetizer.packetize(&synthetic_opus_bytes, Some(audio_level));

        assert_eq!(packet.header.version, 2);
        assert_eq!(packet.header.payload_type, 111);
        assert_eq!(packet.header.sequence_number, 1);
        assert_eq!(packet.header.timestamp, 0);
        assert_eq!(packet.header.extension, true);

        let depacketized = OpusDepacketizer::depacketize(&packet, Some(1)).expect("Depacketization failed");
        assert_eq!(depacketized.payload, synthetic_opus_bytes);
        assert_eq!(depacketized.sequence_number, 1);
        assert_eq!(depacketized.timestamp, 0);

        let parsed_level = depacketized.audio_level.expect("Missing audio level extension");
        assert_eq!(parsed_level.voice_activity, true);
        assert_eq!(parsed_level.level_dbov, 18);
    }

    #[test]
    fn test_pcm_audio_energy_dbov_calculation() {
        // 1. Full scale sine-like samples -> high energy (low -dBov value)
        let loud_samples = vec![25000i16; 480];
        let (is_voice, dbov) = compute_pcm_audio_level_dbov(&loud_samples);
        assert!(dbov < 10);
        assert_eq!(is_voice, true);

        // 2. Silence samples -> maximum -dBov value (127)
        let silence_samples = vec![0i16; 480];
        let (is_voice_silence, dbov_silence) = compute_pcm_audio_level_dbov(&silence_samples);
        assert_eq!(dbov_silence, 127);
        assert_eq!(is_voice_silence, false);
    }

    #[test]
    fn test_capture_frame_pipeline_integration() {
        let mut packetizer = Vp8Packetizer::new(0x77889900).with_max_payload_size(600);

        // Create a synthetic captured UI camera frame
        let width = 64;
        let height = 48;
        let pixels = vec![Color32::from_rgb(180, 200, 220); width * height];
        let captured_frame = ColorImage {
            size: [width, height],
            pixels,
        };

        // 1. Packetize Keyframe
        let key_packets = capture_frame_to_vp8_rtp_packets(&captured_frame, &mut packetizer, 90000, true);
        assert!(!key_packets.is_empty());
        assert_eq!(key_packets.first().unwrap().header.payload_type, 96);
        assert_eq!(key_packets.last().unwrap().header.marker, true);

        // Validate reassembly
        let mut assembler = Vp8FrameAssembler::new();
        let mut assembled_key = None;
        for pkt in key_packets {
            if let Some(f) = assembler.push_packet(pkt).unwrap() {
                assembled_key = Some(f);
            }
        }
        let key_frame = assembled_key.expect("Keyframe should assemble");
        assert_eq!(key_frame.is_keyframe, true);
        assert_eq!(key_frame.timestamp, 90000);

        // 2. Packetize Interframe (delta frame)
        let delta_packets = capture_frame_to_vp8_rtp_packets(&captured_frame, &mut packetizer, 93000, false);
        assert!(!delta_packets.is_empty());

        let mut assembled_delta = None;
        for pkt in delta_packets {
            if let Some(f) = assembler.push_packet(pkt).unwrap() {
                assembled_delta = Some(f);
            }
        }
        let delta_frame = assembled_delta.expect("Delta frame should assemble");
        assert_eq!(delta_frame.is_keyframe, false);
        assert_eq!(delta_frame.timestamp, 93000);
    }

    #[test]
    fn test_audio_capture_pipeline_integration() {
        let mut packetizer = OpusPacketizer::new(0x33445566);
        let pcm_mic_samples = vec![12000i16; 960]; // 20ms speech audio @ 48kHz

        let rtp_packet = audio_samples_to_opus_rtp_packet(&pcm_mic_samples, &mut packetizer);

        assert_eq!(rtp_packet.header.version, 2);
        assert_eq!(rtp_packet.header.payload_type, 111);
        assert_eq!(rtp_packet.header.extension, true);

        let decoded = OpusDepacketizer::depacketize(&rtp_packet, Some(1)).unwrap();
        assert!(!decoded.payload.is_empty());
        let audio_level = decoded.audio_level.unwrap();
        assert_eq!(audio_level.voice_activity, true);
        assert!(audio_level.level_dbov < 20);
    }
}
