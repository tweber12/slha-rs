use std::collections::HashMap;
use std::hash::Hash;
use std::str;
use nom;
use {end_of_string, skip_whitespace, skip_lines, parse_opt_comment, eol_or_eof, parse_i64};

/// A trait for blocks that can be read from an SLHA file.
pub trait SlhaBlock<E>: Sized {
    /// Parse the block from an SLHA file.
    ///
    /// The argument of the `parse` function are all lines that belong
    /// to the block.
    fn parse<'a>(&[Line<'a>]) -> Result<Self, E>;
}

pub trait Parseable: Sized {
    fn parse<'a>(&'a [u8]) -> nom::IResult<&'a [u8], Self>;
}
impl Parseable for i64 {
    fn parse(input: &[u8]) -> nom::IResult<&[u8], i64> {
        parse_i64(input)
    }
}
impl Parseable for f64 {
    fn parse(input: &[u8]) -> nom::IResult<&[u8], f64> {
        nom::double(input)
    }
}
impl<K1, K2> Parseable for (K1, K2)
where
    K1: Parseable,
    K2: Parseable,
{
    fn parse(input: &[u8]) -> nom::IResult<&[u8], (K1, K2)> {
        let parse_k1 = K1::parse;
        let parse_k2 = K2::parse;
        do_parse!(
            input,
            k1: parse_k1 >> skip_whitespace >> k2: parse_k2 >> ((k1, k2))
        )
    }
}

pub struct Block<Key, Value> {
    map: HashMap<Key, Value>,
}
impl<Key, Value> SlhaBlock<nom::IError> for Block<Key, Value>
where
    Key: Hash + Eq + Parseable,
    Value: Parseable,
{
    fn parse<'input>(lines: &[Line<'input>]) -> Result<Self, nom::IError> {
        let map: Result<HashMap<Key, Value>, nom::IError> = lines
            .iter()
            .map(|line| parse_line_block(line.data).to_full_result())
            .collect();
        Ok(Block { map: map? })
    }
}

fn parse_line_block<'input, K, V>(input: &[u8]) -> nom::IResult<&[u8], (K, V)>
where
    K: Parseable,
    V: Parseable,
{
    let parse_key = K::parse;
    let parse_value = V::parse;
    do_parse!(
        input,
        key: parse_key >> skip_whitespace >> value: parse_value >> ((key, value))
    )
}

/// A line read from an SLHA file.
#[derive(Debug)]
pub struct Line<'input> {
    /// The data contained in the line.
    data: &'input [u8],
    /// The comment at the end of the line, if present.
    comment: Option<&'input [u8]>,
}

/// An SLHA file.
#[derive(Debug)]
pub struct Slha<'a> {
    blocks: HashMap<String, Vec<Line<'a>>>,
}
impl<'a> Slha<'a> {
    /// Create a new Slha object from raw data.
    fn parse(input: &'a [u8]) -> Result<Slha<'a>, nom::IError> {
        let blocks = many0!(input, parse_block).to_full_result()?;
        let blocks = blocks.into_iter().collect();
        Ok(Slha { blocks })
    }

    /// Lookup a block.
    fn get_block<B: SlhaBlock<E>, E>(&self, name: &str) -> Option<Result<B, E>> {
        let lines = match self.blocks.get(name) {
            Some(lines) => lines,
            None => return None,
        };
        Some(B::parse(lines))
    }
}

named!(parse_block<(String, Vec<Line>)>,
    do_parse!(
        name: parse_block_header >>
        lines: parse_block_lines >>
        ((name, lines))
    )
);

named!(parse_block_header<String>,
    map_res!(
        do_parse!(
            ws!(tag_no_case!("block")) >>
            name: take_till1!(end_of_word) >>
            skip_whitespace >>
            parse_opt_comment >>
            eol_or_eof >>
            (name)
        ),
        |n: &[u8]| str::from_utf8(n).map(|n| n.trim().to_lowercase())
    )
);

named!(parse_block_lines<Vec<Line>>,
    do_parse!(
        skip_lines >>
        lines: many0!(preceded!(skip_lines, ws!(parse_line))) >>
        (lines)
    )
);

named!(parse_line<Line>,
    do_parse!(
        not!(parse_block_header) >>
        data: take_till1!(end_of_string) >>
        skip_whitespace >>
        comment: parse_opt_comment >>
        skip_whitespace >>
        eol_or_eof >>
        (Line { data, comment })
    )
);

fn end_of_word(c: u8) -> bool {
    nom::is_space(c) || c == b'#' || c == b'\n' || c == b'\r'
}

#[cfg(test)]
mod tests {
    use super::{Slha, Block};

    #[test]
    fn test_parse_block() {
        let input = b"\
BLOCK TEST
 1 3
 4 6 ";
        let slha = Slha::parse(input).unwrap();
        println!("{:?}", slha);
        let block: Block<i64, i64> = slha.get_block("test").unwrap().unwrap();
        assert_eq!(block.map.len(), 2);
        assert_eq!(block.map[&1], 3);
        assert_eq!(block.map[&4], 6);
    }
}
