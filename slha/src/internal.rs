use {Line, SlhaBlock, Decay, ParseResult, Parseable};
use errors::*;

use std::{iter, result, str};

#[derive(Clone, Debug, PartialEq)]
pub enum Segment<'a> {
    Block {
        name: String,
        block: Vec<Line<'a>>,
        scale: Option<f64>,
    },
    Decay {
        pdg_id: i64,
        width: f64,
        decays: Vec<Decay>,
    },
}

pub trait WrappedBlock<E>: Sized {
    type Wrapper: Default;
    fn parse_into<'a>(
        lines: &[Line<'a>],
        scale: Option<f64>,
        wrapped: &mut Self::Wrapper,
        name: &str,
    ) -> result::Result<(), E>;
    fn unwrap(name: &str, wrapped: Self::Wrapper) -> result::Result<Self, E>;
}

impl<T> WrappedBlock<Error> for T
where
    T: SlhaBlock<Error>,
{
    type Wrapper = Option<T>;
    fn parse_into<'a>(
        lines: &[Line<'a>],
        scale: Option<f64>,
        wrapped: &mut Option<T>,
        name: &str,
    ) -> Result<()> {
        match *wrapped {
            None => {
                *wrapped = Some(T::parse(lines, scale).chain_err(|| {
                    ErrorKind::InvalidBlock(name.to_string())
                })?)
            }

            Some(_) => return Err(ErrorKind::DuplicateBlock(name.to_string()).into()),
        }
        Ok(())
    }
    fn unwrap(name: &str, wrapped: Option<T>) -> Result<T> {
        match wrapped {
            Some(block) => Ok(block),
            None => Err(ErrorKind::MissingBlock(name.to_string()).into()),
        }
    }
}

impl<T> WrappedBlock<Error> for Option<T>
where
    T: SlhaBlock<Error>,
{
    type Wrapper = Option<T>;
    fn parse_into<'a>(
        lines: &[Line<'a>],
        scale: Option<f64>,
        wrapped: &mut Option<T>,
        name: &str,
    ) -> Result<()> {
        match *wrapped {
            None => {
                *wrapped = Some(T::parse(lines, scale).chain_err(|| {
                    ErrorKind::InvalidBlock(name.to_string())
                })?)
            }
            Some(_) => return Err(ErrorKind::DuplicateBlock(name.to_string()).into()),
        }
        Ok(())
    }
    fn unwrap(_: &str, wrapped: Option<T>) -> Result<Option<T>> {
        Ok(wrapped)
    }
}

impl<T> WrappedBlock<Error> for Vec<T>
where
    T: SlhaBlock<Error>,
{
    type Wrapper = Vec<T>;
    fn parse_into<'a>(
        lines: &[Line<'a>],
        scale: Option<f64>,
        wrapped: &mut Vec<T>,
        name: &str,
    ) -> Result<()> {
        wrapped.push(T::parse(lines, scale).chain_err(|| {
            ErrorKind::InvalidBlock(name.to_string())
        })?);
        Ok(())
    }
    fn unwrap(name: &str, wrapped: Vec<T>) -> Result<Vec<T>> {
        let mut no_scale = false;
        let mut seen = Vec::new();
        for block in &wrapped {
            if let Some(scale) = block.scale() {
                if no_scale {
                    return Err(ErrorKind::RedefinedBlockWithQ(name.to_string()).into());
                }
                if seen.contains(&scale) {
                    return Err(
                        ErrorKind::DuplicateBlockScale(name.to_string(), scale).into(),
                    );
                }
                seen.push(scale);
            } else {
                no_scale = true;
                if !seen.is_empty() {
                    return Err(ErrorKind::RedefinedBlockWithQ(name.to_string()).into());
                }
            }
        }
        Ok(wrapped)
    }
}

pub fn parse_block_from<'a, B: SlhaBlock<Error>>(
    input: &[Line<'a>],
    scale: Option<f64>,
) -> Result<B> {
    B::parse(input, scale)
}

pub fn parse_segment<'a>(
    input: &mut iter::Peekable<str::Lines<'a>>,
) -> Option<Result<Segment<'a>>> {
    skip_empty_lines(input);
    match input.next() {
        Some(line) => Some(parse_segment_line(line, input)),
        None => None,
    }
}

fn parse_segment_line<'a>(
    line: &'a str,
    input: &mut iter::Peekable<str::Lines<'a>>,
) -> Result<Segment<'a>> {
    if line.starts_with(|c: char| c.is_whitespace()) {
        bail!(ErrorKind::UnexpectedIdent(line.to_string()));
    }
    match next_word(line) {
        Some((kw, rest)) => {
            match kw.to_lowercase().as_ref() {
                "block" => parse_block(rest, input),
                "decay" => parse_decay_table(rest, input),
                kw => bail!(ErrorKind::UnknownSegment(kw.to_string())),
            }
        }
        None => unreachable!("All empty lines have been skipped, so this line MUST NOT be empty."),
    }
}

fn parse_block<'a, Iter>(header: &str, input: &mut iter::Peekable<Iter>) -> Result<Segment<'a>>
where
    Iter: Iterator<Item = &'a str>,
{
    let (name, scale) = parse_block_header(header)?;
    let mut block = Vec::new();
    loop {
        {
            skip_empty_lines(input);
            let line = match input.peek() {
                Some(line) => line,
                None => break,
            };
            if !line.starts_with(|c: char| c.is_whitespace()) {
                break;
            }
            let (data, comment) = split_comment(line.trim());
            block.push(Line { data, comment });
        }
        input.next();
    }
    Ok(Segment::Block { name, block, scale })
}

