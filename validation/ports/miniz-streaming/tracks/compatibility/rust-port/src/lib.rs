#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MzStatus {
    Ok,
    StreamEnd,
    BufError,
    StreamError,
    DataError,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MzFlush {
    NoFlush,
    Finish,
}

pub type MzAlloc = fn(items: usize, size: usize) -> Option<Vec<u8>>;
pub type MzFree = fn(buffer: Vec<u8>);

#[derive(Debug)]
pub struct MzStream<'a> {
    pub next_in: Option<&'a [u8]>,
    pub avail_in: usize,
    pub next_out: Option<&'a mut [u8]>,
    pub avail_out: usize,
    pub total_in: usize,
    pub total_out: usize,
    pub zalloc: Option<MzAlloc>,
    pub zfree: Option<MzFree>,
    initialized: bool,
    ended: bool,
    decoded: bool,
    decoded_output: Vec<u8>,
}

impl<'a> MzStream<'a> {
    pub fn new(input: &'a [u8], output: &'a mut [u8]) -> Self {
        let output_len = output.len();

        Self {
            next_in: Some(input),
            avail_in: input.len(),
            next_out: Some(output),
            avail_out: output_len,
            total_in: 0,
            total_out: 0,
            zalloc: None,
            zfree: None,
            initialized: false,
            ended: false,
            decoded: false,
            decoded_output: Vec::new(),
        }
    }

    pub fn set_allocators(&mut self, zalloc: MzAlloc, zfree: MzFree) {
        self.zalloc = Some(zalloc);
        self.zfree = Some(zfree);
    }
}

pub fn mz_inflate_init(stream: &mut MzStream<'_>) -> MzStatus {
    if stream.ended {
        return MzStatus::StreamError;
    }

    stream.initialized = true;
    MzStatus::Ok
}

pub fn mz_inflate(stream: &mut MzStream<'_>, flush: MzFlush) -> MzStatus {
    if !stream.initialized || stream.ended {
        return MzStatus::StreamError;
    }

    let Some(input) = stream.next_in else {
        return MzStatus::StreamError;
    };
    let Some(output) = stream.next_out.as_deref_mut() else {
        return MzStatus::StreamError;
    };

    if !stream.decoded {
        match decode_zlib_stored_blocks(input) {
            Ok(decoded_output) => {
                stream.decoded_output = decoded_output;
                stream.decoded = true;
                stream.total_in = input.len();
                stream.avail_in = 0;
            }
            Err(()) => return MzStatus::DataError,
        }
    }

    let input_remaining = stream.decoded_output.len().saturating_sub(stream.total_out);
    let output_remaining = stream
        .avail_out
        .min(output.len().saturating_sub(stream.total_out));
    let byte_count = input_remaining.min(output_remaining);

    if byte_count == 0 {
        return if input_remaining == 0 && flush == MzFlush::Finish {
            MzStatus::StreamEnd
        } else {
            MzStatus::BufError
        };
    }

    let output_start = stream.total_out;
    output[output_start..output_start + byte_count]
        .copy_from_slice(&stream.decoded_output[output_start..output_start + byte_count]);

    stream.total_out += byte_count;
    stream.avail_out -= byte_count;

    if stream.total_out == stream.decoded_output.len() && flush == MzFlush::Finish {
        MzStatus::StreamEnd
    } else {
        MzStatus::Ok
    }
}

pub fn mz_inflate_end(stream: &mut MzStream<'_>) -> MzStatus {
    if !stream.initialized || stream.ended {
        return MzStatus::StreamError;
    }

    stream.ended = true;
    stream.initialized = false;
    MzStatus::Ok
}

