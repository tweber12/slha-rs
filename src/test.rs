use std::collections::HashMap;
use std::hash::Hash;
use std::iter;
use std::num::{ParseFloatError, ParseIntError};
use std::str;
use std::str::FromStr;

/// A trait for blocks that can be read from an SLHA file.
pub trait SlhaBlock<E>: Sized {
    /// Parse the block from an SLHA file.
    ///
    /// The argument of the `parse` function are all lines that belong
    /// to the block.
    fn parse<'a>(&[Line<'a>]) -> Result<Self, E>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseResult<'input, T> {
    Done(&'input str, T),
    Error(ParseError),
}
impl<'input, T> ParseResult<'input, T> {
    fn end(self) -> Result<T, ParseError> {
        match self {
            ParseResult::Error(e) => Err(e),
            ParseResult::Done(input, _) if !input.is_empty() => Err(ParseError::IncompleteParse(
                input.to_string(),
            )),
            ParseResult::Done(_, value) => Ok(value),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    IncompleteParse(String),
    UnexpectedEol,
    InvalidInt(ParseIntError),
    InvalidFloat(ParseFloatError),
    UnknownSegment(String),
    UnexpectedIdent,
    MissingBlockName,
    MalformedBlockHeader(String),
}
pub trait Parseable: Sized {
    fn parse<'input>(&'input str) -> ParseResult<'input, Self>;
}

impl Parseable for String {
    fn parse<'input>(input: &'input str) -> ParseResult<'input, String> {
        let input = input.trim();
        if input.is_empty() {
            return ParseResult::Error(ParseError::UnexpectedEol);
        }
        ParseResult::Done("", input.to_string())
    }
}

macro_rules! impl_parseable {
    ($int:ty, $err:ident) => {
        impl Parseable for $int {
            fn parse<'input>(input: &'input str) -> ParseResult<'input, $int> {
                let (word, rest) = match next_word(input) {
                    Some(a) => a,
                    None => return ParseResult::Error(ParseError::UnexpectedEol),
                };
                let value: $int = match word.parse() {
                    Ok(value) => value,
                    Err(err) => return ParseResult::Error(ParseError::$err(err)),
                };
                ParseResult::Done(rest, value)
            }
        }
    }
}
impl_parseable!(i8, InvalidInt);
impl_parseable!(i16, InvalidInt);
impl_parseable!(i32, InvalidInt);
impl_parseable!(i64, InvalidInt);
impl_parseable!(u8, InvalidInt);
impl_parseable!(u16, InvalidInt);
impl_parseable!(u32, InvalidInt);
impl_parseable!(u64, InvalidInt);
impl_parseable!(f32, InvalidFloat);
impl_parseable!(f64, InvalidFloat);

macro_rules! impl_parseable_tuple {
    ($($name:ident),+) => {
        impl<$($name),*> Parseable for ($($name),*)
        where
            $($name: Parseable),*
        {
            fn parse<'input>(input: &'input str) -> ParseResult<'input, ($($name),*)> {
                let mut input = input;
                $(
                    let (rest, $name) = match $name::parse(input.trim_left()) {
                        ParseResult::Done(rest, value) => (rest, value),
                        ParseResult::Error(err) => return ParseResult::Error(err),
                    };
                    input = rest;
                )*
                ParseResult::Done(rest, ($($name),*))
            }
        }
    }
}
impl_parseable_tuple!(K1, K2);
impl_parseable_tuple!(K1, K2, K3);
impl_parseable_tuple!(K1, K2, K3, K4);
impl_parseable_tuple!(K1, K2, K3, K4, K5);
impl_parseable_tuple!(K1, K2, K3, K4, K5, K6);
impl_parseable_tuple!(K1, K2, K3, K4, K5, K6, K7);
impl_parseable_tuple!(K1, K2, K3, K4, K5, K6, K7, K8);
impl_parseable_tuple!(K1, K2, K3, K4, K5, K6, K7, K8, K9);
impl_parseable_tuple!(K1, K2, K3, K4, K5, K6, K7, K8, K9, K10);
impl_parseable_tuple!(K1, K2, K3, K4, K5, K6, K7, K8, K9, K10, K11);
impl_parseable_tuple!(K1, K2, K3, K4, K5, K6, K7, K8, K9, K10, K11, K12);

fn next_word(input: &str) -> Option<(&str, &str)> {
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

pub struct Block<Key, Value> {
    map: HashMap<Key, Value>,
}
impl<Key, Value> SlhaBlock<ParseError> for Block<Key, Value>
where
    Key: Hash + Eq + Parseable,
    Value: Parseable,
{
    fn parse<'input>(lines: &[Line<'input>]) -> Result<Self, ParseError> {
        let map: Result<HashMap<Key, Value>, ParseError> = lines
            .iter()
            .map(|line| parse_line_block(line.data).end())
            .collect();
        Ok(Block { map: map? })
    }
}

