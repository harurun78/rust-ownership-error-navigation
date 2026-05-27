#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InflateError {
    OutputLimitTooSmall { needed: usize, limit: usize },
    AlreadyFinished,
    InvalidZlibHeader,
    PresetDictionaryUnsupported,
    UnsupportedBlockType { block_type: u8 },
    InvalidStoredBlockLength,
    Adler32Mismatch { expected: u32, actual: u32 },
    TruncatedInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InflateOutcome {
    pub output: Vec<u8>,
    pub consumed: usize,
    pub finished: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InflateProgress {
    pub consumed: usize,
    pub total_out: usize,
}

#[derive(Debug, Default)]
pub struct Inflater {
    output: Vec<u8>,
    finished: bool,
}

impl Inflater {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, input: &[u8]) -> Result<InflateProgress, InflateError> {
        if self.finished {
            return Err(InflateError::AlreadyFinished);
        }

        let decoded = decode_zlib_stored_blocks(input)?;
        let consumed = input.len();
        self.output.extend_from_slice(&decoded);

        Ok(InflateProgress {
            consumed,
            total_out: self.output.len(),
        })
    }

    pub fn finish(mut self) -> Result<InflateOutcome, InflateError> {
        self.finished = true;

        Ok(InflateOutcome {
            consumed: self.output.len(),
            output: self.output,
            finished: true,
        })
    }
}

pub fn inflate_all(input: &[u8]) -> Result<Vec<u8>, InflateError> {
    decode_zlib_stored_blocks(input)
}

pub fn inflate_with_output_limit(input: &[u8], limit: usize) -> Result<Vec<u8>, InflateError> {
    let output = decode_zlib_stored_blocks(input)?;

    if output.len() > limit {
        return Err(InflateError::OutputLimitTooSmall {
            needed: output.len(),
            limit,
        });
    }

    Ok(output)
}

fn decode_zlib_stored_blocks(input: &[u8]) -> Result<Vec<u8>, InflateError> {
    if input.len() < 6 {
        return Err(InflateError::TruncatedInput);
    }
    if input[0] & 0x0f != 8 {
        return Err(InflateError::InvalidZlibHeader);
    }
    if ((u16::from(input[0]) << 8) | u16::from(input[1])) % 31 != 0 {
        return Err(InflateError::InvalidZlibHeader);
    }
    if input[1] & 0x20 != 0 {
        return Err(InflateError::PresetDictionaryUnsupported);
    }

    let mut offset = 2;
    let mut output = Vec::new();

    loop {
        if offset >= input.len().saturating_sub(4) {
            return Err(InflateError::TruncatedInput);
        }

        let block_header = input[offset];
        offset += 1;
        let final_block = block_header & 1 == 1;
        let block_type = (block_header >> 1) & 0b11;
        if block_type != 0 {
            return Err(InflateError::UnsupportedBlockType { block_type });
        }

        if offset + 4 > input.len().saturating_sub(4) {
            return Err(InflateError::TruncatedInput);
        }

        let len = u16::from_le_bytes([input[offset], input[offset + 1]]);
        let nlen = u16::from_le_bytes([input[offset + 2], input[offset + 3]]);
        offset += 4;
        if len != !nlen {
            return Err(InflateError::InvalidStoredBlockLength);
        }

        let len = usize::from(len);
        if offset + len > input.len().saturating_sub(4) {
            return Err(InflateError::TruncatedInput);
        }

        output.extend_from_slice(&input[offset..offset + len]);
        offset += len;

        if final_block {
            break;
        }
    }

    if offset + 4 != input.len() {
        return Err(InflateError::TruncatedInput);
    }

    let expected = u32::from_be_bytes([
        input[offset],
        input[offset + 1],
        input[offset + 2],
        input[offset + 3],
    ]);
    let actual = adler32(&output);
    if actual != expected {
        return Err(InflateError::Adler32Mismatch { expected, actual });
    }

    Ok(output)
}

fn adler32(input: &[u8]) -> u32 {
    const MOD_ADLER: u32 = 65_521;
    let mut a = 1_u32;
    let mut b = 0_u32;

    for byte in input {
        a = (a + u32::from(*byte)) % MOD_ADLER;
        b = (b + a) % MOD_ADLER;
    }

    (b << 16) | a
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stored_zlib(payload: &[u8]) -> Vec<u8> {
        stored_zlib_blocks(&[payload])
    }

    fn stored_zlib_blocks(blocks: &[&[u8]]) -> Vec<u8> {
        let mut input = vec![0x78, 0x01];
        let mut payload = Vec::new();

        for (index, block) in blocks.iter().enumerate() {
            let final_block = u8::from(index + 1 == blocks.len());
            let len = block.len() as u16;
            input.push(final_block);
            input.extend_from_slice(&len.to_le_bytes());
            input.extend_from_slice(&(!len).to_le_bytes());
            input.extend_from_slice(block);
            payload.extend_from_slice(block);
        }

        input.extend_from_slice(&adler32(&payload).to_be_bytes());
        input
    }

    #[test]
    fn rust_native_inflate_all_decodes_stored_block_to_owned_output() {
        assert_eq!(
            inflate_all(&stored_zlib(b"stream bytes")),
            Ok(b"stream bytes".to_vec())
        );
    }

    #[test]
    fn rust_native_inflater_uses_short_borrow_updates() {
        let mut inflater = Inflater::new();

        assert_eq!(
            inflater.update(&stored_zlib(b"abc")),
            Ok(InflateProgress {
                consumed: stored_zlib(b"abc").len(),
                total_out: 3,
            })
        );
        assert_eq!(
            inflater.update(&stored_zlib(b"def")),
            Ok(InflateProgress {
                consumed: stored_zlib(b"def").len(),
                total_out: 6,
            })
        );
        assert_eq!(
            inflater.finish(),
            Ok(InflateOutcome {
                output: b"abcdef".to_vec(),
                consumed: 6,
                finished: true,
            })
        );
    }

    #[test]
    fn rust_native_output_limit_is_an_error_value() {
        assert_eq!(
            inflate_with_output_limit(&stored_zlib(b"abcdef"), 3),
            Err(InflateError::OutputLimitTooSmall {
                needed: 6,
                limit: 3,
            })
        );
    }

    #[test]
    fn rust_native_reports_invalid_stored_block_checksum() {
        let mut input = stored_zlib(b"abc");
        let last = input.len() - 1;
        input[last] ^= 1;

        assert!(matches!(
            inflate_all(&input),
            Err(InflateError::Adler32Mismatch { .. })
        ));
    }

    #[test]
    fn rust_native_decodes_multiple_stored_blocks() {
        assert_eq!(
            inflate_all(&stored_zlib_blocks(&[b"abc", b"def"])),
            Ok(b"abcdef".to_vec())
        );
    }
}