fn decode_zlib_stored_blocks(input: &[u8]) -> Result<Vec<u8>, ()> {
    if input.len() < 6 {
        return Err(());
    }
    if input[0] & 0x0f != 8 {
        return Err(());
    }
    if ((u16::from(input[0]) << 8) | u16::from(input[1])) % 31 != 0 {
        return Err(());
    }
    if input[1] & 0x20 != 0 {
        return Err(());
    }

    let mut offset = 2;
    let mut output = Vec::new();

    loop {
        if offset >= input.len().saturating_sub(4) {
            return Err(());
        }

        let block_header = input[offset];
        offset += 1;
        let final_block = block_header & 1 == 1;
        let block_type = (block_header >> 1) & 0b11;
        if block_type != 0 {
            return Err(());
        }

        if offset + 4 > input.len().saturating_sub(4) {
            return Err(());
        }

        let len = u16::from_le_bytes([input[offset], input[offset + 1]]);
        let nlen = u16::from_le_bytes([input[offset + 2], input[offset + 3]]);
        offset += 4;
        if len != !nlen {
            return Err(());
        }

        let len = usize::from(len);
        if offset + len > input.len().saturating_sub(4) {
            return Err(());
        }

        output.extend_from_slice(&input[offset..offset + len]);
        offset += len;

        if final_block {
            break;
        }
    }

    if offset + 4 != input.len() {
        return Err(());
    }

    let expected = u32::from_be_bytes([
        input[offset],
        input[offset + 1],
        input[offset + 2],
        input[offset + 3],
    ]);
    if adler32(&output) != expected {
        return Err(());
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

    fn test_alloc(items: usize, size: usize) -> Option<Vec<u8>> {
        Some(vec![0; items * size])
    }

    fn test_free(_: Vec<u8>) {}

    #[test]
    fn c_style_stream_decodes_stored_block_to_caller_output() {
        let input = stored_zlib(b"stream bytes");
        let mut output = [0_u8; 16];
        let mut stream = MzStream::new(&input, &mut output);
        stream.set_allocators(test_alloc, test_free);

        assert_eq!(mz_inflate_init(&mut stream), MzStatus::Ok);
        assert_eq!(
            mz_inflate(&mut stream, MzFlush::Finish),
            MzStatus::StreamEnd
        );
        assert_eq!(stream.total_in, input.len());
        assert_eq!(stream.total_out, b"stream bytes".len());
        assert_eq!(mz_inflate_end(&mut stream), MzStatus::Ok);

        drop(stream);
        assert_eq!(&output[..b"stream bytes".len()], b"stream bytes");
    }

    #[test]
    fn c_style_stream_reports_output_buffer_pressure() {
        let input = stored_zlib(b"abcdef");
        let mut output = [0_u8; 3];
        let mut stream = MzStream::new(&input, &mut output);

        assert_eq!(mz_inflate_init(&mut stream), MzStatus::Ok);
        assert_eq!(mz_inflate(&mut stream, MzFlush::Finish), MzStatus::Ok);
        assert_eq!(mz_inflate(&mut stream, MzFlush::Finish), MzStatus::BufError);
        assert_eq!(stream.total_in, input.len());
        assert_eq!(stream.total_out, 3);
    }

    #[test]
    fn c_style_stream_rejects_calls_outside_lifecycle() {
        let input = stored_zlib(b"abc");
        let mut output = [0_u8; 3];
        let mut stream = MzStream::new(&input, &mut output);

        assert_eq!(
            mz_inflate(&mut stream, MzFlush::NoFlush),
            MzStatus::StreamError
        );
        assert_eq!(mz_inflate_end(&mut stream), MzStatus::StreamError);
    }

    #[test]
    fn c_style_stream_rejects_invalid_stored_block_checksum() {
        let mut input = stored_zlib(b"abc");
        let last = input.len() - 1;
        input[last] ^= 1;
        let mut output = [0_u8; 3];
        let mut stream = MzStream::new(&input, &mut output);

        assert_eq!(mz_inflate_init(&mut stream), MzStatus::Ok);
        assert_eq!(
            mz_inflate(&mut stream, MzFlush::Finish),
            MzStatus::DataError
        );
    }

    #[test]
    fn c_style_stream_decodes_multiple_stored_blocks() {
        let input = stored_zlib_blocks(&[b"abc", b"def"]);
        let mut output = [0_u8; 6];
        let mut stream = MzStream::new(&input, &mut output);

        assert_eq!(mz_inflate_init(&mut stream), MzStatus::Ok);
        assert_eq!(
            mz_inflate(&mut stream, MzFlush::Finish),
            MzStatus::StreamEnd
        );

        drop(stream);
        assert_eq!(&output, b"abcdef");
    }
}
