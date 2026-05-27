//! Minimal libpng validation port for PNG signature and chunk-header parsing.
//!
//! This crate intentionally covers only the first parsing boundary: comparing
//! the 8-byte PNG signature, validating 4-byte chunk type names, and extracting
//! owned chunk-header metadata from progressive input.

pub const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

const PNG_SIGNATURE_LEN: usize = PNG_SIGNATURE.len();
const CHUNK_HEADER_LEN: usize = 8;
const CHUNK_CRC_LEN: usize = 4;
const IHDR_PAYLOAD_LEN: usize = 13;
const PNG_UINT_31_MAX: u32 = 0x7fff_ffff;

#[derive(Debug, PartialEq, Eq)]
pub enum PngParseError {
    InvalidSignature {
        offset: usize,
        expected: u8,
        actual: u8,
    },
    InvalidChunkLength {
        length: u32,
    },
    InvalidChunkType {
        bytes: [u8; 4],
    },
    InvalidIhdrLength {
        length: usize,
    },
    InvalidIhdrDimensions {
        width: u32,
        height: u32,
    },
    InvalidIhdrColorType {
        color_type: u8,
    },
    InvalidIhdrBitDepthColorType {
        bit_depth: u8,
        color_type: u8,
    },
    InvalidIhdrCompressionMethod {
        method: u8,
    },
    InvalidIhdrFilterMethod {
        method: u8,
    },
    InvalidIhdrInterlaceMethod {
        method: u8,
    },
    MissingIhdr,
    IhdrNotFirst,
    DuplicateIhdr,
    MissingIdatBeforeIend,
    MissingIend,
    ChunkAfterIend {
        chunk_type: [u8; 4],
    },
    UnknownCriticalChunk {
        chunk_type: [u8; 4],
    },
    TrailingBytesAfterIend {
        byte_count: usize,
    },
    UnexpectedEndOfInput {
        buffered_len: usize,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ChunkType {
    bytes: [u8; 4],
}

impl ChunkType {
    pub fn from_bytes(bytes: [u8; 4]) -> Result<Self, PngParseError> {
        if !is_ascii_letter(bytes[0])
            || !is_ascii_letter(bytes[1])
            || !bytes[2].is_ascii_uppercase()
            || !is_ascii_letter(bytes[3])
        {
            return Err(PngParseError::InvalidChunkType { bytes });
        }

        Ok(Self { bytes })
    }

    pub fn bytes(self) -> [u8; 4] {
        self.bytes
    }

    pub fn is_critical(self) -> bool {
        self.bytes[0].is_ascii_uppercase()
    }

    pub fn is_ancillary(self) -> bool {
        !self.is_critical()
    }

    pub fn is_public(self) -> bool {
        self.bytes[1].is_ascii_uppercase()
    }

    pub fn is_private(self) -> bool {
        !self.is_public()
    }

    pub fn has_valid_reserved_bit(self) -> bool {
        self.bytes[2].is_ascii_uppercase()
    }

    pub fn is_safe_to_copy(self) -> bool {
        self.bytes[3].is_ascii_lowercase()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ChunkHeader {
    pub length: u32,
    pub chunk_type: ChunkType,
}

impl ChunkHeader {
    pub fn parse(bytes: [u8; CHUNK_HEADER_LEN]) -> Result<Self, PngParseError> {
        let length = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        if length > PNG_UINT_31_MAX {
            return Err(PngParseError::InvalidChunkLength { length });
        }

        let chunk_type = ChunkType::from_bytes([bytes[4], bytes[5], bytes[6], bytes[7]])?;

        Ok(Self { length, chunk_type })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Chunk {
    pub header: ChunkHeader,
    pub payload: Vec<u8>,
    pub crc: u32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IhdrColorType {
    Grayscale,
    Truecolor,
    Indexed,
    GrayscaleAlpha,
    TruecolorAlpha,
}

impl IhdrColorType {
    fn from_byte(byte: u8) -> Result<Self, PngParseError> {
        match byte {
            0 => Ok(Self::Grayscale),
            2 => Ok(Self::Truecolor),
            3 => Ok(Self::Indexed),
            4 => Ok(Self::GrayscaleAlpha),
            6 => Ok(Self::TruecolorAlpha),
            _ => Err(PngParseError::InvalidIhdrColorType { color_type: byte }),
        }
    }

    fn byte(self) -> u8 {
        match self {
            Self::Grayscale => 0,
            Self::Truecolor => 2,
            Self::Indexed => 3,
            Self::GrayscaleAlpha => 4,
            Self::TruecolorAlpha => 6,
        }
    }

    fn allows_bit_depth(self, bit_depth: u8) -> bool {
        match self {
            Self::Grayscale => matches!(bit_depth, 1 | 2 | 4 | 8 | 16),
            Self::Truecolor => matches!(bit_depth, 8 | 16),
            Self::Indexed => matches!(bit_depth, 1 | 2 | 4 | 8),
            Self::GrayscaleAlpha | Self::TruecolorAlpha => matches!(bit_depth, 8 | 16),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PngCompressionMethod {
    Deflate,
}

impl PngCompressionMethod {
    fn from_byte(byte: u8) -> Result<Self, PngParseError> {
        match byte {
            0 => Ok(Self::Deflate),
            _ => Err(PngParseError::InvalidIhdrCompressionMethod { method: byte }),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PngFilterMethod {
    Adaptive,
}

impl PngFilterMethod {
    fn from_byte(byte: u8) -> Result<Self, PngParseError> {
        match byte {
            0 => Ok(Self::Adaptive),
            _ => Err(PngParseError::InvalidIhdrFilterMethod { method: byte }),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PngInterlaceMethod {
    None,
    Adam7,
}

impl PngInterlaceMethod {
    fn from_byte(byte: u8) -> Result<Self, PngParseError> {
        match byte {
            0 => Ok(Self::None),
            1 => Ok(Self::Adam7),
            _ => Err(PngParseError::InvalidIhdrInterlaceMethod { method: byte }),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Ihdr {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: IhdrColorType,
    pub compression_method: PngCompressionMethod,
    pub filter_method: PngFilterMethod,
    pub interlace_method: PngInterlaceMethod,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PngStructureSummary {
    pub width: u32,
    pub height: u32,
    pub idat_count: usize,
    pub ancillary_count: usize,
}

impl Ihdr {
    pub fn parse(payload: &[u8]) -> Result<Self, PngParseError> {
        if payload.len() != IHDR_PAYLOAD_LEN {
            return Err(PngParseError::InvalidIhdrLength {
                length: payload.len(),
            });
        }

        let width = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
        let height = u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]);

        if width == 0 || height == 0 {
            return Err(PngParseError::InvalidIhdrDimensions { width, height });
        }

        let bit_depth = payload[8];
        let color_type = IhdrColorType::from_byte(payload[9])?;

        if !color_type.allows_bit_depth(bit_depth) {
            return Err(PngParseError::InvalidIhdrBitDepthColorType {
                bit_depth,
                color_type: color_type.byte(),
            });
        }

        Ok(Self {
            width,
            height,
            bit_depth,
            color_type,
            compression_method: PngCompressionMethod::from_byte(payload[10])?,
            filter_method: PngFilterMethod::from_byte(payload[11])?,
            interlace_method: PngInterlaceMethod::from_byte(payload[12])?,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseOutcome {
    NeedMoreData,
    SignatureComplete,
    ChunkHeader(ChunkHeader),
    Chunk(Chunk),
}

#[derive(Debug, Default)]
pub struct PngStreamParser {
    buffer: Vec<u8>,
    signature_complete: bool,
    pending_header: Option<ChunkHeader>,
}

impl PngStreamParser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn buffered_len(&self) -> usize {
        self.buffer.len()
    }

    pub fn feed(&mut self, input: &[u8]) -> Result<ParseOutcome, PngParseError> {
        self.buffer.extend_from_slice(input);
        self.next_outcome()
    }

    pub fn next_outcome(&mut self) -> Result<ParseOutcome, PngParseError> {
        if !self.signature_complete {
            return self.parse_signature();
        }

        if self.pending_header.is_some() {
            return self.parse_pending_chunk();
        }

        self.parse_chunk_header()
    }

    fn parse_signature(&mut self) -> Result<ParseOutcome, PngParseError> {
        for (offset, actual) in self.buffer.iter().take(PNG_SIGNATURE_LEN).enumerate() {
            if *actual != PNG_SIGNATURE[offset] {
                return Err(PngParseError::InvalidSignature {
                    offset,
                    expected: PNG_SIGNATURE[offset],
                    actual: *actual,
                });
            }
        }

        if self.buffer.len() < PNG_SIGNATURE_LEN {
            return Ok(ParseOutcome::NeedMoreData);
        }

        self.buffer.drain(..PNG_SIGNATURE_LEN);
        self.signature_complete = true;
        Ok(ParseOutcome::SignatureComplete)
    }

    fn parse_chunk_header(&mut self) -> Result<ParseOutcome, PngParseError> {
        if self.buffer.len() < CHUNK_HEADER_LEN {
            return Ok(ParseOutcome::NeedMoreData);
        }

        let header = [
            self.buffer[0],
            self.buffer[1],
            self.buffer[2],
            self.buffer[3],
            self.buffer[4],
            self.buffer[5],
            self.buffer[6],
            self.buffer[7],
        ];
        let header = ChunkHeader::parse(header)?;
        self.buffer.drain(..CHUNK_HEADER_LEN);
        self.pending_header = Some(header);

        Ok(ParseOutcome::ChunkHeader(header))
    }

    fn parse_pending_chunk(&mut self) -> Result<ParseOutcome, PngParseError> {
        let header = self
            .pending_header
            .expect("pending chunk header exists before payload parsing");
        let payload_len = header.length as usize;
        let full_chunk_len = payload_len + CHUNK_CRC_LEN;

        if self.buffer.len() < full_chunk_len {
            return Ok(ParseOutcome::NeedMoreData);
        }

        let payload = self.buffer.drain(..payload_len).collect();
        let crc = u32::from_be_bytes([
            self.buffer[0],
            self.buffer[1],
            self.buffer[2],
            self.buffer[3],
        ]);
        self.buffer.drain(..CHUNK_CRC_LEN);
        self.pending_header = None;

        Ok(ParseOutcome::Chunk(Chunk {
            header,
            payload,
            crc,
        }))
    }
}

pub fn png_sig_cmp(sig: &[u8], start: usize, num_to_check: usize) -> i32 {
    let mut count = num_to_check.min(PNG_SIGNATURE_LEN);

    if count < 1 || start > PNG_SIGNATURE_LEN - 1 {
        return -1;
    }

    if start + count > PNG_SIGNATURE_LEN {
        count = PNG_SIGNATURE_LEN - start;
    }

    if sig.len() < start + count {
        return -1;
    }

    for offset in start..start + count {
        let actual = sig[offset];
        let expected = PNG_SIGNATURE[offset];

        if actual != expected {
            return i32::from(actual) - i32::from(expected);
        }
    }

    0
}

pub fn validate_png_chunks(chunks: &[Chunk]) -> Result<PngStructureSummary, PngParseError> {
    let Some(first_chunk) = chunks.first() else {
        return Err(PngParseError::MissingIhdr);
    };

    if first_chunk.header.chunk_type.bytes() != *b"IHDR" {
        return Err(PngParseError::IhdrNotFirst);
    }

    let mut ihdr = None;
    let mut idat_count = 0;
    let mut ancillary_count = 0;
    let mut seen_iend = false;

    for chunk in chunks {
        let chunk_type = chunk.header.chunk_type;
        let chunk_type_bytes = chunk_type.bytes();

        if seen_iend {
            return Err(PngParseError::ChunkAfterIend {
                chunk_type: chunk_type_bytes,
            });
        }

        match &chunk_type_bytes {
            b"IHDR" => {
                if ihdr.is_some() {
                    return Err(PngParseError::DuplicateIhdr);
                }

                ihdr = Some(Ihdr::parse(&chunk.payload)?);
            }
            b"IDAT" => {
                idat_count += 1;
            }
            b"IEND" => {
                if idat_count == 0 {
                    return Err(PngParseError::MissingIdatBeforeIend);
                }

                seen_iend = true;
            }
            b"PLTE" => {}
            _ if chunk_type.is_ancillary() => {
                ancillary_count += 1;
            }
            _ => {
                return Err(PngParseError::UnknownCriticalChunk {
                    chunk_type: chunk_type_bytes,
                });
            }
        }
    }

    let ihdr = ihdr.ok_or(PngParseError::MissingIhdr)?;

    if idat_count == 0 {
        return Err(PngParseError::MissingIdatBeforeIend);
    }

    if !seen_iend {
        return Err(PngParseError::MissingIend);
    }

    Ok(PngStructureSummary {
        width: ihdr.width,
        height: ihdr.height,
        idat_count,
        ancillary_count,
    })
}

pub fn validate_png_structure(input: &[u8]) -> Result<PngStructureSummary, PngParseError> {
    let mut parser = PngStreamParser::new();
    let mut chunks = Vec::new();

    match parser.feed(input)? {
        ParseOutcome::SignatureComplete => {}
        ParseOutcome::NeedMoreData => {
            return Err(PngParseError::UnexpectedEndOfInput {
                buffered_len: parser.buffered_len(),
            });
        }
        ParseOutcome::ChunkHeader(_) | ParseOutcome::Chunk(_) => unreachable!(
            "a newly initialized parser cannot emit a chunk before signature completion"
        ),
    }

    loop {
        match parser.next_outcome()? {
            ParseOutcome::NeedMoreData => {
                if parser.buffered_len() == 0 {
                    break;
                }

                return Err(PngParseError::UnexpectedEndOfInput {
                    buffered_len: parser.buffered_len(),
                });
            }
            ParseOutcome::SignatureComplete => unreachable!("signature is parsed once"),
            ParseOutcome::ChunkHeader(_) => {}
            ParseOutcome::Chunk(chunk) => {
                let is_iend = chunk.header.chunk_type.bytes() == *b"IEND";
                chunks.push(chunk);

                if is_iend {
                    if parser.buffered_len() > 0 {
                        return Err(PngParseError::TrailingBytesAfterIend {
                            byte_count: parser.buffered_len(),
                        });
                    }

                    break;
                }
            }
        }
    }

    validate_png_chunks(&chunks)
}

fn is_ascii_letter(byte: u8) -> bool {
    byte.is_ascii_uppercase() || byte.is_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ihdr_payload(
        width: u32,
        height: u32,
        bit_depth: u8,
        color_type: u8,
        compression_method: u8,
        filter_method: u8,
        interlace_method: u8,
    ) -> [u8; IHDR_PAYLOAD_LEN] {
        let width_bytes = width.to_be_bytes();
        let height_bytes = height.to_be_bytes();

        [
            width_bytes[0],
            width_bytes[1],
            width_bytes[2],
            width_bytes[3],
            height_bytes[0],
            height_bytes[1],
            height_bytes[2],
            height_bytes[3],
            bit_depth,
            color_type,
            compression_method,
            filter_method,
            interlace_method,
        ]
    }

    fn chunk(chunk_type: [u8; 4], payload: Vec<u8>) -> Chunk {
        Chunk {
            header: ChunkHeader {
                length: payload.len() as u32,
                chunk_type: ChunkType::from_bytes(chunk_type).expect("test chunk type is valid"),
            },
            payload,
            crc: 0,
        }
    }

    fn minimal_chunks() -> Vec<Chunk> {
        vec![
            chunk(*b"IHDR", ihdr_payload(32, 16, 8, 2, 0, 0, 0).to_vec()),
            chunk(*b"IDAT", vec![1, 2, 3]),
            chunk(*b"IEND", Vec::new()),
        ]
    }

    fn append_chunk_bytes(input: &mut Vec<u8>, chunk: &Chunk) {
        input.extend_from_slice(&chunk.header.length.to_be_bytes());
        input.extend_from_slice(&chunk.header.chunk_type.bytes());
        input.extend_from_slice(&chunk.payload);
        input.extend_from_slice(&chunk.crc.to_be_bytes());
    }

    fn minimal_png_bytes() -> Vec<u8> {
        let chunks = minimal_chunks();
        let mut input = PNG_SIGNATURE.to_vec();

        for chunk in &chunks {
            append_chunk_bytes(&mut input, chunk);
        }

        input
    }

    #[test]
    fn signature_comparison_accepts_full_and_partial_matches() {
        assert_eq!(png_sig_cmp(&PNG_SIGNATURE, 0, PNG_SIGNATURE.len()), 0);
        assert_eq!(png_sig_cmp(&PNG_SIGNATURE, 0, 4), 0);
        assert_eq!(png_sig_cmp(&PNG_SIGNATURE, 4, 4), 0);
        assert_eq!(png_sig_cmp(&PNG_SIGNATURE, 6, 8), 0);
    }

    #[test]
    fn signature_comparison_rejects_invalid_bytes() {
        let mut corrupted = PNG_SIGNATURE;
        corrupted[1] = b'X';

        assert_ne!(png_sig_cmp(&corrupted, 0, PNG_SIGNATURE.len()), 0);
        assert_eq!(png_sig_cmp(&PNG_SIGNATURE, 8, 1), -1);
        assert_eq!(png_sig_cmp(&PNG_SIGNATURE, 0, 0), -1);
    }

    #[test]
    fn chunk_type_property_helpers_reflect_png_bits() {
        let ihdr = ChunkType::from_bytes(*b"IHDR").expect("IHDR is valid");
        assert!(ihdr.is_critical());
        assert!(!ihdr.is_ancillary());
        assert!(ihdr.is_public());
        assert!(!ihdr.is_private());
        assert!(ihdr.has_valid_reserved_bit());
        assert!(!ihdr.is_safe_to_copy());

        let text = ChunkType::from_bytes(*b"tEXt").expect("tEXt is valid");
        assert!(text.is_ancillary());
        assert!(text.is_public());
        assert!(text.has_valid_reserved_bit());
        assert!(text.is_safe_to_copy());

        let private = ChunkType::from_bytes(*b"vpAg").expect("vpAg is valid");
        assert!(private.is_private());
    }

    #[test]
    fn chunk_type_rejects_invalid_reserved_bit() {
        assert_eq!(
            ChunkType::from_bytes(*b"IHdR"),
            Err(PngParseError::InvalidChunkType { bytes: *b"IHdR" })
        );
        assert_eq!(
            ChunkType::from_bytes(*b"IH1R"),
            Err(PngParseError::InvalidChunkType { bytes: *b"IH1R" })
        );
    }

    #[test]
    fn chunk_header_parse_returns_owned_metadata() {
        let header =
            ChunkHeader::parse([0, 0, 0, 13, b'I', b'H', b'D', b'R']).expect("valid IHDR header");

        assert_eq!(header.length, 13);
        assert_eq!(header.chunk_type.bytes(), *b"IHDR");
    }

    #[test]
    fn chunk_header_rejects_length_overflow() {
        assert_eq!(
            ChunkHeader::parse([0x80, 0, 0, 0, b'I', b'H', b'D', b'R']),
            Err(PngParseError::InvalidChunkLength {
                length: 0x8000_0000
            })
        );
    }

    #[test]
    fn ihdr_parse_returns_payload_metadata_fields() {
        let payload = ihdr_payload(800, 600, 8, 6, 0, 0, 1);

        assert_eq!(
            Ihdr::parse(&payload),
            Ok(Ihdr {
                width: 800,
                height: 600,
                bit_depth: 8,
                color_type: IhdrColorType::TruecolorAlpha,
                compression_method: PngCompressionMethod::Deflate,
                filter_method: PngFilterMethod::Adaptive,
                interlace_method: PngInterlaceMethod::Adam7,
            })
        );
    }

    #[test]
    fn ihdr_payload_length_must_be_exactly_thirteen_bytes() {
        assert_eq!(
            Ihdr::parse(&ihdr_payload(1, 1, 8, 2, 0, 0, 0)[..12]),
            Err(PngParseError::InvalidIhdrLength { length: 12 })
        );

        let mut payload = ihdr_payload(1, 1, 8, 2, 0, 0, 0).to_vec();
        payload.push(0);

        assert_eq!(
            Ihdr::parse(&payload),
            Err(PngParseError::InvalidIhdrLength { length: 14 })
        );
    }

    #[test]
    fn ihdr_dimensions_must_be_nonzero() {
        assert_eq!(
            Ihdr::parse(&ihdr_payload(0, 1, 8, 2, 0, 0, 0)),
            Err(PngParseError::InvalidIhdrDimensions {
                width: 0,
                height: 1,
            })
        );
        assert_eq!(
            Ihdr::parse(&ihdr_payload(1, 0, 8, 2, 0, 0, 0)),
            Err(PngParseError::InvalidIhdrDimensions {
                width: 1,
                height: 0,
            })
        );
    }

    #[test]
    fn ihdr_accepts_common_bit_depth_color_type_combinations() {
        let valid_pairs = [
            (1, 0),
            (2, 0),
            (4, 0),
            (8, 0),
            (16, 0),
            (8, 2),
            (16, 2),
            (1, 3),
            (2, 3),
            (4, 3),
            (8, 3),
            (8, 4),
            (16, 4),
            (8, 6),
            (16, 6),
        ];

        for (bit_depth, color_type) in valid_pairs {
            assert!(
                Ihdr::parse(&ihdr_payload(1, 1, bit_depth, color_type, 0, 0, 0)).is_ok(),
                "bit depth {bit_depth} and color type {color_type} should be valid"
            );
        }
    }

    #[test]
    fn ihdr_rejects_invalid_bit_depth_color_type_combinations() {
        let invalid_pairs = [(3, 0), (4, 2), (16, 3), (4, 4), (4, 6)];

        for (bit_depth, color_type) in invalid_pairs {
            assert_eq!(
                Ihdr::parse(&ihdr_payload(1, 1, bit_depth, color_type, 0, 0, 0)),
                Err(PngParseError::InvalidIhdrBitDepthColorType {
                    bit_depth,
                    color_type,
                })
            );
        }

        assert_eq!(
            Ihdr::parse(&ihdr_payload(1, 1, 8, 5, 0, 0, 0)),
            Err(PngParseError::InvalidIhdrColorType { color_type: 5 })
        );
    }

    #[test]
    fn ihdr_rejects_invalid_compression_filter_and_interlace_methods() {
        assert_eq!(
            Ihdr::parse(&ihdr_payload(1, 1, 8, 2, 1, 0, 0)),
            Err(PngParseError::InvalidIhdrCompressionMethod { method: 1 })
        );
        assert_eq!(
            Ihdr::parse(&ihdr_payload(1, 1, 8, 2, 0, 1, 0)),
            Err(PngParseError::InvalidIhdrFilterMethod { method: 1 })
        );
        assert_eq!(
            Ihdr::parse(&ihdr_payload(1, 1, 8, 2, 0, 0, 2)),
            Err(PngParseError::InvalidIhdrInterlaceMethod { method: 2 })
        );
    }

    #[test]
    fn parser_reports_invalid_signature_bytes() {
        let mut parser = PngStreamParser::new();
        let mut corrupted = PNG_SIGNATURE;
        corrupted[0] = 0;

        assert_eq!(
            parser.feed(&corrupted),
            Err(PngParseError::InvalidSignature {
                offset: 0,
                expected: PNG_SIGNATURE[0],
                actual: 0,
            })
        );
    }

    #[test]
    fn parser_handles_partial_signature_and_chunk_header_input() {
        let mut parser = PngStreamParser::new();

        assert_eq!(
            parser.feed(&PNG_SIGNATURE[..3]),
            Ok(ParseOutcome::NeedMoreData)
        );
        assert_eq!(parser.buffered_len(), 3);

        assert_eq!(
            parser.feed(&PNG_SIGNATURE[3..]),
            Ok(ParseOutcome::SignatureComplete)
        );
        assert_eq!(parser.buffered_len(), 0);

        assert_eq!(parser.feed(&[0, 0, 0]), Ok(ParseOutcome::NeedMoreData));
        assert_eq!(parser.buffered_len(), 3);

        let outcome = parser.feed(&[13, b'I', b'H', b'D', b'R']);
        assert_eq!(
            outcome,
            Ok(ParseOutcome::ChunkHeader(ChunkHeader {
                length: 13,
                chunk_type: ChunkType::from_bytes(*b"IHDR").expect("IHDR is valid"),
            }))
        );
        assert_eq!(parser.buffered_len(), 0);
    }

    #[test]
    fn parser_keeps_chunk_header_bytes_when_signature_arrives_with_extra_input() {
        let mut parser = PngStreamParser::new();
        let mut input = PNG_SIGNATURE.to_vec();
        input.extend_from_slice(&[0, 0, 0, 13, b'I', b'H', b'D', b'R']);

        assert_eq!(parser.feed(&input), Ok(ParseOutcome::SignatureComplete));
        assert_eq!(parser.buffered_len(), CHUNK_HEADER_LEN);
        assert_eq!(
            parser.next_outcome(),
            Ok(ParseOutcome::ChunkHeader(ChunkHeader {
                length: 13,
                chunk_type: ChunkType::from_bytes(*b"IHDR").expect("IHDR is valid"),
            }))
        );
        assert_eq!(parser.buffered_len(), 0);
    }

    #[test]
    fn parser_returns_owned_chunk_after_payload_and_crc_are_available() {
        let mut parser = PngStreamParser::new();
        let mut input = PNG_SIGNATURE.to_vec();
        input.extend_from_slice(&[0, 0, 0, 5, b't', b'E', b'X', b't']);
        input.extend_from_slice(b"hello");
        input.extend_from_slice(&[0x12, 0x34, 0x56, 0x78]);

        assert_eq!(parser.feed(&input), Ok(ParseOutcome::SignatureComplete));
        assert_eq!(
            parser.next_outcome(),
            Ok(ParseOutcome::ChunkHeader(ChunkHeader {
                length: 5,
                chunk_type: ChunkType::from_bytes(*b"tEXt").expect("tEXt is valid"),
            }))
        );
        assert_eq!(
            parser.next_outcome(),
            Ok(ParseOutcome::Chunk(Chunk {
                header: ChunkHeader {
                    length: 5,
                    chunk_type: ChunkType::from_bytes(*b"tEXt").expect("tEXt is valid"),
                },
                payload: b"hello".to_vec(),
                crc: 0x1234_5678,
            }))
        );
        assert_eq!(parser.buffered_len(), 0);
    }

    #[test]
    fn parser_reports_need_more_data_for_partial_payload_and_crc() {
        let mut parser = PngStreamParser::new();

        assert_eq!(
            parser.feed(&PNG_SIGNATURE),
            Ok(ParseOutcome::SignatureComplete)
        );
        assert_eq!(
            parser.feed(&[0, 0, 0, 3, b'I', b'D', b'A', b'T']),
            Ok(ParseOutcome::ChunkHeader(ChunkHeader {
                length: 3,
                chunk_type: ChunkType::from_bytes(*b"IDAT").expect("IDAT is valid"),
            }))
        );

        assert_eq!(parser.feed(&[1, 2]), Ok(ParseOutcome::NeedMoreData));
        assert_eq!(parser.buffered_len(), 2);
        assert_eq!(parser.feed(&[3, 0xaa]), Ok(ParseOutcome::NeedMoreData));
        assert_eq!(parser.buffered_len(), 4);
        assert_eq!(parser.feed(&[0xbb, 0xcc]), Ok(ParseOutcome::NeedMoreData));
        assert_eq!(parser.buffered_len(), 6);
        assert_eq!(
            parser.feed(&[0xdd]),
            Ok(ParseOutcome::Chunk(Chunk {
                header: ChunkHeader {
                    length: 3,
                    chunk_type: ChunkType::from_bytes(*b"IDAT").expect("IDAT is valid"),
                },
                payload: vec![1, 2, 3],
                crc: 0xaabb_ccdd,
            }))
        );
        assert_eq!(parser.buffered_len(), 0);
    }

    #[test]
    fn parser_can_return_following_chunk_header_after_full_chunk() {
        let mut parser = PngStreamParser::new();
        let mut input = PNG_SIGNATURE.to_vec();
        input.extend_from_slice(&[0, 0, 0, 0, b'I', b'D', b'A', b'T']);
        input.extend_from_slice(&[0, 0, 0, 1]);
        input.extend_from_slice(&[0, 0, 0, 0, b'I', b'E', b'N', b'D']);

        assert_eq!(parser.feed(&input), Ok(ParseOutcome::SignatureComplete));
        assert_eq!(
            parser.next_outcome(),
            Ok(ParseOutcome::ChunkHeader(ChunkHeader {
                length: 0,
                chunk_type: ChunkType::from_bytes(*b"IDAT").expect("IDAT is valid"),
            }))
        );
        assert_eq!(
            parser.next_outcome(),
            Ok(ParseOutcome::Chunk(Chunk {
                header: ChunkHeader {
                    length: 0,
                    chunk_type: ChunkType::from_bytes(*b"IDAT").expect("IDAT is valid"),
                },
                payload: Vec::new(),
                crc: 1,
            }))
        );
        assert_eq!(
            parser.next_outcome(),
            Ok(ParseOutcome::ChunkHeader(ChunkHeader {
                length: 0,
                chunk_type: ChunkType::from_bytes(*b"IEND").expect("IEND is valid"),
            }))
        );
    }

    #[test]
    fn structure_validator_accepts_ordered_png_chunks_and_returns_summary() {
        assert_eq!(
            validate_png_chunks(&minimal_chunks()),
            Ok(PngStructureSummary {
                width: 32,
                height: 16,
                idat_count: 1,
                ancillary_count: 0,
            })
        );
    }

    #[test]
    fn structure_validator_uses_signature_and_stream_parser_for_full_png_bytes() {
        assert_eq!(
            validate_png_structure(&minimal_png_bytes()),
            Ok(PngStructureSummary {
                width: 32,
                height: 16,
                idat_count: 1,
                ancillary_count: 0,
            })
        );
    }

    #[test]
    fn structure_validator_requires_ihdr_first_and_exactly_once() {
        let mut chunks = minimal_chunks();
        chunks.insert(0, chunk(*b"tEXt", b"before IHDR".to_vec()));

        assert_eq!(
            validate_png_chunks(&chunks),
            Err(PngParseError::IhdrNotFirst)
        );

        let mut duplicate = minimal_chunks();
        duplicate.insert(
            1,
            chunk(*b"IHDR", ihdr_payload(1, 1, 8, 2, 0, 0, 0).to_vec()),
        );

        assert_eq!(
            validate_png_chunks(&duplicate),
            Err(PngParseError::DuplicateIhdr)
        );
    }

    #[test]
    fn structure_validator_requires_idat_before_iend_and_iend_final() {
        let chunks_without_idat = vec![
            chunk(*b"IHDR", ihdr_payload(1, 1, 8, 2, 0, 0, 0).to_vec()),
            chunk(*b"IEND", Vec::new()),
        ];

        assert_eq!(
            validate_png_chunks(&chunks_without_idat),
            Err(PngParseError::MissingIdatBeforeIend)
        );

        let mut chunks_after_iend = minimal_chunks();
        chunks_after_iend.push(chunk(*b"tEXt", b"late".to_vec()));

        assert_eq!(
            validate_png_chunks(&chunks_after_iend),
            Err(PngParseError::ChunkAfterIend {
                chunk_type: *b"tEXt",
            })
        );
    }

    #[test]
    fn structure_validator_accepts_unknown_ancillary_chunks() {
        let chunks = vec![
            chunk(*b"IHDR", ihdr_payload(2, 2, 8, 6, 0, 0, 0).to_vec()),
            chunk(*b"vpAg", b"private ancillary payload".to_vec()),
            chunk(*b"IDAT", vec![0]),
            chunk(*b"IEND", Vec::new()),
        ];

        assert_eq!(
            validate_png_chunks(&chunks),
            Ok(PngStructureSummary {
                width: 2,
                height: 2,
                idat_count: 1,
                ancillary_count: 1,
            })
        );
    }

    #[test]
    fn structure_validator_rejects_unknown_critical_chunks() {
        let chunks = vec![
            chunk(*b"IHDR", ihdr_payload(1, 1, 8, 2, 0, 0, 0).to_vec()),
            chunk(*b"ABCD", Vec::new()),
            chunk(*b"IDAT", vec![0]),
            chunk(*b"IEND", Vec::new()),
        ];

        assert_eq!(
            validate_png_chunks(&chunks),
            Err(PngParseError::UnknownCriticalChunk {
                chunk_type: *b"ABCD",
            })
        );
    }

    #[test]
    fn structure_validator_rejects_trailing_bytes_after_iend() {
        let mut input = minimal_png_bytes();
        input.extend_from_slice(b"extra");

        assert_eq!(
            validate_png_structure(&input),
            Err(PngParseError::TrailingBytesAfterIend { byte_count: 5 })
        );
    }

    #[test]
    fn structure_validator_validates_ihdr_payload() {
        let chunks = vec![
            chunk(*b"IHDR", ihdr_payload(0, 1, 8, 2, 0, 0, 0).to_vec()),
            chunk(*b"IDAT", vec![0]),
            chunk(*b"IEND", Vec::new()),
        ];

        assert_eq!(
            validate_png_chunks(&chunks),
            Err(PngParseError::InvalidIhdrDimensions {
                width: 0,
                height: 1,
            })
        );
    }
}
