use std::collections::HashMap;
use std::hash::Hash;
use std::iter;
use std::num::{ParseFloatError, ParseIntError};
use std::str;

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
    UnexpectedIdent(String),
    MissingBlockName,
    MalformedBlockHeader(String),
    DuplicateBlock(String),
    RedefinedBlockWithQ(String),
    InvalidScale(ParseFloatError),
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
        #[allow(non_snake_case)]
        #[allow(unused_assignments)]
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
    pub map: HashMap<Key, Value>,
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

#[derive(Debug)]
enum BlockScale<'a> {
    WithScale(Vec<(f64, Vec<Line<'a>>)>),
    WithoutScale(Vec<Line<'a>>),
}

enum Segment<'a> {
    Block {
        name: String,
        block: Vec<Line<'a>>,
        scale: Option<f64>,
    },
}

/// An SLHA file.
#[derive(Debug)]
pub struct Slha<'a> {
    blocks: HashMap<String, BlockScale<'a>>,
}
impl<'a> Slha<'a> {
    /// Create a new Slha object from raw data.
    pub fn parse(input: &'a str) -> Result<Slha<'a>, ParseError> {
        let mut slha = Slha { blocks: HashMap::new() };
        let mut lines = input.lines().peekable();
        while let Some(segment) = parse_segment(&mut lines) {
            match segment? {
                Segment::Block { name, block, scale } => slha.insert_block(name, block, scale)?,
            }
        }
        Ok(slha)
    }

    /// Lookup a block.
    pub fn get_block<B: SlhaBlock<E>, E>(&self, name: &str) -> Option<Result<B, E>> {
        let block = match self.blocks.get(name) {
            Some(lines) => lines,
            None => return None,
        };
        let lines = match block {
            &BlockScale::WithoutScale(ref lines) => lines,
            &BlockScale::WithScale(ref blocks) => &(blocks[0].1),
        };
        Some(B::parse(lines))
    }

    fn insert_block(
        &mut self,
        name: String,
        block: Vec<Line<'a>>,
        scale: Option<f64>,
    ) -> Result<(), ParseError> {
        if let Some(scale) = scale {
            self.insert_block_scale(name, block, scale)
        } else {
            self.insert_block_noscale(name, block)
        }
    }

    fn insert_block_noscale(
        &mut self,
        name: String,
        block: Vec<Line<'a>>,
    ) -> Result<(), ParseError> {
        if self.blocks.contains_key(&name) {
            return Err(ParseError::DuplicateBlock(name));
        }
        self.blocks.insert(name, BlockScale::WithoutScale(block));
        Ok(())
    }

    fn insert_block_scale(
        &mut self,
        name: String,
        block: Vec<Line<'a>>,
        scale: f64,
    ) -> Result<(), ParseError> {
        let entry = self.blocks.entry(name.clone()).or_insert_with(|| {
            BlockScale::WithScale(Vec::new())
        });
        match *entry {
            BlockScale::WithoutScale(_) => return Err(ParseError::RedefinedBlockWithQ(name)),
            BlockScale::WithScale(ref mut blocks) => blocks.push((scale, block)),
        };
        Ok(())
    }
}

fn parse_segment<'a>(
    input: &mut iter::Peekable<str::Lines<'a>>,
) -> Option<Result<Segment<'a>, ParseError>> {
    skip_empty_lines(input);
    let line = match input.next() {
        Some(line) => line,
        None => return None,
    };
    if line.starts_with(|c: char| c.is_whitespace()) {
        return Some(Err(ParseError::UnexpectedIdent(line.to_string())));
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
) -> Result<Segment<'a>, ParseError>
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

fn parse_block_header(header: &str) -> Result<(String, Option<f64>), ParseError> {
    let (data, _) = split_comment(header);
    let (name, rest) = match next_word(data) {
        None => return Err(ParseError::MissingBlockName),
        Some((name, rest)) => (name.to_lowercase(), rest),
    };
    let scale = parse_block_scale(rest)?;
    Ok((name, scale))
}

