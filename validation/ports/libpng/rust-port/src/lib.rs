//! Minimal libpng validation port for PNG signature and chunk-header parsing.
//!
//! This crate intentionally covers only the first parsing boundary: comparing
//! the 8-byte PNG signature, validating 4-byte chunk type names, and extracting
//! owned chunk-header metadata from progressive input.

use std::io::Read;

pub const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

const ADAM7_PASSES: [(u32, u32, u32, u32); 7] = [
    (0, 0, 8, 8),
    (4, 0, 8, 8),
    (0, 4, 4, 8),
    (2, 0, 4, 4),
    (0, 2, 2, 4),
    (1, 0, 2, 2),
    (0, 1, 1, 2),
];

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
    CrcMismatch {
        chunk_type: [u8; 4],
        expected: u32,
        actual: u32,
    },
    UnknownCriticalChunk {
        chunk_type: [u8; 4],
    },
    TrailingBytesAfterIend {
        byte_count: usize,
    },
    MissingImageData,
    UnsupportedDecodeFormat {
        bit_depth: u8,
        color_type: u8,
        interlace_method: u8,
    },
    InflateFailed,
    InvalidInflatedDataLength {
        expected: usize,
        actual: usize,
    },
    InvalidPlteLength {
        length: usize,
    },
    MissingPlte,
    DuplicatePlte,
    PlteAfterIdat,
    PlteNotAllowed {
        color_type: u8,
    },
    InvalidPaletteIndex {
        index: u8,
        palette_len: usize,
    },
    InvalidTrnsLength {
        color_type: u8,
        length: usize,
    },
    TrnsNotAllowed {
        color_type: u8,
    },
    InvalidFilterType {
        row: usize,
        filter_type: u8,
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

    fn byte(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Adam7 => 1,
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

#[derive(Debug, PartialEq, Eq)]
pub struct PngImage {
    pub width: u32,
    pub height: u32,
    pub color_type: IhdrColorType,
    pub bit_depth: u8,
    pub pixels: Vec<u8>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PaletteEntry {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct DecodeLayout {
    filter_bytes_per_pixel: usize,
    row_data_len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Transparency {
    Grayscale { sample: u8 },
    Truecolor { red: u8, green: u8, blue: u8 },
    Indexed { alpha: Vec<u8> },
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
    let mut seen_plte = false;
    let mut seen_iend = false;

    for chunk in chunks {
        validate_chunk_crc(chunk)?;

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
                if matches!(
                    ihdr.map(|ihdr| ihdr.color_type),
                    Some(IhdrColorType::Indexed)
                ) && !seen_plte
                {
                    return Err(PngParseError::MissingPlte);
                }

                idat_count += 1;
            }
            b"IEND" => {
                if idat_count == 0 {
                    return Err(PngParseError::MissingIdatBeforeIend);
                }

                seen_iend = true;
            }
            b"PLTE" => {
                if seen_plte {
                    return Err(PngParseError::DuplicatePlte);
                }

                if idat_count > 0 {
                    return Err(PngParseError::PlteAfterIdat);
                }

                let ihdr = ihdr.ok_or(PngParseError::MissingIhdr)?;
                if matches!(
                    ihdr.color_type,
                    IhdrColorType::Grayscale | IhdrColorType::GrayscaleAlpha
                ) {
                    return Err(PngParseError::PlteNotAllowed {
                        color_type: ihdr.color_type.byte(),
                    });
                }

                parse_plte(&chunk.payload)?;
                seen_plte = true;
            }
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

pub fn calculate_chunk_crc(chunk_type: ChunkType, payload: &[u8]) -> u32 {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(&chunk_type.bytes());
    hasher.update(payload);
    hasher.finalize()
}

pub fn validate_chunk_crc(chunk: &Chunk) -> Result<(), PngParseError> {
    let expected = calculate_chunk_crc(chunk.header.chunk_type, &chunk.payload);

    if chunk.crc != expected {
        return Err(PngParseError::CrcMismatch {
            chunk_type: chunk.header.chunk_type.bytes(),
            expected,
            actual: chunk.crc,
        });
    }

    Ok(())
}

pub fn parse_png_chunks(input: &[u8]) -> Result<Vec<Chunk>, PngParseError> {
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

    Ok(chunks)
}

pub fn validate_png_structure(input: &[u8]) -> Result<PngStructureSummary, PngParseError> {
    let chunks = parse_png_chunks(input)?;

    validate_png_chunks(&chunks)
}

pub fn decode_png_image(input: &[u8]) -> Result<PngImage, PngParseError> {
    let chunks = parse_png_chunks(input)?;
    validate_png_chunks(&chunks)?;

    let ihdr_chunk = chunks
        .iter()
        .find(|chunk| chunk.header.chunk_type.bytes() == *b"IHDR")
        .ok_or(PngParseError::MissingIhdr)?;
    let ihdr = Ihdr::parse(&ihdr_chunk.payload)?;
    let layout = decode_layout(ihdr)?;
    let transparency = find_transparency(&chunks, ihdr)?;

    if transparency.is_some() && ihdr.bit_depth != 8 {
        return Err(PngParseError::UnsupportedDecodeFormat {
            bit_depth: ihdr.bit_depth,
            color_type: ihdr.color_type.byte(),
            interlace_method: ihdr.interlace_method.byte(),
        });
    }

    let mut compressed = Vec::new();
    for chunk in chunks
        .iter()
        .filter(|chunk| chunk.header.chunk_type.bytes() == *b"IDAT")
    {
        compressed.extend_from_slice(&chunk.payload);
    }

    if compressed.is_empty() {
        return Err(PngParseError::MissingImageData);
    }

    let mut decoder = flate2::read::ZlibDecoder::new(compressed.as_slice());
    let mut inflated = Vec::new();
    decoder
        .read_to_end(&mut inflated)
        .map_err(|_| PngParseError::InflateFailed)?;

    let reconstructed = match ihdr.interlace_method {
        PngInterlaceMethod::None => reconstruct_scanlines(
            &inflated,
            ihdr.height,
            layout.row_data_len,
            layout.filter_bytes_per_pixel,
        )?,
        PngInterlaceMethod::Adam7 => {
            reconstruct_adam7_scanlines(&inflated, ihdr, layout.filter_bytes_per_pixel)?
        }
    };
    let pixels = match ihdr.color_type {
        IhdrColorType::Grayscale => match transparency {
            Some(Transparency::Grayscale { sample }) => {
                expand_grayscale_transparency(&reconstructed, sample)
            }
            _ => expand_packed_grayscale_samples(&reconstructed, ihdr.width, ihdr.bit_depth),
        },
        IhdrColorType::Truecolor => match transparency {
            Some(Transparency::Truecolor { red, green, blue }) => {
                expand_truecolor_transparency(&reconstructed, red, green, blue)
            }
            _ => reconstructed,
        },
        IhdrColorType::Indexed => {
            let palette = find_palette(&chunks)?;
            let indices = expand_packed_indices(&reconstructed, ihdr.width, ihdr.bit_depth);

            match transparency {
                Some(Transparency::Indexed { alpha }) => {
                    expand_indexed_pixels_with_alpha(&indices, &palette, &alpha)?
                }
                _ => expand_indexed_pixels(&indices, &palette)?,
            }
        }
        IhdrColorType::GrayscaleAlpha | IhdrColorType::TruecolorAlpha => reconstructed,
    };

    Ok(PngImage {
        width: ihdr.width,
        height: ihdr.height,
        color_type: ihdr.color_type,
        bit_depth: ihdr.bit_depth,
        pixels,
    })
}

fn decode_layout(ihdr: Ihdr) -> Result<DecodeLayout, PngParseError> {
    let channel_count = match ihdr.color_type {
        IhdrColorType::Grayscale | IhdrColorType::Indexed => 1,
        IhdrColorType::Truecolor => 3,
        IhdrColorType::GrayscaleAlpha => 2,
        IhdrColorType::TruecolorAlpha => 4,
    };

    let bits_per_pixel = match ihdr.bit_depth {
        1 | 2 | 4
            if matches!(
                ihdr.color_type,
                IhdrColorType::Grayscale | IhdrColorType::Indexed
            ) =>
        {
            ihdr.bit_depth as usize
        }
        8 => channel_count * 8,
        16 if ihdr.color_type != IhdrColorType::Indexed => channel_count * 16,
        _ => Err(PngParseError::UnsupportedDecodeFormat {
            bit_depth: ihdr.bit_depth,
            color_type: ihdr.color_type.byte(),
            interlace_method: ihdr.interlace_method.byte(),
        })?,
    };

    let row_bits = ihdr.width as usize * bits_per_pixel;

    Ok(DecodeLayout {
        filter_bytes_per_pixel: bits_per_pixel.div_ceil(8).max(1),
        row_data_len: row_bits.div_ceil(8),
    })
}

pub fn parse_plte(payload: &[u8]) -> Result<Vec<PaletteEntry>, PngParseError> {
    if payload.is_empty() || !payload.len().is_multiple_of(3) || payload.len() > 256 * 3 {
        return Err(PngParseError::InvalidPlteLength {
            length: payload.len(),
        });
    }

    Ok(payload
        .chunks_exact(3)
        .map(|entry| PaletteEntry {
            red: entry[0],
            green: entry[1],
            blue: entry[2],
        })
        .collect())
}

pub fn parse_trns(
    payload: &[u8],
    color_type: IhdrColorType,
) -> Result<Transparency, PngParseError> {
    match color_type {
        IhdrColorType::Grayscale => {
            if payload.len() != 2 {
                return Err(PngParseError::InvalidTrnsLength {
                    color_type: color_type.byte(),
                    length: payload.len(),
                });
            }

            Ok(Transparency::Grayscale { sample: payload[1] })
        }
        IhdrColorType::Truecolor => {
            if payload.len() != 6 {
                return Err(PngParseError::InvalidTrnsLength {
                    color_type: color_type.byte(),
                    length: payload.len(),
                });
            }

            Ok(Transparency::Truecolor {
                red: payload[1],
                green: payload[3],
                blue: payload[5],
            })
        }
        IhdrColorType::Indexed => {
            if payload.is_empty() || payload.len() > 256 {
                return Err(PngParseError::InvalidTrnsLength {
                    color_type: color_type.byte(),
                    length: payload.len(),
                });
            }

            Ok(Transparency::Indexed {
                alpha: payload.to_vec(),
            })
        }
        IhdrColorType::GrayscaleAlpha | IhdrColorType::TruecolorAlpha => {
            Err(PngParseError::TrnsNotAllowed {
                color_type: color_type.byte(),
            })
        }
    }
}

fn find_transparency(chunks: &[Chunk], ihdr: Ihdr) -> Result<Option<Transparency>, PngParseError> {
    chunks
        .iter()
        .find(|chunk| chunk.header.chunk_type.bytes() == *b"tRNS")
        .map(|chunk| parse_trns(&chunk.payload, ihdr.color_type))
        .transpose()
}

fn find_palette(chunks: &[Chunk]) -> Result<Vec<PaletteEntry>, PngParseError> {
    let plte = chunks
        .iter()
        .find(|chunk| chunk.header.chunk_type.bytes() == *b"PLTE")
        .ok_or(PngParseError::MissingPlte)?;

    parse_plte(&plte.payload)
}

fn expand_indexed_pixels(
    indices: &[u8],
    palette: &[PaletteEntry],
) -> Result<Vec<u8>, PngParseError> {
    let mut pixels = Vec::with_capacity(indices.len() * 3);

    for index in indices {
        let entry = palette.get(*index as usize).ok_or({
            PngParseError::InvalidPaletteIndex {
                index: *index,
                palette_len: palette.len(),
            }
        })?;

        pixels.extend_from_slice(&[entry.red, entry.green, entry.blue]);
    }

    Ok(pixels)
}

fn expand_packed_grayscale_samples(pixels: &[u8], width: u32, bit_depth: u8) -> Vec<u8> {
    if bit_depth >= 8 {
        return pixels.to_vec();
    }

    expand_packed_samples(pixels, width, bit_depth)
        .into_iter()
        .map(|sample| scale_sample_to_u8(sample, bit_depth))
        .collect()
}

fn expand_packed_indices(indices: &[u8], width: u32, bit_depth: u8) -> Vec<u8> {
    if bit_depth >= 8 {
        return indices.to_vec();
    }

    expand_packed_samples(indices, width, bit_depth)
}

fn expand_packed_samples(input: &[u8], width: u32, bit_depth: u8) -> Vec<u8> {
    let samples_per_byte = 8 / bit_depth as usize;
    let mask = (1 << bit_depth) - 1;
    let mut samples = Vec::with_capacity(width as usize);

    for byte in input {
        for sample_offset in 0..samples_per_byte {
            if samples.len() == width as usize {
                return samples;
            }

            let shift = 8 - bit_depth as usize * (sample_offset + 1);
            samples.push((byte >> shift) & mask);
        }
    }

    samples
}

fn scale_sample_to_u8(sample: u8, bit_depth: u8) -> u8 {
    let max_sample = (1 << bit_depth) - 1;
    ((u16::from(sample) * 255) / max_sample as u16) as u8
}

fn expand_grayscale_transparency(pixels: &[u8], transparent_sample: u8) -> Vec<u8> {
    let mut expanded = Vec::with_capacity(pixels.len() * 2);

    for sample in pixels {
        let alpha = if *sample == transparent_sample {
            0
        } else {
            255
        };
        expanded.extend_from_slice(&[*sample, alpha]);
    }

    expanded
}

fn expand_truecolor_transparency(
    pixels: &[u8],
    transparent_red: u8,
    transparent_green: u8,
    transparent_blue: u8,
) -> Vec<u8> {
    let mut expanded = Vec::with_capacity(pixels.len() / 3 * 4);

    for pixel in pixels.chunks_exact(3) {
        let alpha = if pixel == [transparent_red, transparent_green, transparent_blue] {
            0
        } else {
            255
        };

        expanded.extend_from_slice(&[pixel[0], pixel[1], pixel[2], alpha]);
    }

    expanded
}

fn expand_indexed_pixels_with_alpha(
    indices: &[u8],
    palette: &[PaletteEntry],
    alpha: &[u8],
) -> Result<Vec<u8>, PngParseError> {
    let mut pixels = Vec::with_capacity(indices.len() * 4);

    for index in indices {
        let entry = palette.get(*index as usize).ok_or({
            PngParseError::InvalidPaletteIndex {
                index: *index,
                palette_len: palette.len(),
            }
        })?;
        let alpha = alpha.get(*index as usize).copied().unwrap_or(255);

        pixels.extend_from_slice(&[entry.red, entry.green, entry.blue, alpha]);
    }

    Ok(pixels)
}

fn reconstruct_scanlines(
    inflated: &[u8],
    height: u32,
    row_len: usize,
    filter_bytes_per_pixel: usize,
) -> Result<Vec<u8>, PngParseError> {
    let expected = (row_len + 1) * height as usize;

    if inflated.len() != expected {
        return Err(PngParseError::InvalidInflatedDataLength {
            expected,
            actual: inflated.len(),
        });
    }

    let mut pixels = vec![0; row_len * height as usize];

    for row in 0..height as usize {
        let source_start = row * (row_len + 1);
        let filter_type = inflated[source_start];
        let scanline = &inflated[source_start + 1..source_start + 1 + row_len];
        let target_start = row * row_len;

        for column in 0..row_len {
            let raw = scanline[column];
            let left = if column >= filter_bytes_per_pixel {
                pixels[target_start + column - filter_bytes_per_pixel]
            } else {
                0
            };
            let up = if row > 0 {
                pixels[target_start + column - row_len]
            } else {
                0
            };
            let up_left = if row > 0 && column >= filter_bytes_per_pixel {
                pixels[target_start + column - row_len - filter_bytes_per_pixel]
            } else {
                0
            };

            pixels[target_start + column] = match filter_type {
                0 => raw,
                1 => raw.wrapping_add(left),
                2 => raw.wrapping_add(up),
                3 => raw.wrapping_add(((u16::from(left) + u16::from(up)) / 2) as u8),
                4 => raw.wrapping_add(paeth_predictor(left, up, up_left)),
                _ => return Err(PngParseError::InvalidFilterType { row, filter_type }),
            };
        }
    }

    Ok(pixels)
}

fn reconstruct_adam7_scanlines(
    inflated: &[u8],
    ihdr: Ihdr,
    bytes_per_pixel: usize,
) -> Result<Vec<u8>, PngParseError> {
    if ihdr.bit_depth < 8 {
        return Err(PngParseError::UnsupportedDecodeFormat {
            bit_depth: ihdr.bit_depth,
            color_type: ihdr.color_type.byte(),
            interlace_method: ihdr.interlace_method.byte(),
        });
    }

    let output_len = ihdr.width as usize * ihdr.height as usize * bytes_per_pixel;
    let mut output = vec![0; output_len];
    let mut offset = 0;

    for (start_x, start_y, step_x, step_y) in ADAM7_PASSES {
        let pass_width = adam7_pass_size(ihdr.width, start_x, step_x);
        let pass_height = adam7_pass_size(ihdr.height, start_y, step_y);

        if pass_width == 0 || pass_height == 0 {
            continue;
        }

        let row_len = pass_width as usize * bytes_per_pixel;
        let pass_len = (row_len + 1) * pass_height as usize;

        if inflated.len() < offset + pass_len {
            return Err(PngParseError::InvalidInflatedDataLength {
                expected: offset + pass_len,
                actual: inflated.len(),
            });
        }

        let pass_pixels = reconstruct_scanlines(
            &inflated[offset..offset + pass_len],
            pass_height,
            row_len,
            bytes_per_pixel,
        )?;
        offset += pass_len;

        for pass_y in 0..pass_height as usize {
            for pass_x in 0..pass_width as usize {
                let image_x = start_x as usize + pass_x * step_x as usize;
                let image_y = start_y as usize + pass_y * step_y as usize;
                let source_start = (pass_y * pass_width as usize + pass_x) * bytes_per_pixel;
                let target_start = (image_y * ihdr.width as usize + image_x) * bytes_per_pixel;

                output[target_start..target_start + bytes_per_pixel]
                    .copy_from_slice(&pass_pixels[source_start..source_start + bytes_per_pixel]);
            }
        }
    }

    if offset != inflated.len() {
        return Err(PngParseError::InvalidInflatedDataLength {
            expected: offset,
            actual: inflated.len(),
        });
    }

    Ok(output)
}

fn adam7_pass_size(size: u32, start: u32, step: u32) -> u32 {
    if size <= start {
        0
    } else {
        (size - start).div_ceil(step)
    }
}

fn paeth_predictor(left: u8, up: u8, up_left: u8) -> u8 {
    let left = i32::from(left);
    let up = i32::from(up);
    let up_left = i32::from(up_left);
    let prediction = left + up - up_left;
    let left_distance = (prediction - left).abs();
    let up_distance = (prediction - up).abs();
    let up_left_distance = (prediction - up_left).abs();

    if left_distance <= up_distance && left_distance <= up_left_distance {
        left as u8
    } else if up_distance <= up_left_distance {
        up as u8
    } else {
        up_left as u8
    }
}

fn is_ascii_letter(byte: u8) -> bool {
    byte.is_ascii_uppercase() || byte.is_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

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
        let chunk_type = ChunkType::from_bytes(chunk_type).expect("test chunk type is valid");
        let crc = calculate_chunk_crc(chunk_type, &payload);

        Chunk {
            header: ChunkHeader {
                length: payload.len() as u32,
                chunk_type,
            },
            payload,
            crc,
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

    fn png_bytes_from_chunks(chunks: &[Chunk]) -> Vec<u8> {
        let mut input = PNG_SIGNATURE.to_vec();

        for chunk in chunks {
            append_chunk_bytes(&mut input, chunk);
        }

        input
    }

    fn minimal_png_bytes() -> Vec<u8> {
        png_bytes_from_chunks(&minimal_chunks())
    }

    fn zlib_compress(input: &[u8]) -> Vec<u8> {
        let mut encoder =
            flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder
            .write_all(input)
            .expect("test scanline bytes should compress");
        encoder.finish().expect("test zlib stream should finish")
    }

    fn image_png_bytes(width: u32, height: u32, color_type: u8, scanlines: &[u8]) -> Vec<u8> {
        image_png_bytes_with_bit_depth(width, height, 8, color_type, scanlines)
    }

    fn image_png_bytes_with_bit_depth(
        width: u32,
        height: u32,
        bit_depth: u8,
        color_type: u8,
        scanlines: &[u8],
    ) -> Vec<u8> {
        image_png_bytes_with_interlace(width, height, bit_depth, color_type, 0, scanlines)
    }

    fn image_png_bytes_with_interlace(
        width: u32,
        height: u32,
        bit_depth: u8,
        color_type: u8,
        interlace_method: u8,
        scanlines: &[u8],
    ) -> Vec<u8> {
        let chunks = vec![
            chunk(
                *b"IHDR",
                ihdr_payload(width, height, bit_depth, color_type, 0, 0, interlace_method).to_vec(),
            ),
            chunk(*b"IDAT", zlib_compress(scanlines)),
            chunk(*b"IEND", Vec::new()),
        ];

        png_bytes_from_chunks(&chunks)
    }

    fn image_png_bytes_with_extra_chunks(
        width: u32,
        height: u32,
        color_type: u8,
        extra_chunks: Vec<Chunk>,
        scanlines: &[u8],
    ) -> Vec<u8> {
        let mut chunks = vec![chunk(
            *b"IHDR",
            ihdr_payload(width, height, 8, color_type, 0, 0, 0).to_vec(),
        )];
        chunks.extend(extra_chunks);
        chunks.push(chunk(*b"IDAT", zlib_compress(scanlines)));
        chunks.push(chunk(*b"IEND", Vec::new()));

        png_bytes_from_chunks(&chunks)
    }

    fn indexed_png_bytes(width: u32, height: u32, palette: Vec<u8>, scanlines: &[u8]) -> Vec<u8> {
        let chunks = vec![
            chunk(
                *b"IHDR",
                ihdr_payload(width, height, 8, 3, 0, 0, 0).to_vec(),
            ),
            chunk(*b"PLTE", palette),
            chunk(*b"IDAT", zlib_compress(scanlines)),
            chunk(*b"IEND", Vec::new()),
        ];

        png_bytes_from_chunks(&chunks)
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
    fn chunk_crc_validation_accepts_matching_crc() {
        let chunk = chunk(*b"tEXt", b"hello".to_vec());

        assert_eq!(validate_chunk_crc(&chunk), Ok(()));
    }

    #[test]
    fn chunk_crc_validation_rejects_mismatched_crc() {
        let mut chunks = minimal_chunks();
        chunks[1].crc ^= 1;

        assert_eq!(
            validate_png_chunks(&chunks),
            Err(PngParseError::CrcMismatch {
                chunk_type: *b"IDAT",
                expected: calculate_chunk_crc(chunks[1].header.chunk_type, &chunks[1].payload),
                actual: chunks[1].crc,
            })
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

    #[test]
    fn structure_validator_requires_plte_before_indexed_idat() {
        let chunks = vec![
            chunk(*b"IHDR", ihdr_payload(1, 1, 8, 3, 0, 0, 0).to_vec()),
            chunk(*b"IDAT", vec![0]),
            chunk(*b"IEND", Vec::new()),
        ];

        assert_eq!(
            validate_png_chunks(&chunks),
            Err(PngParseError::MissingPlte)
        );
    }

    #[test]
    fn structure_validator_rejects_duplicate_plte() {
        let chunks = vec![
            chunk(*b"IHDR", ihdr_payload(1, 1, 8, 3, 0, 0, 0).to_vec()),
            chunk(*b"PLTE", vec![255, 0, 0]),
            chunk(*b"PLTE", vec![0, 255, 0]),
            chunk(*b"IDAT", vec![0]),
            chunk(*b"IEND", Vec::new()),
        ];

        assert_eq!(
            validate_png_chunks(&chunks),
            Err(PngParseError::DuplicatePlte)
        );
    }

    #[test]
    fn structure_validator_rejects_plte_after_idat() {
        let chunks = vec![
            chunk(*b"IHDR", ihdr_payload(1, 1, 8, 2, 0, 0, 0).to_vec()),
            chunk(*b"IDAT", vec![0]),
            chunk(*b"PLTE", vec![255, 0, 0]),
            chunk(*b"IEND", Vec::new()),
        ];

        assert_eq!(
            validate_png_chunks(&chunks),
            Err(PngParseError::PlteAfterIdat)
        );
    }

    #[test]
    fn structure_validator_rejects_plte_for_grayscale() {
        let chunks = vec![
            chunk(*b"IHDR", ihdr_payload(1, 1, 8, 0, 0, 0, 0).to_vec()),
            chunk(*b"PLTE", vec![255, 0, 0]),
            chunk(*b"IDAT", vec![0]),
            chunk(*b"IEND", Vec::new()),
        ];

        assert_eq!(
            validate_png_chunks(&chunks),
            Err(PngParseError::PlteNotAllowed { color_type: 0 })
        );
    }

    #[test]
    fn decode_png_image_reads_tiny_grayscale_pixels() {
        let input = image_png_bytes(2, 2, 0, &[0, 10, 20, 0, 30, 40]);

        assert_eq!(
            decode_png_image(&input),
            Ok(PngImage {
                width: 2,
                height: 2,
                color_type: IhdrColorType::Grayscale,
                bit_depth: 8,
                pixels: vec![10, 20, 30, 40],
            })
        );
    }

    #[test]
    fn decode_png_image_expands_one_bit_grayscale_samples() {
        let input = image_png_bytes_with_bit_depth(4, 1, 1, 0, &[0, 0b1010_0000]);

        assert_eq!(
            decode_png_image(&input),
            Ok(PngImage {
                width: 4,
                height: 1,
                color_type: IhdrColorType::Grayscale,
                bit_depth: 1,
                pixels: vec![255, 0, 255, 0],
            })
        );
    }

    #[test]
    fn decode_png_image_expands_four_bit_grayscale_samples() {
        let input = image_png_bytes_with_bit_depth(3, 1, 4, 0, &[0, 0x0f, 0x80]);

        assert_eq!(
            decode_png_image(&input),
            Ok(PngImage {
                width: 3,
                height: 1,
                color_type: IhdrColorType::Grayscale,
                bit_depth: 4,
                pixels: vec![0, 255, 136],
            })
        );
    }

    #[test]
    fn decode_png_image_preserves_16_bit_grayscale_sample_bytes() {
        let input = image_png_bytes_with_bit_depth(2, 1, 16, 0, &[0, 0x12, 0x34, 0xab, 0xcd]);

        assert_eq!(
            decode_png_image(&input),
            Ok(PngImage {
                width: 2,
                height: 1,
                color_type: IhdrColorType::Grayscale,
                bit_depth: 16,
                pixels: vec![0x12, 0x34, 0xab, 0xcd],
            })
        );
    }

    #[test]
    fn decode_png_image_reconstructs_adam7_grayscale_pixels() {
        let adam7_scanlines = [
            0, 1, // pass 1: (0,0)
            0, 3, // pass 4: (2,0)
            0, 7, 9, // pass 5: (0,2), (2,2)
            0, 2, // pass 6 row 0: (1,0)
            0, 8, // pass 6 row 1: (1,2)
            0, 4, 5, 6, // pass 7: y=1, x=0..2
        ];
        let input = image_png_bytes_with_interlace(3, 3, 8, 0, 1, &adam7_scanlines);

        assert_eq!(
            decode_png_image(&input),
            Ok(PngImage {
                width: 3,
                height: 3,
                color_type: IhdrColorType::Grayscale,
                bit_depth: 8,
                pixels: vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
            })
        );
    }

    #[test]
    fn decode_png_image_reconstructs_truecolor_sub_filter() {
        let input = image_png_bytes(2, 1, 2, &[1, 10, 20, 30, 5, 5, 5]);

        assert_eq!(
            decode_png_image(&input),
            Ok(PngImage {
                width: 2,
                height: 1,
                color_type: IhdrColorType::Truecolor,
                bit_depth: 8,
                pixels: vec![10, 20, 30, 15, 25, 35],
            })
        );
    }

    #[test]
    fn decode_png_image_preserves_16_bit_truecolor_sample_bytes() {
        let input =
            image_png_bytes_with_bit_depth(1, 1, 16, 2, &[0, 0x00, 0x10, 0x00, 0x20, 0x00, 0x30]);

        assert_eq!(
            decode_png_image(&input),
            Ok(PngImage {
                width: 1,
                height: 1,
                color_type: IhdrColorType::Truecolor,
                bit_depth: 16,
                pixels: vec![0x00, 0x10, 0x00, 0x20, 0x00, 0x30],
            })
        );
    }

    #[test]
    fn decode_png_image_reads_tiny_grayscale_alpha_pixels() {
        let input = image_png_bytes(2, 1, 4, &[0, 10, 255, 20, 128]);

        assert_eq!(
            decode_png_image(&input),
            Ok(PngImage {
                width: 2,
                height: 1,
                color_type: IhdrColorType::GrayscaleAlpha,
                bit_depth: 8,
                pixels: vec![10, 255, 20, 128],
            })
        );
    }

    #[test]
    fn decode_png_image_preserves_16_bit_grayscale_alpha_sample_bytes() {
        let input = image_png_bytes_with_bit_depth(1, 1, 16, 4, &[0, 0x00, 0x80, 0xff, 0xff]);

        assert_eq!(
            decode_png_image(&input),
            Ok(PngImage {
                width: 1,
                height: 1,
                color_type: IhdrColorType::GrayscaleAlpha,
                bit_depth: 16,
                pixels: vec![0x00, 0x80, 0xff, 0xff],
            })
        );
    }

    #[test]
    fn decode_png_image_reconstructs_truecolor_alpha_sub_filter() {
        let input = image_png_bytes(2, 1, 6, &[1, 10, 20, 30, 255, 5, 5, 5, 0]);

        assert_eq!(
            decode_png_image(&input),
            Ok(PngImage {
                width: 2,
                height: 1,
                color_type: IhdrColorType::TruecolorAlpha,
                bit_depth: 8,
                pixels: vec![10, 20, 30, 255, 15, 25, 35, 255],
            })
        );
    }

    #[test]
    fn decode_png_image_preserves_16_bit_truecolor_alpha_sample_bytes() {
        let input = image_png_bytes_with_bit_depth(
            1,
            1,
            16,
            6,
            &[0, 0x00, 0x10, 0x00, 0x20, 0x00, 0x30, 0xff, 0xff],
        );

        assert_eq!(
            decode_png_image(&input),
            Ok(PngImage {
                width: 1,
                height: 1,
                color_type: IhdrColorType::TruecolorAlpha,
                bit_depth: 16,
                pixels: vec![0x00, 0x10, 0x00, 0x20, 0x00, 0x30, 0xff, 0xff],
            })
        );
    }

    #[test]
    fn plte_parser_returns_palette_entries() {
        assert_eq!(
            parse_plte(&[255, 0, 0, 0, 255, 0]),
            Ok(vec![
                PaletteEntry {
                    red: 255,
                    green: 0,
                    blue: 0,
                },
                PaletteEntry {
                    red: 0,
                    green: 255,
                    blue: 0,
                },
            ])
        );

        assert_eq!(
            parse_plte(&[255, 0, 0, 1]),
            Err(PngParseError::InvalidPlteLength { length: 4 })
        );
    }

    #[test]
    fn decode_png_image_expands_indexed_pixels_to_rgb() {
        let input = indexed_png_bytes(2, 1, vec![255, 0, 0, 0, 255, 0], &[0, 0, 1]);

        assert_eq!(
            decode_png_image(&input),
            Ok(PngImage {
                width: 2,
                height: 1,
                color_type: IhdrColorType::Indexed,
                bit_depth: 8,
                pixels: vec![255, 0, 0, 0, 255, 0],
            })
        );
    }

    #[test]
    fn decode_png_image_expands_two_bit_indexed_pixels_to_rgb() {
        let chunks = vec![
            chunk(*b"IHDR", ihdr_payload(4, 1, 2, 3, 0, 0, 0).to_vec()),
            chunk(
                *b"PLTE",
                vec![255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255],
            ),
            chunk(*b"IDAT", zlib_compress(&[0, 0b0001_1011])),
            chunk(*b"IEND", Vec::new()),
        ];
        let input = png_bytes_from_chunks(&chunks);

        assert_eq!(
            decode_png_image(&input),
            Ok(PngImage {
                width: 4,
                height: 1,
                color_type: IhdrColorType::Indexed,
                bit_depth: 2,
                pixels: vec![255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255],
            })
        );
    }

    #[test]
    fn decode_png_image_requires_plte_for_indexed_color() {
        let input = image_png_bytes(1, 1, 3, &[0, 0]);

        assert_eq!(decode_png_image(&input), Err(PngParseError::MissingPlte));
    }

    #[test]
    fn decode_png_image_rejects_invalid_palette_index() {
        let input = indexed_png_bytes(1, 1, vec![255, 0, 0], &[0, 1]);

        assert_eq!(
            decode_png_image(&input),
            Err(PngParseError::InvalidPaletteIndex {
                index: 1,
                palette_len: 1,
            })
        );
    }

    #[test]
    fn trns_parser_returns_transparency_metadata() {
        assert_eq!(
            parse_trns(&[0, 20], IhdrColorType::Grayscale),
            Ok(Transparency::Grayscale { sample: 20 })
        );
        assert_eq!(
            parse_trns(&[0, 10, 0, 20, 0, 30], IhdrColorType::Truecolor),
            Ok(Transparency::Truecolor {
                red: 10,
                green: 20,
                blue: 30,
            })
        );
        assert_eq!(
            parse_trns(&[0, 128], IhdrColorType::Indexed),
            Ok(Transparency::Indexed {
                alpha: vec![0, 128],
            })
        );
    }

    #[test]
    fn decode_png_image_expands_grayscale_trns_to_alpha() {
        let input = image_png_bytes_with_extra_chunks(
            2,
            1,
            0,
            vec![chunk(*b"tRNS", vec![0, 20])],
            &[0, 10, 20],
        );

        assert_eq!(
            decode_png_image(&input),
            Ok(PngImage {
                width: 2,
                height: 1,
                color_type: IhdrColorType::Grayscale,
                bit_depth: 8,
                pixels: vec![10, 255, 20, 0],
            })
        );
    }

    #[test]
    fn decode_png_image_expands_truecolor_trns_to_alpha() {
        let input = image_png_bytes_with_extra_chunks(
            2,
            1,
            2,
            vec![chunk(*b"tRNS", vec![0, 10, 0, 20, 0, 30])],
            &[0, 10, 20, 30, 1, 2, 3],
        );

        assert_eq!(
            decode_png_image(&input),
            Ok(PngImage {
                width: 2,
                height: 1,
                color_type: IhdrColorType::Truecolor,
                bit_depth: 8,
                pixels: vec![10, 20, 30, 0, 1, 2, 3, 255],
            })
        );
    }

    #[test]
    fn decode_png_image_expands_indexed_trns_to_rgba() {
        let chunks = vec![
            chunk(*b"IHDR", ihdr_payload(2, 1, 8, 3, 0, 0, 0).to_vec()),
            chunk(*b"PLTE", vec![255, 0, 0, 0, 255, 0]),
            chunk(*b"tRNS", vec![0, 128]),
            chunk(*b"IDAT", zlib_compress(&[0, 0, 1])),
            chunk(*b"IEND", Vec::new()),
        ];
        let input = png_bytes_from_chunks(&chunks);

        assert_eq!(
            decode_png_image(&input),
            Ok(PngImage {
                width: 2,
                height: 1,
                color_type: IhdrColorType::Indexed,
                bit_depth: 8,
                pixels: vec![255, 0, 0, 0, 0, 255, 0, 128],
            })
        );
    }

    #[test]
    fn decode_png_image_rejects_invalid_trns_length() {
        let input =
            image_png_bytes_with_extra_chunks(1, 1, 0, vec![chunk(*b"tRNS", vec![0])], &[0, 10]);

        assert_eq!(
            decode_png_image(&input),
            Err(PngParseError::InvalidTrnsLength {
                color_type: 0,
                length: 1,
            })
        );
    }

    #[test]
    fn decode_png_image_rejects_trns_for_alpha_color_types() {
        let input = image_png_bytes_with_extra_chunks(
            1,
            1,
            6,
            vec![chunk(*b"tRNS", vec![0, 10, 0, 20, 0, 30])],
            &[0, 10, 20, 30, 255],
        );

        assert_eq!(
            decode_png_image(&input),
            Err(PngParseError::TrnsNotAllowed { color_type: 6 })
        );
    }
}
