#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MzStatus {
    Ok,
    StreamEnd,
    BufError,
    StreamError,
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

    let input_remaining = stream
        .avail_in
        .min(input.len().saturating_sub(stream.total_in));
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

    let input_start = stream.total_in;
    let output_start = stream.total_out;
    output[output_start..output_start + byte_count]
        .copy_from_slice(&input[input_start..input_start + byte_count]);

    stream.total_in += byte_count;
    stream.total_out += byte_count;
    stream.avail_in -= byte_count;
    stream.avail_out -= byte_count;

    if stream.avail_in == 0 && flush == MzFlush::Finish {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_alloc(items: usize, size: usize) -> Option<Vec<u8>> {
        Some(vec![0; items * size])
    }

    fn test_free(_: Vec<u8>) {}

    #[test]
    fn c_style_stream_copies_input_to_caller_output() {
        let input = b"stream bytes";
        let mut output = [0_u8; 16];
        let mut stream = MzStream::new(input, &mut output);
        stream.set_allocators(test_alloc, test_free);

        assert_eq!(mz_inflate_init(&mut stream), MzStatus::Ok);
        assert_eq!(
            mz_inflate(&mut stream, MzFlush::Finish),
            MzStatus::StreamEnd
        );
        assert_eq!(stream.total_in, input.len());
        assert_eq!(stream.total_out, input.len());
        assert_eq!(mz_inflate_end(&mut stream), MzStatus::Ok);

        drop(stream);
        assert_eq!(&output[..input.len()], input);
    }

    #[test]
    fn c_style_stream_reports_output_buffer_pressure() {
        let input = b"abcdef";
        let mut output = [0_u8; 3];
        let mut stream = MzStream::new(input, &mut output);

        assert_eq!(mz_inflate_init(&mut stream), MzStatus::Ok);
        assert_eq!(mz_inflate(&mut stream, MzFlush::Finish), MzStatus::Ok);
        assert_eq!(mz_inflate(&mut stream, MzFlush::Finish), MzStatus::BufError);
        assert_eq!(stream.total_in, 3);
        assert_eq!(stream.total_out, 3);
    }

    #[test]
    fn c_style_stream_rejects_calls_outside_lifecycle() {
        let input = b"abc";
        let mut output = [0_u8; 3];
        let mut stream = MzStream::new(input, &mut output);

        assert_eq!(
            mz_inflate(&mut stream, MzFlush::NoFlush),
            MzStatus::StreamError
        );
        assert_eq!(mz_inflate_end(&mut stream), MzStatus::StreamError);
    }
}