fn parse_block_scale(header: &str) -> Result<Option<f64>, ParseError> {
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
                _ => return Err(ParseError::MalformedBlockHeader(header.to_string())),
            }
        }
        _ => return Err(ParseError::MalformedBlockHeader(header.to_string())),
    };
    match str::parse(rest.trim()) {
        Ok(scale) => Ok(Some(scale)),
        Err(e) => Err(ParseError::InvalidScale(e)),
    }
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
# Lets put a comment here, because why not
 1 3 # Testcase number one
# How about we separate the two lines here
 # by two comment lines, one of which is indented
 4 6     # Testcase number two

# The masses of all particles
block Mass
  6  173.2    # M_top
# A trailing comment can't hurt, now can it?
";
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

    #[test]
    fn test_example_3() {
        // Pieces of the example file from appendix D.2 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "
Block Umix  # chargino U mixing matrix
  1  1     9.16207706e-01   # U_{1,1}
  1  2    -4.00703680e-01   # U_{1,2}
  2  1     4.00703680e-01   # U_{2,1}
  2  2     9.16207706e-01   # U_{2,2}
Block gauge Q= 4.64649125e+02
     1     3.60872342e-01   # g’(Q)MSSM DRbar
     2     6.46479280e-01   # g(Q)MSSM DRbar
     3     1.09623002e+00   # g3(Q)MSSM DRbar
Block yu Q= 4.64649125e+02
  3  3     8.88194465e-01   # Yt(Q)MSSM DRbar
Block yd Q= 4.64649125e+02
  3  3     1.40135884e-01   # Yb(Q)MSSM DRbar
Block ye Q= 4.64649125e+02
  3  3     9.97405356e-02   # Ytau(Q)MSSM DRbar
Block hmix Q= 4.64649125e+02  # Higgs mixing parameters
     1     3.58660361e+02   # mu(Q)MSSM DRbar
     2     9.75139550e+00   # tan beta(Q)MSSM DRbar
     3     2.44923506e+02   # higgs vev(Q)MSSM DRbar
     4     1.69697051e+04   # [m3^2/cosBsinB](Q)MSSM DRbar
";
        let slha = Slha::parse(input).unwrap();
        println!("{:?}", slha);
        let gauge: Block<i8, f64> = slha.get_block("gauge").unwrap().unwrap();
        assert_eq!(gauge.map.len(), 3);
        assert_eq!(gauge.map[&1], 3.60872342e-01);
        assert_eq!(gauge.map[&2], 6.46479280e-01);
        assert_eq!(gauge.map[&3], 1.09623002e+00);
        let umix: Block<(u8, u8), f64> = slha.get_block("umix").unwrap().unwrap();
        assert_eq!(umix.map.len(), 4);
        assert_eq!(umix.map[&(1, 1)], 9.16207706e-01);
        assert_eq!(umix.map[&(1, 2)], -4.00703680e-01);
        assert_eq!(umix.map[&(2, 1)], 4.00703680e-01);
        assert_eq!(umix.map[&(2, 2)], 9.16207706e-01);
        let yu: Block<(u8, u8), f64> = slha.get_block("yu").unwrap().unwrap();
        assert_eq!(yu.map.len(), 1);
        assert_eq!(yu.map[&(3, 3)], 8.88194465e-01);
        let yd: Block<(u8, u8), f64> = slha.get_block("yd").unwrap().unwrap();
        assert_eq!(yd.map.len(), 1);
        assert_eq!(yd.map[&(3, 3)], 1.40135884e-01);
        let ye: Block<(u8, u8), f64> = slha.get_block("ye").unwrap().unwrap();
        assert_eq!(ye.map.len(), 1);
        assert_eq!(ye.map[&(3, 3)], 9.97405356e-02);
        let hmix: Block<i8, f64> = slha.get_block("hmix").unwrap().unwrap();
        assert_eq!(hmix.map.len(), 4);
        assert_eq!(hmix.map[&1], 3.58660361e+02);
        assert_eq!(hmix.map[&2], 9.75139550e+00);
        assert_eq!(hmix.map[&3], 2.44923506e+02);
        assert_eq!(hmix.map[&4], 1.69697051e+04);
    }
}