fn parse_line_block<'input, K, V>(input: &'input str) -> ParseResult<'input, (K, V)>
where
    K: Parseable,
    V: Parseable,
{
    let input = input.trim_left();
    let (input, key) = match K::parse(input) {
        ParseResult::Done(input, key) => (input.trim_left(), key),
        ParseResult::Error(e) => return ParseResult::Error(e),
    };
    let (input, value) = match V::parse(input) {
        ParseResult::Done(input, key) => (input.trim_left(), key),
        ParseResult::Error(e) => return ParseResult::Error(e),
    };
    ParseResult::Done(input, (key, value))
}

/// A line read from an SLHA file.
#[derive(Debug)]
pub struct Line<'input> {
    /// The data contained in the line.
    data: &'input str,
    /// The comment at the end of the line, if present.
    comment: Option<&'input str>,
}

/// An SLHA file.
#[derive(Debug)]
pub struct Slha<'a> {
    blocks: HashMap<String, Vec<Line<'a>>>,
}
impl<'a> Slha<'a> {
    /// Create a new Slha object from raw data.
    fn parse(input: &'a str) -> Result<Slha<'a>, ParseError> {
        let mut blocks = HashMap::new();
        let mut lines = input.lines().peekable();
        loop {
            let (name, block) = match parse_segment(&mut lines) {
                Some(s) => s?,
                None => break,
            };
            blocks.insert(name, block);
        }
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

fn parse_segment<'a>(
    input: &mut iter::Peekable<str::Lines<'a>>,
) -> Option<Result<(String, Vec<Line<'a>>), ParseError>> {
    skip_empty_lines(input);
    let line = match input.next() {
        Some(line) => line,
        None => return None,
    };
    if line.starts_with(|c: char| c.is_whitespace()) {
        return Some(Err(ParseError::UnexpectedIdent));
    }
    match next_word(line) {
        Some((kw, rest)) => Some(match kw.to_lowercase().as_ref() {
            "block" => parse_block(rest, input),
            kw => Err(ParseError::UnknownSegment(kw.to_string())),
        }),
        None => unreachable!("All empty lines have been skipped, so this line MUST NOT be empty."),
    }
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

fn parse_block<'a, Iter>(
    header: &str,
    input: &mut iter::Peekable<Iter>,
) -> Result<(String, Vec<Line<'a>>), ParseError>
where
    Iter: Iterator<Item = &'a str>,
{
    let (data, _) = split_comment(header);
    let name = match next_word(data) {
        None => return Err(ParseError::MissingBlockName),
        Some((name, rest)) if !rest.trim().is_empty() => {
            return Err(ParseError::MalformedBlockHeader(rest.to_string()));
        }
        Some((name, _)) => name.to_lowercase(),
    };
    let mut lines = Vec::new();
    loop {
        {
            let line = match input.peek() {
                Some(line) => line,
                None => break,
            };
            if !line.starts_with(|c: char| c.is_whitespace()) {
                break;
            }
            let (data, comment) = split_comment(line.trim());
            lines.push(Line { data, comment });
        }
        input.next();
    }
    Ok((name, lines))
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
    use super::{Slha, Block, Parseable, ParseResult};
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

    #[test]
    fn test_parse_tuple() {
        type T2 = (u8, u8);
        assert_eq!(T2::parse("1 2"), ParseResult::Done("", (1, 2)));
        assert_eq!(T2::parse("    1 2"), ParseResult::Done("", (1, 2)));
        assert_eq!(T2::parse("1 2   456"), ParseResult::Done("   456", (1, 2)));
        assert_eq!(
            T2::parse(" 1    2      foobar"),
            ParseResult::Done("      foobar", (1, 2))
        );
    }

    #[test]
    fn test_parse_block() {
        let input = "\
BLOCK TEST
 1 3
 4 6
block Mass
  6  173.2";
        let slha = Slha::parse(input).unwrap();
        println!("{:?}", slha);
        let block: Block<i64, i64> = slha.get_block("test").unwrap().unwrap();
        assert_eq!(block.map.len(), 2);
        assert_eq!(block.map[&1], 3);
        assert_eq!(block.map[&4], 6);
        let block: Block<i64, f64> = slha.get_block("mass").unwrap().unwrap();
        assert_eq!(block.map.len(), 1);
        assert_eq!(block.map[&6], 173.2);
    }

    #[test]
    fn test_parse_block_comment() {
        let input = "\
# This block contains information
# about testing.
BLOCK TEST # This is the block header
 1 3 # Testcase number one
 4 6     # Testcase number two

# The masses of all particles
block Mass
  6  173.2    # M_top";
        let slha = Slha::parse(input).unwrap();
        println!("{:?}", slha);
        let block: Block<i64, i64> = slha.get_block("test").unwrap().unwrap();
        assert_eq!(block.map.len(), 2);
        assert_eq!(block.map[&1], 3);
        assert_eq!(block.map[&4], 6);
        let block: Block<i64, f64> = slha.get_block("mass").unwrap().unwrap();
        assert_eq!(block.map.len(), 1);
        assert_eq!(block.map[&6], 173.2);
    }

    #[test]
    fn test_example_1() {
        // Example file from appendix D.1 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
# SUSY Les Houches Accord 1.0 - example input file
# Snowmsas point 1a
Block MODSEL  # Select model
     1    1   # sugra
Block SMINPUTS   # Standard Model inputs
     3      0.1172  # alpha_s(MZ) SM MSbar
     5      4.25    # Mb(mb) SM MSbar
     6    174.3     # Mtop(pole)
Block MINPAR  # SUSY breaking input parameters
     3     10.0     # tanb
     4      1.0     # sign(mu)
     1    100.0     # m0
     2    250.0     # m12
     5   -100.0     # A0 ";
        let slha = Slha::parse(input).unwrap();
        println!("{:?}", slha);
        let sminputs: Block<i8, f64> = slha.get_block("sminputs").unwrap().unwrap();
        assert_eq!(sminputs.map.len(), 3);
        assert_eq!(sminputs.map[&3], 0.1172);
        assert_eq!(sminputs.map[&5], 4.25);
        assert_eq!(sminputs.map[&6], 174.3);
        let modsel: Block<i8, i8> = slha.get_block("modsel").unwrap().unwrap();
        assert_eq!(modsel.map.len(), 1);
        assert_eq!(modsel.map[&1], 1);
        let minpar: Block<i8, f64> = slha.get_block("minpar").unwrap().unwrap();
        assert_eq!(minpar.map.len(), 5);
        assert_eq!(minpar.map[&3], 10.0);
        assert_eq!(minpar.map[&4], 1.0);
        assert_eq!(minpar.map[&1], 100.0);
        assert_eq!(minpar.map[&2], 250.0);
        assert_eq!(minpar.map[&5], -100.0);
    }

    #[test]
    fn test_almost_example_1() {
        // Example file from appendix D.1 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
# SUSY Les Houches Accord 1.0 - example input file
# Snowmsas point 1a
Block MODSEL  # Select model
     1    1   $ sugra
Block SMINPUTS   # Standard Model inputs
     3      0.1172  $ alpha_s(MZ) SM MSbar
     5      4.25    $ Mb(mb) SM MSbar
     6    174.3     $ Mtop(pole)
Block MINPAR  # SUSY breaking input parameters
     3     10.0     $ tanb
     4      1.0     $ sign(mu)
     1    100.0     $ m0
     2    250.0     $ m12
     5   -100.0     $ A0 ";
        let slha = Slha::parse(input).unwrap();
        println!("{:?}", slha);
        let sminputs: Block<i8, String> = slha.get_block("sminputs").unwrap().unwrap();
        assert_eq!(sminputs.map.len(), 3);
        assert_eq!(sminputs.map[&3], "0.1172  $ alpha_s(MZ) SM MSbar");
        assert_eq!(sminputs.map[&5], "4.25    $ Mb(mb) SM MSbar");
        assert_eq!(sminputs.map[&6], "174.3     $ Mtop(pole)");
        let modsel: Block<i8, String> = slha.get_block("modsel").unwrap().unwrap();
        assert_eq!(modsel.map.len(), 1);
        assert_eq!(modsel.map[&1], "1   $ sugra");
        let minpar: Block<i8, String> = slha.get_block("minpar").unwrap().unwrap();
        assert_eq!(minpar.map.len(), 5);
        assert_eq!(minpar.map[&3], "10.0     $ tanb");
        assert_eq!(minpar.map[&4], "1.0     $ sign(mu)");
        assert_eq!(minpar.map[&1], "100.0     $ m0");
        assert_eq!(minpar.map[&2], "250.0     $ m12");
        assert_eq!(minpar.map[&5], "-100.0     $ A0");
    }

    #[test]
    fn test_example_2() {
        // Pieces of the example file from appendix D.2 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "
Block stopmix  # stop mixing matrix
  1  1     5.37975095e-01   # O_{11}
  1  2     8.42960733e-01   # O_{12}
  2  1     8.42960733e-01   # O_{21}
  2  2    -5.37975095e-01   # O_{22}
Block sbotmix  # sbottom mixing matrix
  1  1     9.47346882e-01   # O_{11}
  1  2     3.20209128e-01   # O_{12}
  2  1    -3.20209128e-01   # O_{21}
  2  2     9.47346882e-01   # O_{22}
Block staumix  # stau mixing matrix
  1  1     2.78399839e-01   # O_{11}
  1  2     9.60465267e-01   # O_{12}
  2  1     9.60465267e-01   # O_{21}
  2  2    -2.78399839e-01   # O_{22}
";
        let slha = Slha::parse(input).unwrap();
        println!("{:?}", slha);
        let stopmix: Block<(u8, u8), f64> = slha.get_block("stopmix").unwrap().unwrap();
        assert_eq!(stopmix.map.len(), 4);
        assert_eq!(stopmix.map[&(1, 1)], 5.37975095e-01);
        assert_eq!(stopmix.map[&(1, 2)], 8.42960733e-01);
        assert_eq!(stopmix.map[&(2, 1)], 8.42960733e-01);
        assert_eq!(stopmix.map[&(2, 2)], -5.37975095e-01);
        let sbotmix: Block<(u8, u8), f64> = slha.get_block("sbotmix").unwrap().unwrap();
        assert_eq!(sbotmix.map.len(), 4);
        assert_eq!(sbotmix.map[&(1, 1)], 9.47346882e-01);
        assert_eq!(sbotmix.map[&(1, 2)], 3.20209128e-01);
        assert_eq!(sbotmix.map[&(2, 1)], -3.20209128e-01);
        assert_eq!(sbotmix.map[&(2, 2)], 9.47346882e-01);
        let staumix: Block<(u8, u8), f64> = slha.get_block("staumix").unwrap().unwrap();
        assert_eq!(staumix.map.len(), 4);
        assert_eq!(staumix.map[&(1, 1)], 2.78399839e-01);
        assert_eq!(staumix.map[&(1, 2)], 9.60465267e-01);
        assert_eq!(staumix.map[&(2, 1)], 9.60465267e-01);
        assert_eq!(staumix.map[&(2, 2)], -2.78399839e-01);
    }

    #[test]
    fn test_almost_example_2() {
        // Pieces of the example file from appendix D.2 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "
Block stopmix  # stop mixing matrix
  1  1     5.37975095e-01   # O_{11}
  2  2     8.42960733e-01   # O_{12}
  3  1     8.42960733e-01   # O_{21}
  4  2    -5.37975095e-01   # O_{22}
Block sbotmix  # sbottom mixing matrix
  1  1     9.47346882e-01   # O_{11}
  2  2     3.20209128e-01   # O_{12}
  3  1    -3.20209128e-01   # O_{21}
  4  2     9.47346882e-01   # O_{22}
Block staumix  # stau mixing matrix
  1  1     2.78399839e-01   # O_{11}
  2  2     9.60465267e-01   # O_{12}
  3  1     9.60465267e-01   # O_{21}
  4  2    -2.78399839e-01   # O_{22}
";
        let slha = Slha::parse(input).unwrap();
        println!("{:?}", slha);
        let stopmix: Block<u8, (u8, f64)> = slha.get_block("stopmix").unwrap().unwrap();
        assert_eq!(stopmix.map.len(), 4);
        assert_eq!(stopmix.map[&1], (1, 5.37975095e-01));
        assert_eq!(stopmix.map[&2], (2, 8.42960733e-01));
        assert_eq!(stopmix.map[&3], (1, 8.42960733e-01));
        assert_eq!(stopmix.map[&4], (2, -5.37975095e-01));
        let sbotmix: Block<u8, (u8, f64)> = slha.get_block("sbotmix").unwrap().unwrap();
        assert_eq!(sbotmix.map.len(), 4);
        assert_eq!(sbotmix.map[&1], (1, 9.47346882e-01));
        assert_eq!(sbotmix.map[&2], (2, 3.20209128e-01));
        assert_eq!(sbotmix.map[&3], (1, -3.20209128e-01));
        assert_eq!(sbotmix.map[&4], (2, 9.47346882e-01));
        let staumix: Block<u8, (u8, f64)> = slha.get_block("staumix").unwrap().unwrap();
        assert_eq!(staumix.map.len(), 4);
        assert_eq!(staumix.map[&1], (1, 2.78399839e-01));
        assert_eq!(staumix.map[&2], (2, 9.60465267e-01));
        assert_eq!(staumix.map[&3], (1, 9.60465267e-01));
        assert_eq!(staumix.map[&4], (2, -2.78399839e-01));
    }
}
