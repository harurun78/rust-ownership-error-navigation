#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InflateError {
    OutputLimitTooSmall { needed: usize, limit: usize },
    AlreadyFinished,
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

        self.output.extend_from_slice(input);

        Ok(InflateProgress {
            consumed: input.len(),
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
    Ok(input.to_vec())
}

pub fn inflate_with_output_limit(input: &[u8], limit: usize) -> Result<Vec<u8>, InflateError> {
    if input.len() > limit {
        return Err(InflateError::OutputLimitTooSmall {
            needed: input.len(),
            limit,
        });
    }

    Ok(input.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_native_inflate_all_returns_owned_output() {
        assert_eq!(inflate_all(b"stream bytes"), Ok(b"stream bytes".to_vec()));
    }

    #[test]
    fn rust_native_inflater_uses_short_borrow_updates() {
        let mut inflater = Inflater::new();

        assert_eq!(
            inflater.update(b"abc"),
            Ok(InflateProgress {
                consumed: 3,
                total_out: 3,
            })
        );
        assert_eq!(
            inflater.update(b"def"),
            Ok(InflateProgress {
                consumed: 3,
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
            inflate_with_output_limit(b"abcdef", 3),
            Err(InflateError::OutputLimitTooSmall {
                needed: 6,
                limit: 3,
            })
        );
    }
}
