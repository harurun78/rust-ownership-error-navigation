#[derive(Debug, PartialEq, Eq)]
pub enum MinifyError {
    UnterminatedString { pos: usize },
    UnterminatedBlockComment { pos: usize },
}

pub fn minify_json(input: &str) -> Result<String, MinifyError> {
    let mut output = String::new();
    let mut chars = input.char_indices().peekable();

    while let Some((pos, ch)) = chars.next() {
        match ch {
            ' ' | '\n' | '\r' | '\t' => {}
            '"' => copy_string(pos, &mut chars, &mut output)?,
            '/' => match chars.peek() {
                Some((_, '/')) => {
                    chars.next();
                    skip_line_comment(&mut chars);
                }
                Some((_, '*')) => {
                    chars.next();
                    skip_block_comment(pos, &mut chars)?;
                }
                _ => output.push('/'),
            },
            _ => output.push(ch),
        }
    }

    Ok(output)
}

fn copy_string<I>(
    start: usize,
    chars: &mut std::iter::Peekable<I>,
    output: &mut String,
) -> Result<(), MinifyError>
where
    I: Iterator<Item = (usize, char)>,
{
    output.push('"');

    while let Some((_, ch)) = chars.next() {
        output.push(ch);
        match ch {
            '"' => return Ok(()),
            '\\' => match chars.next() {
                Some((_, escaped)) => output.push(escaped),
                None => return Err(MinifyError::UnterminatedString { pos: start }),
            },
            _ => {}
        }
    }

    Err(MinifyError::UnterminatedString { pos: start })
}

fn skip_line_comment<I>(chars: &mut std::iter::Peekable<I>)
where
    I: Iterator<Item = (usize, char)>,
{
    for (_, ch) in chars.by_ref() {
        if ch == '\n' || ch == '\r' {
            break;
        }
    }
}

fn skip_block_comment<I>(
    start: usize,
    chars: &mut std::iter::Peekable<I>,
) -> Result<(), MinifyError>
where
    I: Iterator<Item = (usize, char)>,
{
    while let Some((_, ch)) = chars.next() {
        if ch == '*' && chars.peek().is_some_and(|(_, next)| *next == '/') {
            chars.next();
            return Ok(());
        }
    }

    Err(MinifyError::UnterminatedBlockComment { pos: start })
}