fn parse_block_header(header: &str) -> Result<(String, Option<f64>)> {
    let (data, _) = split_comment(header);
    let (name, rest) = match next_word(data) {
        None => bail!(ErrorKind::MissingBlockName),
        Some((name, rest)) => (name.to_lowercase(), rest),
    };
    let scale = parse_block_scale(rest).chain_err(|| {
        ErrorKind::InvalidBlock(name.clone())
    })?;
    Ok((name, scale))
}

fn parse_block_scale(header: &str) -> Result<Option<f64>> {
    let (word, rest) = match next_word(header) {
        None => return Ok(None),
        Some(a) => a,
    };
    println!("word: '{}', rest: '{}'", word, rest);
    let rest = match word.to_lowercase().as_ref() {
        "q=" => rest,
        "q" => {
            match next_word(rest) {
                Some(("=", rest)) => rest,
                _ => bail!(ErrorKind::MalformedBlockHeader(header.to_string())),
            }
        }
        _ => bail!(ErrorKind::MalformedBlockHeader(header.to_string())),
    };
    f64::parse(rest.trim())
        .end()
        .chain_err(|| ErrorKind::InvalidScale)
        .map(Some)
}

fn parse_decay_table<'a, Iter>(
    header: &str,
    input: &mut iter::Peekable<Iter>,
) -> Result<Segment<'a>>
where
    Iter: Iterator<Item = &'a str>,
{
    let (pdg_id, width) = parse_decay_table_header(header)?;
    let mut decays = Vec::new();
    loop {
        {
            skip_empty_lines(input);
            let line = match input.peek() {
                Some(line) => line,
                None => break,
            };
            if !line.starts_with(|c: char| c.is_whitespace()) {
                break;
            }
            let (data, _) = split_comment(line.trim());
            let n = decays.len() + 1;
            decays.push(parse_decay(data)
                .chain_err(|| ErrorKind::InvalidDecayLine(n))
                .chain_err(|| ErrorKind::InvalidDecay(pdg_id))?);
        }
        input.next();
    }
    Ok(Segment::Decay {
        pdg_id,
        width,
        decays,
    })
}

fn parse_decay_table_header(header: &str) -> Result<(i64, f64)> {
    let (data, _) = split_comment(header);
    let (rest, pdg_id) = i64::parse(data).to_result().chain_err(|| {
        ErrorKind::InvalidDecayingPdgId
    })?;
    let width = f64::parse(rest).end().chain_err(
        || ErrorKind::InvalidDecay(pdg_id),
    )?;
    Ok((pdg_id, width))
}

fn parse_decay(line: &str) -> Result<Decay> {
    let mut rest = line;
    let branching_ratio = match f64::parse(rest) {
        ParseResult::Done(r, value) => {
            rest = r;
            value
        }
        ParseResult::Error(e) => bail!(e.chain_err(|| ErrorKind::InvalidBranchingRatio)),
    };
    let n_daughters = match u8::parse(rest) {
        ParseResult::Done(r, value) => {
            rest = r;
            value
        }
        ParseResult::Error(e) => bail!(e.chain_err(|| ErrorKind::InvalidNumOfDaughters)),
    };
    let mut daughters = Vec::new();
    for i in 0..n_daughters {
        rest = rest.trim();
        if rest.is_empty() {
            bail!(ErrorKind::NotEnoughDaughters(n_daughters, i));
        }
        let daughter_id = match i64::parse(rest) {
            ParseResult::Done(r, value) => {
                rest = r;
                value
            }
            ParseResult::Error(e) => bail!(e.chain_err(|| ErrorKind::InvalidDaughterId)),
        };
        daughters.push(daughter_id);
    }
    rest.trim();
    if !rest.is_empty() {
        bail!(ErrorKind::IncompleteParse(rest.to_string()));
    }
    Ok(Decay {
        branching_ratio,
        daughters,
    })
}

fn skip_empty_lines<'a, Iter>(input: &mut iter::Peekable<Iter>)
where
    Iter: Iterator<Item = &'a str>,
{
    loop {
        let line = match input.peek() {
            Some(line) => line.trim(),
            None => break,
        };
        if line.is_empty() || line.starts_with('#') {
            input.next();
        } else {
            break;
        }
    }
}

pub fn next_word(input: &str) -> Option<(&str, &str)> {
    let input = input.trim_left();
    if input.is_empty() {
        return None;
    }
    let index = match input.find(|c: char| c.is_whitespace()) {
        Some(index) => index,
        None => return Some((input, "")),
    };
    Some(input.split_at(index))
}

fn split_comment(line: &str) -> (&str, Option<&str>) {
    let start = match line.find('#') {
        None => return (line, None),
        Some(start) => start,
    };
    let (data, comment) = line.split_at(start);
    (data, Some(comment))
}

#[cfg(test)]
mod tests {
    use super::next_word;

    #[test]
    fn test_next_word() {
        assert_eq!(next_word(""), None);
        assert_eq!(next_word("    "), None);
        assert_eq!(next_word("\t \t   "), None);
        assert_eq!(next_word("foo"), Some(("foo", "")));
        assert_eq!(next_word("   bar"), Some(("bar", "")));
        assert_eq!(next_word("foo    "), Some(("foo", "    ")));
        assert_eq!(next_word("   bar\t  "), Some(("bar", "\t  ")));
        assert_eq!(next_word("bar\t  foogh"), Some(("bar", "\t  foogh")));
        assert_eq!(next_word("   bar\t  foogh"), Some(("bar", "\t  foogh")));
        assert_eq!(next_word("\tbar"), Some(("bar", "")));
        assert_eq!(next_word(" \t  bar\t  "), Some(("bar", "\t  ")));
        assert_eq!(next_word("\t   \tbar\t  foogh"), Some(("bar", "\t  foogh")));
    }
}