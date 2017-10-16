// Copyright 2017 Torsten Weber
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![recursion_limit="128"]

#[macro_use]
extern crate error_chain;

use std::result;
use std::collections::HashMap;
use std::hash::Hash;
use std::str;

pub mod internal;
use internal::{Segment, next_word};

pub mod errors {
    use std::num::{ParseFloatError, ParseIntError};

    error_chain!{
        errors {
            MissingBlockName {
                description("Missing block name")
            }
            InvalidBlock(name: String) {
                description("Malformed block")
                display("Malformed block: '{}'", name)
            }
            InvalidBlockSingle(name: String) {
                description("Malformed block single")
                display("Malformed block single: '{}'", name)
            }
            InvalidDecayingPdgId {
                description("Failed to parse the pdg id of the decaying particle")
            }
            InvalidDecay(pdg_id: i64) {
                description("Invalid decay table")
                display("Invalid decay table for particle {}", pdg_id)
            }
            IncompleteParse(rest: String) {
                description("The parser did not consume the whole line")
                display("The parser did not consume the whole line, '{}' was left over", rest)
            }
            UnexpectedEol {
                description("The parser reached the end of the line before finishing")
            }
            InvalidInt(err: ParseIntError) {
                description("Failed to parse an integer")
                display("Failed to parse an integer: {}", err)
            }
            InvalidFloat(err: ParseFloatError) {
                description("Failed to parse a floating point number")
                display("Failed to parse a floating point number: {}", err)
            }
            UnknownSegment(segment: String) {
                description("Unknown top level segment encountered")
                display("Unknown top level segment encountered: '{}'", segment)
            }
            UnexpectedIdent(line: String) {
                description("Expected the beginning of a segment, found an indented line instead")
                display("Expected the beginning of a segment, found an indented line instead: '{}'", line)
            }
            MalformedBlockHeader(rest: String) {
                description("Encountered trailing non-whitespace characters after block header")
                display("Encountered trailing non-whitespace characters after block header: '{}'", rest)
            }
            InvalidBlockLine(n: usize) {
                description("Failed to parse a line in the body")
                display("Failed to parse the {}th data line in the body", n)
            }
            InvalidBlockKey {
                description("Failed to parse the key of a block")
            }
            InvalidBlockValue {
                description("Failed to parse the value of a block")
            }
            DuplicateBlock(name: String) {
                description("Found a duplicate block")
                display("Found a duplicate block: '{}'", name)
            }
            DuplicateBlockScale(name: String, scale: f64) {
                description("Found a duplicate block with equal scale")
                display("Found a duplicate block with name '{}' and scale '{}'", name, scale)
            }
            RedefinedBlockWithQ(name: String) {
                description("Found a duplicate block with and without scale")
                display("Found a duplicate block with and without scale: '{}'", name)
            }
            InvalidScale {
                description("Failed to parse the scale")
            }
            DuplicateDecay(pdg_id: i64) {
                description("Found multiple decay tables for the same particle")
                display("Found multiple decay tables for the same particle: '{}'", pdg_id)
            }
            InvalidDecayLine(n: usize) {
                description("Failed to parse a line in the body")
                display("Failed to parse the {}th data line in the body", n)
            }
            InvalidWidth {
                description("Failed to parse the width")
            }
            InvalidBranchingRatio {
                description("Failed to parse the branching ratio")
            }
            InvalidNumOfDaughters {
                description("Failed to parse the number of daughter particles")
            }
            NotEnoughDaughters(expected: u8, found: u8) {
                description("Did not find enough daughter particles")
                display("Did not find enough daughter particles, expected {} but found {}", expected, found)
            }
            InvalidDaughterId {
                description("Failed to parse the pdg id of a daughter particle")
            }
            WrongNumberOfValues(n: usize) {
                description("Found too many values in a single valued block")
                display("Found {} values in a single valued block", n)
            }
            MissingBlock(name: String) {
                description("A block is missing")
                display("Did not find the block with name '{}'", name)
            }
        }
    }
}

use errors::*;

pub trait SlhaDeserialize: Sized {
    fn deserialize(&str) -> Result<Self>;
}

/// A trait for blocks that can be read from an SLHA file.
pub trait SlhaBlock<E>: Sized {
    /// Parse the block from an SLHA file.
    ///
    /// The argument of the `parse` function are all lines that belong
    /// to the block.
    fn parse<'a>(&[Line<'a>], scale: Option<f64>) -> result::Result<Self, E>;
    fn scale(&self) -> Option<f64>;
}

#[derive(Debug)]
pub enum ParseResult<'input, T> {
    Done(&'input str, T),
    Error(Error),
}
impl<'input, T> ParseResult<'input, T> {
    fn end(self) -> Result<T> {
        match self {
            ParseResult::Error(e) => Err(e),
            ParseResult::Done(input, _) if !input.trim().is_empty() => Err(
                ErrorKind::IncompleteParse(
                    input.to_string(),
                ).into(),
            ),
            ParseResult::Done(_, value) => Ok(value),
        }
    }
    fn to_result(self) -> Result<(&'input str, T)> {
        match self {
            ParseResult::Done(rest, value) => Ok((rest, value)),
            ParseResult::Error(err) => Err(err),
        }
    }
}

pub trait Parseable: Sized {
    fn parse<'input>(&'input str) -> ParseResult<'input, Self>;
}

impl Parseable for String {
    fn parse<'input>(input: &'input str) -> ParseResult<'input, String> {
        let input = input.trim();
        if input.is_empty() {
            return ParseResult::Error(ErrorKind::UnexpectedEol.into());
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
                    None => return ParseResult::Error(ErrorKind::UnexpectedEol.into()),
                };
                let value: $int = match word.parse() {
                    Ok(value) => value,
                    Err(err) => return ParseResult::Error(ErrorKind::$err(err).into()),
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

#[derive(Clone, Debug, PartialEq)]
pub struct Block<Key, Value>
where
    Key: Hash + Eq,
{
    pub scale: Option<f64>,
    pub map: HashMap<Key, Value>,
}
impl<Key, Value> SlhaBlock<Error> for Block<Key, Value>
where
    Key: Hash + Eq + Parseable,
    Value: Parseable,
{
    fn parse<'input>(lines: &[Line<'input>], scale: Option<f64>) -> Result<Self> {
        let map: Result<HashMap<Key, Value>> = lines
            .iter()
            .enumerate()
            .map(|(n, line)| {
                parse_line_block(line.data).chain_err(|| ErrorKind::InvalidBlockLine(n + 1))
            })
            .collect();
        Ok(Block { map: map?, scale })
    }
    fn scale(&self) -> Option<f64> {
        self.scale
    }
}

fn parse_line_block<'input, K, V>(input: &'input str) -> Result<(K, V)>
where
    K: Parseable,
    V: Parseable,
{
    let input = input.trim_left();
    let (input, key) = K::parse(input).to_result().chain_err(
        || ErrorKind::InvalidBlockKey,
    )?;
    let value = V::parse(input).end().chain_err(
        || ErrorKind::InvalidBlockValue,
    )?;
    Ok((key, value))
}

#[derive(Clone, Debug, PartialEq)]
pub struct BlockStr<Value> {
    scale: Option<f64>,
    map: HashMap<Vec<String>, Value>,
}
impl<Value> SlhaBlock<Error> for BlockStr<Value>
where
    Value: Parseable,
{
    fn parse<'input>(lines: &[Line<'input>], scale: Option<f64>) -> Result<Self> {
        let map: Result<HashMap<Vec<String>, Value>> = lines
            .iter()
            .map(|line| parse_line_block_str(line.data))
            .collect();
        Ok(BlockStr { scale, map: map? })
    }
    fn scale(&self) -> Option<f64> {
        self.scale
    }
}

fn parse_line_block_str<'input, Value>(line: &'input str) -> Result<(Vec<String>, Value)>
where
    Value: Parseable,
{
    let mut val = Value::parse(line).end();
    let mut keys = Vec::new();
    let mut rest = line;
    while let Err(_) = val {
        if let Some((key, line)) = next_word(rest) {
            keys.push(key.to_string());
            val = Value::parse(line).end();
            rest = line;
        } else {
            return Err(ErrorKind::InvalidBlockValue.into());
        }
    }
    Ok((keys, val.expect("BUG: This should be impossible.")))
}

#[derive(Clone, Debug, PartialEq)]
pub struct BlockSingle<Value> {
    pub value: Value,
    pub scale: Option<f64>,
}
impl<Value> SlhaBlock<Error> for BlockSingle<Value>
where
    Value: Parseable,
{
    fn parse<'input>(lines: &[Line<'input>], scale: Option<f64>) -> Result<Self> {
        if lines.len() != 1 {
            bail!(ErrorKind::WrongNumberOfValues(lines.len()));
        }
        let value = Value::parse(lines[0].data).end()?;
        Ok(BlockSingle { value, scale })
    }
    fn scale(&self) -> Option<f64> {
        self.scale
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DecayTable {
    pub width: f64,
    pub decays: Vec<Decay>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Decay {
    pub branching_ratio: f64,
    pub daughters: Vec<i64>,
}

/// A line read from an SLHA file.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Line<'input> {
    /// The data contained in the line.
    pub data: &'input str,
    /// The comment at the end of the line, if present.
    pub comment: Option<&'input str>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RawBlock<'a> {
    pub scale: Option<f64>,
    pub lines: Vec<Line<'a>>,
}
impl<'a> RawBlock<'a> {
    pub fn to_block<B>(&self, name: &str) -> Result<B>
    where
        B: SlhaBlock<Error>,
    {
        B::parse(&self.lines, self.scale).chain_err(|| ErrorKind::InvalidBlock(name.to_string()))
    }
}

/// An SLHA file.
#[derive(Clone, Debug, PartialEq)]
pub struct Slha<'a> {
    blocks: HashMap<String, Vec<RawBlock<'a>>>,
    decays: HashMap<i64, DecayTable>,
}
impl<'a> Slha<'a> {
    /// Create a new Slha object from raw data.
    pub fn parse(input: &'a str) -> Result<Slha<'a>> {
        let mut slha = Slha {
            blocks: HashMap::new(),
            decays: HashMap::new(),
        };
        let mut lines = input.lines().peekable();
        while let Some(segment) = internal::parse_segment(&mut lines) {
            match segment? {
                Segment::Block { name, block, scale } => {
                    let mut blocks = slha.blocks.entry(name).or_insert_with(|| Vec::new());
                    blocks.push(RawBlock {
                        lines: block,
                        scale,
                    })
                }
                Segment::Decay {
                    pdg_id,
                    width,
                    decays,
                } => slha.insert_decay(pdg_id, width, decays)?,
            }
        }
        Ok(slha)
    }

    /// Lookup a block.
    pub fn get_block<B: SlhaBlock<Error>>(&self, name: &str) -> Option<Result<B>> {
        let name = name.to_lowercase();
        let blocks = match self.blocks.get(&name) {
            Some(blocks) => blocks,
            None => return None,
        };
        if blocks.len() > 1 {
            return Some(Err(ErrorKind::DuplicateBlock(name).into()));
        }
        Some(blocks[0].to_block(&name))
    }

    /// Lookup a block.
    pub fn get_blocks<B: SlhaBlock<Error>>(&self, name: &str) -> Result<Vec<B>> {
        let blocks: Vec<B> = self.get_blocks_unchecked(name)?;
        let mut no_scale = false;
        let mut seen_scales = Vec::new();
        for block in &blocks {
            match block.scale() {
                Some(scale) => seen_scales.push(scale),
                None => no_scale = true,
            }
        }
        if no_scale && !seen_scales.is_empty() {
            bail!(ErrorKind::RedefinedBlockWithQ(name.to_lowercase()));
        }
        if let Some(scale) = find_duplicates(seen_scales) {
            bail!(ErrorKind::DuplicateBlockScale(name.to_lowercase(), scale));
        }
        Ok(blocks)
    }

    /// Lookup a block.
    pub fn get_blocks_unchecked<B: SlhaBlock<Error>>(&self, name: &str) -> Result<Vec<B>> {
        let name = name.to_lowercase();
        let blocks = match self.blocks.get(&name) {
            Some(blocks) => blocks,
            None => return Ok(Vec::new()),
        };
        blocks.iter().map(|block| block.to_block(&name)).collect()
    }

    pub fn get_raw_blocks<'s>(&'s self, name: &str) -> &'s [RawBlock<'a>] {
        let name = name.to_lowercase();
        match self.blocks.get(&name) {
            Some(blocks) => &blocks,
            None => &[],
        }
    }

    pub fn get_decay(&self, pdg_id: i64) -> Option<&DecayTable> {
        self.decays.get(&pdg_id)
    }

    fn insert_decay(&mut self, pdg_id: i64, width: f64, decays: Vec<Decay>) -> Result<()> {
        if self.decays.contains_key(&pdg_id) {
            bail!(ErrorKind::DuplicateDecay(pdg_id));
        }
        self.decays.insert(pdg_id, DecayTable { width, decays });
        Ok(())
    }
}

fn find_duplicates<T: Clone + PartialOrd>(mut list: Vec<T>) -> Option<T> {
    if list.len() < 2 {
        return None;
    }
    list.sort_unstable_by(|e1, e2| e1.partial_cmp(e2).unwrap());
    for (e1, e2) in list.iter().zip(list.iter().skip(1)) {
        if e1 == e2 {
            return Some(e1.clone());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{Slha, Block, BlockSingle, BlockStr, Parseable, ParseResult, Decay, Line};
    use super::errors::{Error, ErrorKind};

    #[test]
    fn test_parse_tuple() {
        macro_rules! unwrap_parseresult {
            ($result:expr) => {
                match $result {
                    ParseResult::Done(rest, value) => (rest, value),
                    ParseResult::Error(err) => panic!(err),
                }
            }
        }
        type T2 = (u8, u8);
        assert_eq!(unwrap_parseresult!(T2::parse("1 2")), ("", (1, 2)));
        assert_eq!(unwrap_parseresult!(T2::parse("    1 2")), ("", (1, 2)));
        assert_eq!(unwrap_parseresult!(T2::parse("1 2   456")), (
            "   456",
            (1, 2),
        ));
        assert_eq!(unwrap_parseresult!(T2::parse(" 1    2      foobar")), (
            "      foobar",
            (1, 2),
        ));
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
    fn test_parse_raw_blocks() {
        let input = "\
BLOCK TEST
 1 3
 4 6
block Mass
  6  173.2
block Mass
  5  0.
  ";
        let slha = Slha::parse(input).unwrap();
        println!("{:?}", slha);
        let blocks = slha.get_raw_blocks("foo");
        assert_eq!(blocks.len(), 0);
        let blocks = slha.get_raw_blocks("test");
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].lines.len(), 2);
        assert_eq!(
            blocks[0].lines[0],
            Line {
                data: "1 3",
                comment: None,
            }
        );
        assert_eq!(
            blocks[0].lines[1],
            Line {
                data: "4 6",
                comment: None,
            }
        );
        let blocks = slha.get_raw_blocks("mass");
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].lines.len(), 1);
        assert_eq!(
            blocks[0].lines[0],
            Line {
                data: "6  173.2",
                comment: None,
            }
        );
        assert_eq!(blocks[1].lines.len(), 1);
        assert_eq!(
            blocks[1].lines[0],
            Line {
                data: "5  0.",
                comment: None,
            }
        );
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
    fn test_parse_raw_blocks_comment() {
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
block Mass # Why not split the masses?
  5  0.   #     Mass of the b-quark
# A trailing comment can't hurt, now can it?
";
        let slha = Slha::parse(input).unwrap();
        println!("{:?}", slha);
        let blocks = slha.get_raw_blocks("test");
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].lines.len(), 2);
        assert_eq!(
            blocks[0].lines[0],
            Line {
                data: "1 3 ",
                comment: Some("# Testcase number one"),
            }
        );
        assert_eq!(
            blocks[0].lines[1],
            Line {
                data: "4 6     ",
                comment: Some("# Testcase number two"),
            }
        );
        let blocks = slha.get_raw_blocks("mass");
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].lines.len(), 1);
        assert_eq!(
            blocks[0].lines[0],
            Line {
                data: "6  173.2    ",
                comment: Some("# M_top"),
            }
        );
        assert_eq!(blocks[1].lines.len(), 1);
        assert_eq!(
            blocks[1].lines[0],
            Line {
                data: "5  0.   ",
                comment: Some("#     Mass of the b-quark"),
            }
        );
    }

    #[test]
    fn test_get_block_case() {
        let input = "\
BLOCK TEST
 1 3
 4 6
";
        let slha = Slha::parse(input).unwrap();
        println!("{:?}", slha);
        let _: Block<i64, i64> = slha.get_block("TEST").unwrap().unwrap();
        let _: Block<i64, i64> = slha.get_block("tEsT").unwrap().unwrap();
        let _: Block<i64, i64> = slha.get_block("TesT").unwrap().unwrap();
        let block: Block<i64, i64> = slha.get_block("test").unwrap().unwrap();
        assert_eq!(block.map.len(), 2);
        assert_eq!(block.map[&1], 3);
        assert_eq!(block.map[&4], 6);
    }

    #[test]
    fn test_parse_decay_table() {
        let input = "\
DECAY   6    1.3
    0.5    2    3   4
    0.25    3    4   5  6
    0.25    4    5   6  7  8
";
        let slha = Slha::parse(input).unwrap();
        let dec = slha.get_decay(6).unwrap();
        assert_eq!(dec.width, 1.3);
        assert_eq!(dec.decays.len(), 3);
        assert_eq!(
            dec.decays[0],
            Decay {
                branching_ratio: 0.5,
                daughters: vec![3, 4],
            }
        );
        assert_eq!(
            dec.decays[1],
            Decay {
                branching_ratio: 0.25,
                daughters: vec![4, 5, 6],
            }
        );
        assert_eq!(
            dec.decays[2],
            Decay {
                branching_ratio: 0.25,
                daughters: vec![5, 6, 7, 8],
            }
        );
    }

    #[test]
    fn test_parse_decay_table_line_comments() {
        let input = "\
DECAY   6    1.3   # top quark decays
    0.5    2    3   4   # BR(t -> 3 4)
    0.25    3    4   5  6 # BR(t -> 4 5 6)
    0.25    4    5   6  7  8      # BR(t -> 5 6 7 8)
";
        let slha = Slha::parse(input).unwrap();
        let dec = slha.get_decay(6).unwrap();
        assert_eq!(dec.width, 1.3);
        assert_eq!(dec.decays.len(), 3);
        assert_eq!(
            dec.decays[0],
            Decay {
                branching_ratio: 0.5,
                daughters: vec![3, 4],
            }
        );
        assert_eq!(
            dec.decays[1],
            Decay {
                branching_ratio: 0.25,
                daughters: vec![4, 5, 6],
            }
        );
        assert_eq!(
            dec.decays[2],
            Decay {
                branching_ratio: 0.25,
                daughters: vec![5, 6, 7, 8],
            }
        );
    }

    #[test]
    fn test_parse_decay_table_comments() {
        let input = "\
# The decay table for a VERY fictional top quark
DECAY   6    1.3   # top quark decays
    # A top decaying into c and s would be very weird...
    0.5    2    3   4   # BR(t -> 3 4)
    # but not nearly as bad as decaying into c b t
    # where the top actually decays into itself.
    0.25    3    4   5  6 # BR(t -> 4 5 6)
    # And again, top -> top + other crap.
    0.25    4    5   6  7  8      # BR(t -> 5 6 7 8)
    # So, very fictional indeed.
    # But it's just an example to test the parser, so this doesn't matter at all.
";
        let slha = Slha::parse(input).unwrap();
        let dec = slha.get_decay(6).unwrap();
        assert_eq!(dec.width, 1.3);
        assert_eq!(dec.decays.len(), 3);
        assert_eq!(
            dec.decays[0],
            Decay {
                branching_ratio: 0.5,
                daughters: vec![3, 4],
            }
        );
        assert_eq!(
            dec.decays[1],
            Decay {
                branching_ratio: 0.25,
                daughters: vec![4, 5, 6],
            }
        );
        assert_eq!(
            dec.decays[2],
            Decay {
                branching_ratio: 0.25,
                daughters: vec![5, 6, 7, 8],
            }
        );
    }

    #[test]
    fn test_parse_block_str() {
        let input = "\
BLOCK TEST
 1 3
 4 6
block Mass
  6  173.2
BloCk FooBar
  1 2 3 4 0.5
  1 assdf 3 4 8
  1 2 4 8.98
";
        let slha = Slha::parse(input).unwrap();
        println!("{:?}", slha);
        let block: BlockStr<i64> = slha.get_block("test").unwrap().unwrap();
        assert_eq!(block.map.len(), 2);
        assert_eq!(block.map[&vec!["1".to_string()]], 3);
        assert_eq!(block.map[&vec!["4".to_string()]], 6);
        let block: BlockStr<(i64, f64)> = slha.get_block("mass").unwrap().unwrap();
        assert_eq!(block.map.len(), 1);
        assert_eq!(block.map[&Vec::new()], (6, 173.2));
        let block: BlockStr<f64> = slha.get_block("foobar").unwrap().unwrap();
        assert_eq!(block.map.len(), 3);
        assert_eq!(
            block.map[&vec![
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string(),
            ]],
            0.5
        );
        assert_eq!(
            block.map[&vec![
                "1".to_string(),
                "assdf".to_string(),
                "3".to_string(),
                "4".to_string(),
            ]],
            8.
        );
        assert_eq!(
            block.map[&vec!["1".to_string(), "2".to_string(), "4".to_string()]],
            8.98
        );
    }

    #[test]
    fn test_parse_block_single() {
        let input = "\
BLOCK TEST
   3
  ";
        let slha = Slha::parse(input).unwrap();
        println!("{:?}", slha);
        let block: BlockSingle<i64> = slha.get_block("test").unwrap().unwrap();
        assert_eq!(block.value, 3);
    }

    #[test]
    fn test_parse_block_single_comments() {
        let input = "\
# This is a test
BLOCK TEST # A blkoc of type test
# Single blocks only contain one line wiht one isngle value on it
   3  # The value of this single block
# That was it. No more stuff
# is allowed in this block
  ";
        let slha = Slha::parse(input).unwrap();
        println!("{:?}", slha);
        let block: BlockSingle<i64> = slha.get_block("test").unwrap().unwrap();
        assert_eq!(block.value, 3);
    }

    #[test]
    fn test_parse_blocks() {
        // Example file from appendix D.1 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
Block yx
    2  1.9e-01
    3  1.4e-01
Block yd Q= 50
    3  5.3
Block ye Q= 4.64649125e+02
    3  3 9.97405356e-02   # Ytau(Q)MSSM DRbar
Block yu Q= 4.64649125e+02
    3  3 8.88194465e-01   # Yt(Q)MSSM DRbar
Block ye Q= 4.64649125e+03
    3  3 9.97405356e-03   # Ytau(Q)MSSM DRbar
         ";

        let slha = Slha::parse(input).unwrap();
        let yx: Vec<Block<i8, f64>> = slha.get_blocks("yx").unwrap();
        assert_eq!(yx.len(), 1);
        assert_eq!(yx[0].map.len(), 2);
        assert_eq!(yx[0].map[&2], 1.9e-01);
        assert_eq!(yx[0].map[&3], 1.4e-01);
        let yd: Vec<Block<i8, f64>> = slha.get_blocks("yd").unwrap();
        assert_eq!(yd.len(), 1);
        assert_eq!(yd[0].map.len(), 1);
        assert_eq!(yd[0].map[&3], 5.3);
        let yu: Vec<Block<(i8, i8), f64>> = slha.get_blocks("yu").unwrap();
        assert_eq!(yu.len(), 1);
        assert_eq!(yu[0].map.len(), 1);
        assert_eq!(yu[0].map[&(3, 3)], 8.88194465e-01);
        let ye: Vec<Block<(i8, i8), f64>> = slha.get_blocks("ye").unwrap();
        assert_eq!(ye.len(), 2);
        assert_eq!(ye[0].map.len(), 1);
        assert_eq!(ye[0].map[&(3, 3)], 9.97405356e-02);
        assert_eq!(ye[1].map.len(), 1);
        assert_eq!(ye[1].map[&(3, 3)], 9.97405356e-03);
        let foo: Vec<Block<(i8, i8), f64>> = slha.get_blocks("foo").unwrap();
        assert_eq!(foo.len(), 0);
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
Block alpha   # Effective Higgs mixing parameter
          -1.13716828e-01   # alpha
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
        let alpha: BlockSingle<f64> = slha.get_block("alpha").unwrap().unwrap();
        assert_eq!(alpha.value, -1.13716828e-01);
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
    fn test_alpha() {
        // Pieces of the example file from appendix D.2 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "
Block alpha   # Effective Higgs mixing parameter
          -1.13716828e-01   # alpha
";
        let slha = Slha::parse(input).unwrap();
        println!("{:?}", slha);
        let alpha: BlockSingle<f64> = slha.get_block("alpha").unwrap().unwrap();
        assert_eq!(alpha.value, -1.13716828e-01);
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

    #[test]
    fn test_example_decay() {
        // Pieces of the example file from appendix D.3 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
# SUSY Les Houches Accord 1.0 - example decay file
# Info from decay package
Block DCINFO          # Program information
     1    SDECAY       # Decay package
     2    1.0          # version number
#         PDG           Width
DECAY   1000021    1.01752300e+00   # gluino decays
#          BR         NDA      ID1       ID2
    4.18313300E-02     2     1000001        -1   # BR(~g -> ~d_L dbar)
    1.55587600E-02     2     2000001        -1   # BR(~g -> ~d_R dbar)
    3.91391000E-02     2     1000002        -2   # BR(~g -> ~u_L ubar)
    1.74358200E-02     2     2000002        -2   # BR(~g -> ~u_R ubar)
    4.18313300E-02     2     1000003        -3   # BR(~g -> ~s_L sbar)
    1.55587600E-02     2     2000003        -3   # BR(~g -> ~s_R sbar)
    3.91391000E-02     2     1000004        -4   # BR(~g -> ~c_L cbar)
    1.74358200E-02     2     2000004        -4   # BR(~g -> ~c_R cbar)
    1.13021900E-01     2     1000005        -5   # BR(~g -> ~b_1 bbar)
    6.30339800E-02     2     2000005        -5   # BR(~g -> ~b_2 bbar)
    9.60140900E-02     2     1000006        -6   # BR(~g -> ~t_1 tbar)
    0.00000000E+00     2     2000006        -6   # BR(~g -> ~t_2 tbar)
    4.18313300E-02     2    -1000001         1   # BR(~g -> ~dbar_L d)
    1.55587600E-02     2    -2000001         1   # BR(~g -> ~dbar_R d)
    3.91391000E-02     2    -1000002         2   # BR(~g -> ~ubar_L u)
    1.74358200E-02     2    -2000002         2   # BR(~g -> ~ubar_R u)
    4.18313300E-02     2    -1000003         3   # BR(~g -> ~sbar_L s)
    1.55587600E-02     2    -2000003         3   # BR(~g -> ~sbar_R s)
    3.91391000E-02     2    -1000004         4   # BR(~g -> ~cbar_L c)
    1.74358200E-02     2    -2000004         4   # BR(~g -> ~cbar_R c)
";
        let slha = Slha::parse(input).unwrap();
        let dcinfo: Block<u8, String> = slha.get_block("dcinfo").unwrap().unwrap();
        assert_eq!(dcinfo.map.len(), 2);
        assert_eq!(dcinfo.map[&1], "SDECAY");
        assert_eq!(dcinfo.map[&2], "1.0");
        let dec = slha.get_decay(1000021).unwrap();
        assert_eq!(dec.width, 1.01752300e+00);
        assert_eq!(dec.decays.len(), 20);
        assert_eq!(
            dec.decays[0],
            Decay {
                branching_ratio: 4.18313300E-02,
                daughters: vec![1000001, -1],
            }
        );
        assert_eq!(
            dec.decays[1],
            Decay {
                branching_ratio: 1.55587600E-02,
                daughters: vec![2000001, -1],
            }
        );
        assert_eq!(
            dec.decays[2],
            Decay {
                branching_ratio: 3.91391000E-02,
                daughters: vec![1000002, -2],
            }
        );
        assert_eq!(
            dec.decays[3],
            Decay {
                branching_ratio: 1.74358200E-02,
                daughters: vec![2000002, -2],
            }
        );
        assert_eq!(
            dec.decays[4],
            Decay {
                branching_ratio: 4.18313300E-02,
                daughters: vec![1000003, -3],
            }
        );
        assert_eq!(
            dec.decays[5],
            Decay {
                branching_ratio: 1.55587600E-02,
                daughters: vec![2000003, -3],
            }
        );
        assert_eq!(
            dec.decays[6],
            Decay {
                branching_ratio: 3.91391000E-02,
                daughters: vec![1000004, -4],
            }
        );
        assert_eq!(
            dec.decays[7],
            Decay {
                branching_ratio: 1.74358200E-02,
                daughters: vec![2000004, -4],
            }
        );
        assert_eq!(
            dec.decays[8],
            Decay {
                branching_ratio: 1.13021900E-01,
                daughters: vec![1000005, -5],
            }
        );
        assert_eq!(
            dec.decays[9],
            Decay {
                branching_ratio: 6.30339800E-02,
                daughters: vec![2000005, -5],
            }
        );
        assert_eq!(
            dec.decays[10],
            Decay {
                branching_ratio: 9.60140900E-02,
                daughters: vec![1000006, -6],
            }
        );
        assert_eq!(
            dec.decays[11],
            Decay {
                branching_ratio: 0.00000000E+00,
                daughters: vec![2000006, -6],
            }
        );
        assert_eq!(
            dec.decays[12],
            Decay {
                branching_ratio: 4.18313300E-02,
                daughters: vec![-1000001, 1],
            }
        );
        assert_eq!(
            dec.decays[13],
            Decay {
                branching_ratio: 1.55587600E-02,
                daughters: vec![-2000001, 1],
            }
        );
        assert_eq!(
            dec.decays[14],
            Decay {
                branching_ratio: 3.91391000E-02,
                daughters: vec![-1000002, 2],
            }
        );
        assert_eq!(
            dec.decays[15],
            Decay {
                branching_ratio: 1.74358200E-02,
                daughters: vec![-2000002, 2],
            }
        );
        assert_eq!(
            dec.decays[16],
            Decay {
                branching_ratio: 4.18313300E-02,
                daughters: vec![-1000003, 3],
            }
        );
        assert_eq!(
            dec.decays[17],
            Decay {
                branching_ratio: 1.55587600E-02,
                daughters: vec![-2000003, 3],
            }
        );
        assert_eq!(
            dec.decays[18],
            Decay {
                branching_ratio: 3.91391000E-02,
                daughters: vec![-1000004, 4],
            }
        );
        assert_eq!(
            dec.decays[19],
            Decay {
                branching_ratio: 1.74358200E-02,
                daughters: vec![-2000004, 4],
            }
        );
    }


    #[test]
    fn test_incomplete_parse() {
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
     1  1  100.0     # m0
     2    250.0     # m12
     5   -100.0     # A0 ";

        let slha = Slha::parse(input).unwrap();
        let block: Result<Block<i8, f64>, Error> = slha.get_block("minpar").unwrap();
        let err = block.unwrap_err();
        if let Error(ErrorKind::InvalidBlock(name), _) = err {
            assert_eq!(&name, "minpar");
        } else {
            panic!("Wrong error variant {:?} instead of InvalidBlock", err);
        }
    }

    #[test]
    fn test_unexpected_eol() {
        // Example file from appendix D.1 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
# SUSY Les Houches Accord 1.0 - example input file
# Snowmsas point 1a
Block MODSEL  # Select model
     1    1   # sugra
Block SMINPUTS   # Standard Model inputs
     3      0.1172  # alpha_s(MZ) SM MSbar
     5          # Mb(mb) SM MSbar
     6    174.3     # Mtop(pole)
Block MINPAR  # SUSY breaking input parameters
     3     10.0     # tanb
     4      1.0     # sign(mu)
     1    100.0     # m0
     2    250.0     # m12
         5   -100.0     # A0 ";

        let slha = Slha::parse(input).unwrap();
        let block: Result<Block<i8, f64>, Error> = slha.get_block("sminputs").unwrap();
        let err = block.unwrap_err();
        if let Error(ErrorKind::InvalidBlock(name), _) = err {
            assert_eq!(&name, "sminputs");
        } else {
            panic!("Wrong error variant {:?} instead of InvalidBlock", err);
        }
    }

    #[test]
    fn test_unexpected_eol_tuple() {
        // Example file from appendix D.1 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
Block yu Q= 4.64649125e+02
    3  3 8.88194465e-01   # Yt(Q)MSSM DRbar
Block yx Q= 40
    3  3 1.4e-01
Block yd Q= 50
    3
Block ye Q= 4.64649125e+02
    3  3 9.97405356e-02   # Ytau(Q)MSSM DRbar
Block ye Q= 4.64649125e+03
    3  3 9.97405356e-03   # Ytau(Q)MSSM DRbar
         ";

        let slha = Slha::parse(input).unwrap();
        let block: Result<Block<i8, f64>, Error> = slha.get_block("yd").unwrap();
        let err = block.unwrap_err();
        if let Error(ErrorKind::InvalidBlock(name), _) = err {
            assert_eq!(&name, "yd");
        } else {
            panic!("Wrong error variant {:?} instead of InvalidBlock", err);
        }
    }

    #[test]
    fn test_invalid_int() {
        // Example file from appendix D.1 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
# SUSY Les Houches Accord 1.0 - example input file
# Snowmsas point 1a
Block MODSEL  # Select model
     1    1   # sugra
Block SMINPUTS   # Standard Model inputs
     3      0.1172  # alpha_s(MZ) SM MSbar
     a      1.23    # Mb(mb) SM MSbar
     6    174.3     # Mtop(pole)
Block MINPAR  # SUSY breaking input parameters
     3     10.0     # tanb
     4      1.0     # sign(mu)
     1    100.0     # m0
     2    250.0     # m12
     5   -100.0     # A0 ";

        let slha = Slha::parse(input).unwrap();
        let block: Result<Block<i8, f64>, Error> = slha.get_block("sminputs").unwrap();
        let err = block.unwrap_err();
        if let Error(ErrorKind::InvalidBlock(name), _) = err {
            assert_eq!(&name, "sminputs");
        } else {
            panic!("Wrong error variant {:?} instead of InvalidBlock", err);
        }
    }

    #[test]
    fn test_invalid_float() {
        // Example file from appendix D.1 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
Block yu Q= 4.64649125e+02
    3  3 8.8819a465e-01   # Yt(Q)MSSM DRbar
Block yd Q= 40
    3  3 1.4e-01
Block yd Q= 50
    3  3 1.4e-01
Block ye Q= 4.64649125e+02
    3  3 9.97405356e-02   # Ytau(Q)MSSM DRbar
Block ye Q= 4.64649125e+03
    3  3 9.97405356e-03   # Ytau(Q)MSSM DRbar
         ";

        let slha = Slha::parse(input).unwrap();
        let block: Result<Block<i8, f64>, Error> = slha.get_block("yu").unwrap();
        let err = block.unwrap_err();
        if let Error(ErrorKind::InvalidBlock(name), _) = err {
            assert_eq!(&name, "yu");
        } else {
            panic!("Wrong error variant {:?} instead of InvalidBlock", err);
        }
    }

    #[test]
    fn test_unknown_segment() {
        // Example file from appendix D.1 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
Block yu Q= 4.64649125e+02
    3  3 8.88193465e-01   # Yt(Q)MSSM DRbar
Block yd Q= 40
    3  3 1.4e-01
Block yd Q= 50
    3  3 1.4e-01
Block ye Q= 4.64649125e+02
    3  3 9.97405356e-02   # Ytau(Q)MSSM DRbar
FLUP ye Q= 4.64649125e+03
    3  3 9.97405356e-03   # Ytau(Q)MSSM DRbar
         ";

        let err = Slha::parse(input).unwrap_err();
        if let Error(ErrorKind::UnknownSegment(name), _) = err {
            assert_eq!(&name, "flup");
        } else {
            panic!("Wrong error variant {:?} instead of UnknownSegment", err);
        }
    }

    #[test]
    fn test_unexpected_ident() {
        // Example file from appendix D.1 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
# SUSY Les Houches Accord 1.0 - example input file
# Snowmsas point 1a
 Block MODSEL  # Select model
     1    1   # sugra
Block SMINPUTS   # Standard Model inputs
     3      0.1172  # alpha_s(MZ) SM MSbar
     5      1.23    # Mb(mb) SM MSbar
     6    174.3     # Mtop(pole)
Block MINPAR  # SUSY breaking input parameters
     3     10.0     # tanb
     4      1.0     # sign(mu)
     1    100.0     # m0
     2    250.0     # m12
     5   -100.0     # A0 ";

        let err = Slha::parse(input).unwrap_err();
        if let Error(ErrorKind::UnexpectedIdent(line), _) = err {
            assert_eq!(&line, " Block MODSEL  # Select model");
        } else {
            panic!("Wrong error variant {:?} instead of UnexpectedIdent", err);
        }
    }

    #[test]
    fn test_missing_block_name() {
        // Example file from appendix D.1 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
Block yu Q= 4.64649125e+02
    3  3 8.88193465e-01   # Yt(Q)MSSM DRbar
Block
    3  3 1.4e-01
Block yd Q= 50
    3  3 1.4e-01
Block yf Q= 4.64649125e+02
    3  3 9.97405356e-02   # Ytau(Q)MSSM DRbar
Block flup Q= 4.64649125e+03
    3  3 9.97405356e-03   # Ytau(Q)MSSM DRbar
         ";

        let err = Slha::parse(input).unwrap_err();
        if let Error(ErrorKind::MissingBlockName, _) = err {
        } else {
            panic!("Wrong error variant {:?} instead of MissingBlockName", err);
        }
    }

    #[test]
    fn test_malformed_block_header() {
        // Example file from appendix D.1 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
# SUSY Les Houches Accord 1.0 - example input file
# Snowmsas point 1a
Block MODSEL  # Select model
     1    1   # sugra
Block SM INPUTS   # Standard Model inputs
     3      0.1172  # alpha_s(MZ) SM MSbar
     5      1.23    # Mb(mb) SM MSbar
     6    174.3     # Mtop(pole)
Block MINPAR  # SUSY breaking input parameters
     3     10.0     # tanb
     4      1.0     # sign(mu)
     1    100.0     # m0
     2    250.0     # m12
     5   -100.0     # A0 ";

        let err = Slha::parse(input).unwrap_err();
        if let Error(ErrorKind::InvalidBlock(name), _) = err {
            assert_eq!(&name, "sm");
        } else {
            panic!("Wrong error variant {:?} instead of InvalidBlock", err);
        }
    }

    #[test]
    fn test_duplicate_block() {
        // Example file from appendix D.1 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
# SUSY Les Houches Accord 1.0 - example input file
# Snowmsas point 1a
Block MODSEL  # Select model
     1    1   # sugra
Block SMINPUTS   # Standard Model inputs
     3      0.1172  # alpha_s(MZ) SM MSbar
     5      1.23    # Mb(mb) SM MSbar
     6    174.3     # Mtop(pole)
Block MODsel  # SUSY breaking input parameters
     3     10.0     # tanb
     4      1.0     # sign(mu)
     1    100.0     # m0
     2    250.0     # m12
     5   -100.0     # A0 ";

        let slha = Slha::parse(input).unwrap();
        let err: Result<Block<i8, f64>, Error> = slha.get_block("modsel").unwrap();
        let err = err.unwrap_err();
        if let Error(ErrorKind::DuplicateBlock(name), _) = err {
            assert_eq!(&name, "modsel");
        } else {
            panic!("Wrong error variant {:?} instead of DuplicateBlock", err);
        }
    }

    #[test]
    fn test_duplicate_block_unchecked() {
        // Example file from appendix D.1 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
# SUSY Les Houches Accord 1.0 - example input file
# Snowmsas point 1a
Block MODSEL  # Select model
     1    1   # sugra
Block SMINPUTS   # Standard Model inputs
     3      0.1172  # alpha_s(MZ) SM MSbar
     5      1.23    # Mb(mb) SM MSbar
     6    174.3     # Mtop(pole)
Block MODsel  # SUSY breaking input parameters
     3     10.0     # tanb
     4      1.0     # sign(mu)
     1    100.0     # m0
     2    250.0     # m12
     5   -100.0     # A0 ";

        let slha = Slha::parse(input).unwrap();
        let blocks: Vec<Block<i8, f64>> = slha.get_blocks_unchecked("modsel").unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].map[&1], 1.0);
        assert_eq!(blocks[1].map[&1], 100.0);
    }

    #[test]
    fn test_duplicate_block_scale() {
        // Example file from appendix D.1 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
Block yu Q= 4.64649125e+02
    3  3 8.88193465e-01   # Yt(Q)MSSM DRbar
Block yu Q= 8
    3  3 1.4e-01
Block yd Q= 50
    3  3 1.4e-01
Block yf Q= 4.64649125e+02
    3  3 9.97405356e-02   # Ytau(Q)MSSM DRbar
Block flup Q= 4.64649125e+03
        3  3 9.97405356e-03   # Ytau(Q)MSSM DRbar
         ";

        let slha = Slha::parse(input).unwrap();
        let block: Result<Block<(i8, i8), f64>, Error> = slha.get_block("yu").unwrap();
        let err = block.unwrap_err();
        if let Error(ErrorKind::DuplicateBlock(name), _) = err {
            assert_eq!(&name, "yu");
        } else {
            panic!("Wrong error variant {:?} instead of DuplicateBlock", err);
        }
    }

    #[test]
    fn test_duplicate_block_scale_unchecked() {
        // Example file from appendix D.1 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
Block yu Q= 4.64649125e+02
    3  3 8.88193465e-01   # Yt(Q)MSSM DRbar
Block yu Q= 8
    3  3 1.4e-01
Block yd Q= 50
    3  3 1.4e-01
Block yf Q= 4.64649125e+02
    3  3 9.97405356e-02   # Ytau(Q)MSSM DRbar
Block flup Q= 4.64649125e+03
        3  3 9.97405356e-03   # Ytau(Q)MSSM DRbar
         ";

        let slha = Slha::parse(input).unwrap();
        let blocks: Vec<Block<(i8, i8), f64>> = slha.get_blocks_unchecked("yu").unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].scale, Some(4.64649125e+02));
        assert_eq!(blocks[0].map[&(3, 3)], 8.88193465e-01);
        assert_eq!(blocks[1].scale, Some(8.));
        assert_eq!(blocks[1].map[&(3, 3)], 1.4e-01);
    }

    #[test]
    fn test_duplicate_block_equal_scale() {
        // Example file from appendix D.1 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
Block yf Q= 4.64649125e+02
    3  3 8.88193465e-01   # Yt(Q)MSSM DRbar
Block yu Q= 8
    3  3 1.4e-01
Block yd Q= 50
    3  3 1.4e-01
Block yf Q= 4.64649125e+02
    3  3 9.97405356e-02   # Ytau(Q)MSSM DRbar
Block flup Q= 4.64649125e+03
    3  3 9.97405356e-03   # Ytau(Q)MSSM DRbar
         ";

        let slha = Slha::parse(input).unwrap();
        let block: Result<Vec<Block<(i8, i8), f64>>, Error> = slha.get_blocks("yf");
        let err = block.unwrap_err();
        if let Error(ErrorKind::DuplicateBlockScale(name, scale), _) = err {
            assert_eq!(&name, "yf");
            assert_eq!(scale, 4.64649125e+02);
        } else {
            panic!(
                "Wrong error variant {:?} instead of DuplicateBlockScale",
                err
            );
        }
    }

    #[test]
    fn test_duplicate_block_equal_scale_unchecked() {
        // Example file from appendix D.1 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
Block yf Q= 4.64649125e+02
    3  3 8.88193465e-01   # Yt(Q)MSSM DRbar
Block yu Q= 8
    3  3 1.4e-01
Block yd Q= 50
    3  3 1.4e-01
Block yf Q= 4.64649125e+02
    3  3 9.97405356e-02   # Ytau(Q)MSSM DRbar
Block flup Q= 4.64649125e+03
    3  3 9.97405356e-03   # Ytau(Q)MSSM DRbar
         ";

        let slha = Slha::parse(input).unwrap();
        let blocks: Vec<Block<(i8, i8), f64>> = slha.get_blocks_unchecked("yf").unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].scale, Some(4.64649125e+02));
        assert_eq!(blocks[0].map[&(3, 3)], 8.88193465e-01);
        assert_eq!(blocks[1].scale, Some(4.64649125e+02));
        assert_eq!(blocks[1].map[&(3, 3)], 9.97405356e-02);
    }

    #[test]
    fn test_redefined_block_with_scale_1() {
        // Example file from appendix D.1 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
Block yf
    3  3 8.88193465e-01   # Yt(Q)MSSM DRbar
Block yu Q= 8
    3  3 1.4e-01
Block yd Q= 50
    3  3 1.4e-01
Block yf Q= 4.64649125e+02
    3  3 9.97405356e-02   # Ytau(Q)MSSM DRbar
Block flup Q= 4.64649125e+03
    3  3 9.97405356e-03   # Ytau(Q)MSSM DRbar
         ";

        let slha = Slha::parse(input).unwrap();
        let block: Result<Vec<Block<(i8, i8), f64>>, Error> = slha.get_blocks("yf");
        let err = block.unwrap_err();
        if let Error(ErrorKind::RedefinedBlockWithQ(name), _) = err {
            assert_eq!(&name, "yf");
        } else {
            panic!(
                "Wrong error variant {:?} instead of RedefinedBlockWithQ",
                err
            );
        }
    }

    #[test]
    fn test_redefined_block_with_scale_1_unchecked() {
        // Example file from appendix D.1 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
Block yf
    3  3 8.88193465e-01   # Yt(Q)MSSM DRbar
Block yu Q= 8
    3  3 1.4e-01
Block yd Q= 50
    3  3 1.4e-01
Block yf Q= 4.64649125e+02
    3  3 9.97405356e-02   # Ytau(Q)MSSM DRbar
Block flup Q= 4.64649125e+03
    3  3 9.97405356e-03   # Ytau(Q)MSSM DRbar
         ";

        let slha = Slha::parse(input).unwrap();
        let blocks: Vec<Block<(i8, i8), f64>> = slha.get_blocks_unchecked("yf").unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].scale, None);
        assert_eq!(blocks[0].map[&(3, 3)], 8.88193465e-01);
        assert_eq!(blocks[1].scale, Some(4.64649125e+02));
        assert_eq!(blocks[1].map[&(3, 3)], 9.97405356e-02);
    }

    #[test]
    fn test_redefined_block_with_scale_2() {
        // Example file from appendix D.1 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
Block yf Q= 4.64649125e+02
    3  3 8.88193465e-01   # Yt(Q)MSSM DRbar
Block yu Q= 8
    3  3 1.4e-01
Block yd Q= 50
    3  3 1.4e-01
Block yf
    3  3 9.97405356e-02   # Ytau(Q)MSSM DRbar
Block flup Q= 4.64649125e+03
        3  3 9.97405356e-03   # Ytau(Q)MSSM DRbar
         ";

        let slha = Slha::parse(input).unwrap();
        let block: Result<Vec<Block<(i8, i8), f64>>, Error> = slha.get_blocks("yf");
        let err = block.unwrap_err();
        if let Error(ErrorKind::RedefinedBlockWithQ(name), _) = err {
            assert_eq!(&name, "yf");
        } else {
            panic!(
                "Wrong error variant {:?} instead of RedefinedBlockWithQ",
                err
            );
        }
    }

    #[test]
    fn test_redefined_block_with_scale_2_unchecked() {
        // Example file from appendix D.1 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
Block yf Q= 4.64649125e+02
    3  3 8.88193465e-01   # Yt(Q)MSSM DRbar
Block yu Q= 8
    3  3 1.4e-01
Block yd Q= 50
    3  3 1.4e-01
Block yf
    3  3 9.97405356e-02   # Ytau(Q)MSSM DRbar
Block flup Q= 4.64649125e+03
    3  3 9.97405356e-03   # Ytau(Q)MSSM DRbar
         ";

        let slha = Slha::parse(input).unwrap();
        let blocks: Vec<Block<(i8, i8), f64>> = slha.get_blocks_unchecked("yf").unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].scale, Some(4.64649125e+02));
        assert_eq!(blocks[0].map[&(3, 3)], 8.88193465e-01);
        assert_eq!(blocks[1].scale, None);
        assert_eq!(blocks[1].map[&(3, 3)], 9.97405356e-02);
    }

    #[test]
    fn test_invalid_scale() {
        // Example file from appendix D.1 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
Block yu Q= 4.64649125e+02
    3  3 8.88193465e-01   # Yt(Q)MSSM DRbar
Block yd Q= 40
    3  3 1.4e-01
Block yd Q=
    3  3 1.4e-01
Block ye Q= 3.23 scale # comment
    3  3 9.97405356e-02   # Ytau(Q)MSSM DRbar
Block flup Q= 4.64649125e+03
    3  3 9.97405356e-03   # Ytau(Q)MSSM DRbar
         ";

        let err = Slha::parse(input).unwrap_err();
        if let Error(ErrorKind::InvalidBlock(name), _) = err {
            assert_eq!(&name, "yd");
        } else {
            panic!("Wrong error variant {:?} instead of InvalidBlock", err);
        }
    }

    #[test]
    fn test_invalid_scale_trailing() {
        // Example file from appendix D.1 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
Block yu Q= 4.64649125e+02
    3  3 8.88193465e-01   # Yt(Q)MSSM DRbar
Block yd Q= 40
    3  3 1.4e-01
Block yd Q= 50
    3  3 1.4e-01
Block ye Q= 70 other stuff # comment
    3  3 9.97405356e-02   # Ytau(Q)MSSM DRbar
Block flup Q= 4.64649125e+03
    3  3 9.97405356e-03   # Ytau(Q)MSSM DRbar
         ";

        let err = Slha::parse(input).unwrap_err();
        if let Error(ErrorKind::InvalidBlock(name), _) = err {
            assert_eq!(&name, "ye");
        } else {
            panic!("Wrong error variant {:?} instead of InvalidBlock", err);
        }
    }

    #[test]
    fn test_duplicate_decay() {
        // Pieces of the example file from appendix D.3 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
DECAY   1000021    1.01752300e+00   # gluino decays
    4.18313300E-02     2     1000001        -1   # BR(~g -> ~d_L dbar)
    1.55587600E-02     2     2000001        -1   # BR(~g -> ~d_R dbar)
    3.91391000E-02     2     1000002        -2   # BR(~g -> ~u_L ubar)
    1.74358200E-02     2     2000002        -2   # BR(~g -> ~u_R ubar)
    4.18313300E-02     2     1000003        -3   # BR(~g -> ~s_L sbar)
DECAY   1000022    1.01752300e+00   # gluino decays
    1.55587600E-02     2     2000003        -3   # BR(~g -> ~s_R sbar)
    3.91391000E-02     2     1000004        -4   # BR(~g -> ~c_L cbar)
    1.74358200E-02     2     2000004        -4   # BR(~g -> ~c_R cbar)
    1.13021900E-01     2     1000005        -5   # BR(~g -> ~b_1 bbar)
    6.30339800E-02     2     2000005        -5   # BR(~g -> ~b_2 bbar)
    9.60140900E-02     2     1000006        -6   # BR(~g -> ~t_1 tbar)
    0.00000000E+00     2     2000006        -6   # BR(~g -> ~t_2 tbar)
DECAY   1000025    1.01752300e+00   # gluino decays
    4.18313300E-02     2    -1000001         1   # BR(~g -> ~dbar_L d)
    1.55587600E-02     2    -2000001         1   # BR(~g -> ~dbar_R d)
    3.91391000E-02     2    -1000002         2   # BR(~g -> ~ubar_L u)
    1.74358200E-02     2    -2000002         2   # BR(~g -> ~ubar_R u)
    4.18313300E-02     2    -1000003         3   # BR(~g -> ~sbar_L s)
DECAY   1000022    1.01752300e+00   # gluino decays
    1.55587600E-02     2    -2000003         3   # BR(~g -> ~sbar_R s)
    3.91391000E-02     2    -1000004         4   # BR(~g -> ~cbar_L c)
    1.74358200E-02     2    -2000004         4   # BR(~g -> ~cbar_R c)
    ";

        let err = Slha::parse(input).unwrap_err();
        if let Error(ErrorKind::DuplicateDecay(pdg_id), _) = err {
            assert_eq!(pdg_id, 1000022);
        } else {
            panic!("Wrong error variant {:?} instead of DuplicateDecay", err);
        }
    }

    #[test]
    fn test_missing_decaying_particle() {
        // Pieces of the example file from appendix D.3 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
DECAY   1000021    1.01752300e+00   # gluino decays
    4.18313300E-02     2     1000001        -1   # BR(~g -> ~d_L dbar)
    1.55587600E-02     2     2000001        -1   # BR(~g -> ~d_R dbar)
    3.91391000E-02     2     1000002        -2   # BR(~g -> ~u_L ubar)
    1.74358200E-02     2     2000002        -2   # BR(~g -> ~u_R ubar)
    4.18313300E-02     2     1000003        -3   # BR(~g -> ~s_L sbar)
DECAY   1000022    1.01752300e+00   # gluino decays
    1.55587600E-02     2     2000003        -3   # BR(~g -> ~s_R sbar)
    3.91391000E-02     2     1000004        -4   # BR(~g -> ~c_L cbar)
    1.74358200E-02     2     2000004        -4   # BR(~g -> ~c_R cbar)
    1.13021900E-01     2     1000005        -5   # BR(~g -> ~b_1 bbar)
    6.30339800E-02     2     2000005        -5   # BR(~g -> ~b_2 bbar)
    9.60140900E-02     2     1000006        -6   # BR(~g -> ~t_1 tbar)
    0.00000000E+00     2     2000006        -6   # BR(~g -> ~t_2 tbar)
DECAY      # gluino decays
    4.18313300E-02     2    -1000001         1   # BR(~g -> ~dbar_L d)
    1.55587600E-02     2    -2000001         1   # BR(~g -> ~dbar_R d)
    3.91391000E-02     2    -1000002         2   # BR(~g -> ~ubar_L u)
    1.74358200E-02     2    -2000002         2   # BR(~g -> ~ubar_R u)
    4.18313300E-02     2    -1000003         3   # BR(~g -> ~sbar_L s)
DECAY   1000020    1.01752300e+00   # gluino decays
    1.55587600E-02     2    -2000003         3   # BR(~g -> ~sbar_R s)
    3.91391000E-02     2    -1000004         4   # BR(~g -> ~cbar_L c)
    1.74358200E-02     2    -2000004         4   # BR(~g -> ~cbar_R c)
";

        let err = Slha::parse(input).unwrap_err();
        if let Error(ErrorKind::InvalidDecayingPdgId, _) = err {
        } else {
            panic!(
                "Wrong error variant {:?} instead of InvalidDecayingPdgId",
                err
            );
        }
    }

    #[test]
    fn test_invalid_pdg_id() {
        // Pieces of the example file from appendix D.3 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
DECAY   1000021    1.01752300e+00   # gluino decays
    4.18313300E-02     2     1000001        -1   # BR(~g -> ~d_L dbar)
    1.55587600E-02     2     2000001        -1   # BR(~g -> ~d_R dbar)
    3.91391000E-02     2     1000002        -2   # BR(~g -> ~u_L ubar)
    1.74358200E-02     2     2000002        -2   # BR(~g -> ~u_R ubar)
    4.18313300E-02     2     1000003        -3   # BR(~g -> ~s_L sbar)
DECAY   1000022    1.01752300e+00   # gluino decays
    1.55587600E-02     2     2000003        -3   # BR(~g -> ~s_R sbar)
    3.91391000E-02     2     1000004        -4   # BR(~g -> ~c_L cbar)
    1.74358200E-02     2     2000004        -4   # BR(~g -> ~c_R cbar)
    1.13021900E-01     2     1000005        -5   # BR(~g -> ~b_1 bbar)
    6.30339800E-02     2     2000005        -5   # BR(~g -> ~b_2 bbar)
    9.60140900E-02     2     1000006        -6   # BR(~g -> ~t_1 tbar)
    0.00000000E+00     2     2000006        -6   # BR(~g -> ~t_2 tbar)
DECAY   100a025    1.01752300e+00   # gluino decays
    4.18313300E-02     2    -1000001         1   # BR(~g -> ~dbar_L d)
    1.55587600E-02     2    -2000001         1   # BR(~g -> ~dbar_R d)
    3.91391000E-02     2    -1000002         2   # BR(~g -> ~ubar_L u)
    1.74358200E-02     2    -2000002         2   # BR(~g -> ~ubar_R u)
    4.18313300E-02     2    -1000003         3   # BR(~g -> ~sbar_L s)
DECAY   1000020    1.01752300e+00   # gluino decays
    1.55587600E-02     2    -2000003         3   # BR(~g -> ~sbar_R s)
    3.91391000E-02     2    -1000004         4   # BR(~g -> ~cbar_L c)
    1.74358200E-02     2    -2000004         4   # BR(~g -> ~cbar_R c)
";

        let err = Slha::parse(input).unwrap_err();
        if let Error(ErrorKind::InvalidDecayingPdgId, _) = err {
        } else {
            panic!(
                "Wrong error variant {:?} instead of InvalidDecayingPdgId",
                err
            );
        }
    }

    #[test]
    fn test_invalid_width() {
        // Pieces of the example file from appendix D.3 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
DECAY   1000021    1.01752300e+00   # gluino decays
    4.18313300E-02     2     1000001        -1   # BR(~g -> ~d_L dbar)
    1.55587600E-02     2     2000001        -1   # BR(~g -> ~d_R dbar)
    3.91391000E-02     2     1000002        -2   # BR(~g -> ~u_L ubar)
    1.74358200E-02     2     2000002        -2   # BR(~g -> ~u_R ubar)
    4.18313300E-02     2     1000003        -3   # BR(~g -> ~s_L sbar)
DECAY   1000022    1.01752300e+00   # gluino decays
    1.55587600E-02     2     2000003        -3   # BR(~g -> ~s_R sbar)
    3.91391000E-02     2     1000004        -4   # BR(~g -> ~c_L cbar)
    1.74358200E-02     2     2000004        -4   # BR(~g -> ~c_R cbar)
    1.13021900E-01     2     1000005        -5   # BR(~g -> ~b_1 bbar)
    6.30339800E-02     2     2000005        -5   # BR(~g -> ~b_2 bbar)
    9.60140900E-02     2     1000006        -6   # BR(~g -> ~t_1 tbar)
    0.00000000E+00     2     2000006        -6   # BR(~g -> ~t_2 tbar)
DECAY   1000025     1,01752300e+00  # gluino decays
    4.18313300E-02     2    -1000001         1   # BR(~g -> ~dbar_L d)
    1.55587600E-02     2    -2000001         1   # BR(~g -> ~dbar_R d)
    3.91391000E-02     2    -1000002         2   # BR(~g -> ~ubar_L u)
    1.74358200E-02     2    -2000002         2   # BR(~g -> ~ubar_R u)
    4.18313300E-02     2    -1000003         3   # BR(~g -> ~sbar_L s)
DECAY   1000020    1.01752300e+00   # gluino decays
    1.55587600E-02     2    -2000003         3   # BR(~g -> ~sbar_R s)
    3.91391000E-02     2    -1000004         4   # BR(~g -> ~cbar_L c)
    1.74358200E-02     2    -2000004         4   # BR(~g -> ~cbar_R c)
";

        let err = Slha::parse(input).unwrap_err();
        if let Error(ErrorKind::InvalidDecay(pdg_id), _) = err {
            assert_eq!(pdg_id, 1000025);
        } else {
            panic!("Wrong error variant {:?} instead of InvalidDecay", err);
        }
    }

    #[test]
    fn test_missing_width() {
        // Pieces of the example file from appendix D.3 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
DECAY   1000021    1.01752300e+00   # gluino decays
    4.18313300E-02     2     1000001        -1   # BR(~g -> ~d_L dbar)
    1.55587600E-02     2     2000001        -1   # BR(~g -> ~d_R dbar)
    3.91391000E-02     2     1000002        -2   # BR(~g -> ~u_L ubar)
    1.74358200E-02     2     2000002        -2   # BR(~g -> ~u_R ubar)
    4.18313300E-02     2     1000003        -3   # BR(~g -> ~s_L sbar)
DECAY   1000022       # gluino decays
    1.55587600E-02     2     2000003        -3   # BR(~g -> ~s_R sbar)
    3.91391000E-02     2     1000004        -4   # BR(~g -> ~c_L cbar)
    1.74358200E-02     2     2000004        -4   # BR(~g -> ~c_R cbar)
    1.13021900E-01     2     1000005        -5   # BR(~g -> ~b_1 bbar)
    6.30339800E-02     2     2000005        -5   # BR(~g -> ~b_2 bbar)
    9.60140900E-02     2     1000006        -6   # BR(~g -> ~t_1 tbar)
    0.00000000E+00     2     2000006        -6   # BR(~g -> ~t_2 tbar)
DECAY   1000025    1.043634   # gluino decays
    4.18313300E-02     2    -1000001         1   # BR(~g -> ~dbar_L d)
    1.55587600E-02     2    -2000001         1   # BR(~g -> ~dbar_R d)
    3.91391000E-02     2    -1000002         2   # BR(~g -> ~ubar_L u)
    1.74358200E-02     2    -2000002         2   # BR(~g -> ~ubar_R u)
    4.18313300E-02     2    -1000003         3   # BR(~g -> ~sbar_L s)
DECAY   1000020    1.01752300e+00   # gluino decays
    1.55587600E-02     2    -2000003         3   # BR(~g -> ~sbar_R s)
    3.91391000E-02     2    -1000004         4   # BR(~g -> ~cbar_L c)
    1.74358200E-02     2    -2000004         4   # BR(~g -> ~cbar_R c)
";

        let err = Slha::parse(input).unwrap_err();
        if let Error(ErrorKind::InvalidDecay(pdg_id), _) = err {
            assert_eq!(pdg_id, 1000022);
        } else {
            panic!("Wrong error variant {:?} instead of InvalidDecay", err);
        }
    }

    #[test]
    fn test_invalid_branchingratio() {
        // Pieces of the example file from appendix D.3 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
DECAY   1000021    1.01752300e+00   # gluino decays
    4.18313300E-02     2     1000001        -1   # BR(~g -> ~d_L dbar)
    1.55587600E-02     2     2000001        -1   # BR(~g -> ~d_R dbar)
    3x91391000E-02     2     1000002        -2   # BR(~g -> ~u_L ubar)
    1.74358200E-02     2     2000002        -2   # BR(~g -> ~u_R ubar)
    4.18313300E-02     2     1000003        -3   # BR(~g -> ~s_L sbar)
DECAY   1000022    1.01752300e+00   # gluino decays
    1.55587600E-02     2     2000003        -3   # BR(~g -> ~s_R sbar)
    3.91391000E-02     2     1000004        -4   # BR(~g -> ~c_L cbar)
    1.74358200E-02     2     2000004        -4   # BR(~g -> ~c_R cbar)
    1.13021900E-01     2     1000005        -5   # BR(~g -> ~b_1 bbar)
    6.30339800E-02     2     2000005        -5   # BR(~g -> ~b_2 bbar)
    9.60140900E-02     2     1000006        -6   # BR(~g -> ~t_1 tbar)
    0.00000000E+00     2     2000006        -6   # BR(~g -> ~t_2 tbar)
DECAY   1000025     1.01752300e+00  # gluino decays
    4.18313300E-02     2    -1000001         1   # BR(~g -> ~dbar_L d)
    1.55587600E-02     2    -2000001         1   # BR(~g -> ~dbar_R d)
    3.91391000E-02     2    -1000002         2   # BR(~g -> ~ubar_L u)
    1.74358200E-02     2    -2000002         2   # BR(~g -> ~ubar_R u)
    4.18313300E-02     2    -1000003         3   # BR(~g -> ~sbar_L s)
DECAY   1000020    1.01752300e+00   # gluino decays
    1.55587600E-02     2    -2000003         3   # BR(~g -> ~sbar_R s)
    3.91391000E-02     2    -1000004         4   # BR(~g -> ~cbar_L c)
    1.74358200E-02     2    -2000004         4   # BR(~g -> ~cbar_R c)
";
        let err = Slha::parse(input).unwrap_err();
        if let Error(ErrorKind::InvalidDecay(pdg_id), _) = err {
            assert_eq!(pdg_id, 1000021);
        } else {
            panic!("Wrong error variant {:?} instead of InvalidDecay", err);
        }
    }

    #[test]
    fn test_invalid_numofdaughters() {
        // Pieces of the example file from appendix D.3 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
DECAY   1000021    1.01752300e+00   # gluino decays
    4.18313300E-02     2     1000001        -1   # BR(~g -> ~d_L dbar)
    1.55587600E-02     2     2000001        -1   # BR(~g -> ~d_R dbar)
    3.91391000E-02     2     1000002        -2   # BR(~g -> ~u_L ubar)
    1.74358200E-02     2     2000002        -2   # BR(~g -> ~u_R ubar)
    4.18313300E-02     2     1000003        -3   # BR(~g -> ~s_L sbar)
DECAY   1000022    1.01752300e+00   # gluino decays
    1.55587600E-02     2     2000003        -3   # BR(~g -> ~s_R sbar)
    3.91391000E-02     2     1000004        -4   # BR(~g -> ~c_L cbar)
    1.74358200E-02     2     2000004        -4   # BR(~g -> ~c_R cbar)
    1.13021900E-01     2     1000005        -5   # BR(~g -> ~b_1 bbar)
    6.30339800E-02     2     2000005        -5   # BR(~g -> ~b_2 bbar)
    9.60140900E-02     2     1000006        -6   # BR(~g -> ~t_1 tbar)
    0.00000000E+00     2     2000006        -6   # BR(~g -> ~t_2 tbar)
DECAY   1000025     1.01752300e+00  # gluino decays
    4.18313300E-02     2    -1000001         1   # BR(~g -> ~dbar_L d)
    1.55587600E-02     2    -2000001         1   # BR(~g -> ~dbar_R d)
    3.91391000E-02     2    -1000002         2   # BR(~g -> ~ubar_L u)
    1.74358200E-02     2    -2000002         2   # BR(~g -> ~ubar_R u)
    4.18313300E-02     2    -1000003         3   # BR(~g -> ~sbar_L s)
DECAY   1000020    1.01752300e+00   # gluino decays
    1.55587600E-02     two  -2000003         3   # BR(~g -> ~sbar_R s)
    3.91391000E-02     2    -1000004         4   # BR(~g -> ~cbar_L c)
    1.74358200E-02     2    -2000004         4   # BR(~g -> ~cbar_R c)
";
        let err = Slha::parse(input).unwrap_err();
        if let Error(ErrorKind::InvalidDecay(pdg_id), _) = err {
            assert_eq!(pdg_id, 1000020);
        } else {
            panic!("Wrong error variant {:?} instead of InvalidDecay", err);
        }
    }

    #[test]
    fn test_invalid_daughterid() {
        // Pieces of the example file from appendix D.3 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
DECAY   1000021    1.01752300e+00   # gluino decays
    4.18313300E-02     2     1000001        -1   # BR(~g -> ~d_L dbar)
    1.55587600E-02     2     2000001        -1   # BR(~g -> ~d_R dbar)
    3.91391000E-02     2     1000002        =2   # BR(~g -> ~u_L ubar)
    1.74358200E-02     2     2000002        -2   # BR(~g -> ~u_R ubar)
    4.18313300E-02     2     1000003        -3   # BR(~g -> ~s_L sbar)
DECAY   1000022    1.01752300e+00   # gluino decays
    1.55587600E-02     2     2000003        -3   # BR(~g -> ~s_R sbar)
    3.91391000E-02     2     1000004        -4   # BR(~g -> ~c_L cbar)
    1.74358200E-02     2     2000004        -4   # BR(~g -> ~c_R cbar)
    1.13021900E-01     2     1000005        -5   # BR(~g -> ~b_1 bbar)
    6.30339800E-02     2     2000005        -5   # BR(~g -> ~b_2 bbar)
    9.60140900E-02     2     1000006        -6   # BR(~g -> ~t_1 tbar)
    0.00000000E+00     2     2000006        -6   # BR(~g -> ~t_2 tbar)
DECAY   1000025     1.01752300e+00  # gluino decays
    4.18313300E-02     2    -1000001         1   # BR(~g -> ~dbar_L d)
    1.55587600E-02     2    -2000001         1   # BR(~g -> ~dbar_R d)
    3.91391000E-02     2    -1000002         2   # BR(~g -> ~ubar_L u)
    1.74358200E-02     2    -2000002         2   # BR(~g -> ~ubar_R u)
    4.18313300E-02     2    -1000003         3   # BR(~g -> ~sbar_L s)
DECAY   1000020    1.01752300e+00   # gluino decays
    1.55587600E-02     2    -2000003         3   # BR(~g -> ~sbar_R s)
    3.91391000E-02     2    -1000004         4   # BR(~g -> ~cbar_L c)
    1.74358200E-02     2    -2000004         4   # BR(~g -> ~cbar_R c)
";
        let err = Slha::parse(input).unwrap_err();
        if let Error(ErrorKind::InvalidDecay(pdg_id), _) = err {
            assert_eq!(pdg_id, 1000021);
        } else {
            panic!("Wrong error variant {:?} instead of InvalidDecay", err);
        }
    }

    #[test]
    fn test_missing_daughter() {
        // Pieces of the example file from appendix D.3 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
DECAY   1000021    1.01752300e+00   # gluino decays
    4.18313300E-02     2     1000001        -1   # BR(~g -> ~d_L dbar)
    1.55587600E-02     2     2000001        -1   # BR(~g -> ~d_R dbar)
    3.91391000E-02     2     1000002        -2   # BR(~g -> ~u_L ubar)
    1.74358200E-02     2     2000002        -2   # BR(~g -> ~u_R ubar)
    4.18313300E-02     2     1000003        -3   # BR(~g -> ~s_L sbar)
DECAY   1000022    1.01752300e+00   # gluino decays
    1.55587600E-02     2     2000003        -3   # BR(~g -> ~s_R sbar)
    3.91391000E-02     2     1000004        -4   # BR(~g -> ~c_L cbar)
    1.74358200E-02     2     2000004           # BR(~g -> ~c_R cbar)
    1.13021900E-01     2     1000005        -5   # BR(~g -> ~b_1 bbar)
    6.30339800E-02     2     2000005        -5   # BR(~g -> ~b_2 bbar)
    9.60140900E-02     2     1000006        -6   # BR(~g -> ~t_1 tbar)
    0.00000000E+00     2     2000006        -6   # BR(~g -> ~t_2 tbar)
DECAY   1000025     1.01752300e+00  # gluino decays
    4.18313300E-02     2    -1000001         1   # BR(~g -> ~dbar_L d)
    1.55587600E-02     2    -2000001         1   # BR(~g -> ~dbar_R d)
    3.91391000E-02     2    -1000002         2   # BR(~g -> ~ubar_L u)
    1.74358200E-02     2    -2000002         2   # BR(~g -> ~ubar_R u)
    4.18313300E-02     2    -1000003         3   # BR(~g -> ~sbar_L s)
DECAY   1000020    1.01752300e+00   # gluino decays
    1.55587600E-02     2    -2000003         3   # BR(~g -> ~sbar_R s)
    3.91391000E-02     2    -1000004         4   # BR(~g -> ~cbar_L c)
    1.74358200E-02     2    -2000004         4   # BR(~g -> ~cbar_R c)
";
        let err = Slha::parse(input).unwrap_err();
        if let Error(ErrorKind::InvalidDecay(pdg_id), _) = err {
            assert_eq!(pdg_id, 1000022);
        } else {
            panic!("Wrong error variant {:?} instead of InvalidDecay", err);
        }
    }

    #[test]
    fn test_too_many_daughters() {
        // Pieces of the example file from appendix D.3 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
DECAY   1000021    1.01752300e+00   # gluino decays
    4.18313300E-02     2     1000001        -1   # BR(~g -> ~d_L dbar)
    1.55587600E-02     2     2000001        -1   # BR(~g -> ~d_R dbar)
    3.91391000E-02     2     1000002        -2   # BR(~g -> ~u_L ubar)
    1.74358200E-02     2     2000002        -2   # BR(~g -> ~u_R ubar)
    4.18313300E-02     2     1000003        -3   # BR(~g -> ~s_L sbar)
DECAY   1000022    1.01752300e+00   # gluino decays
    1.55587600E-02     2     2000003        -3   # BR(~g -> ~s_R sbar)
    3.91391000E-02     2     1000004        -4   # BR(~g -> ~c_L cbar)
    1.74358200E-02     2     2000004        -4 9 # BR(~g -> ~c_R cbar)
    1.13021900E-01     2     1000005        -5   # BR(~g -> ~b_1 bbar)
    6.30339800E-02     2     2000005        -5   # BR(~g -> ~b_2 bbar)
    9.60140900E-02     2     1000006        -6   # BR(~g -> ~t_1 tbar)
    0.00000000E+00     2     2000006        -6   # BR(~g -> ~t_2 tbar)
DECAY   1000025     1.01752300e+00  # gluino decays
    4.18313300E-02     2    -1000001         1   # BR(~g -> ~dbar_L d)
    1.55587600E-02     2    -2000001         1   # BR(~g -> ~dbar_R d)
    3.91391000E-02     2    -1000002         2   # BR(~g -> ~ubar_L u)
    1.74358200E-02     2    -2000002         2   # BR(~g -> ~ubar_R u)
    4.18313300E-02     2    -1000003         3   # BR(~g -> ~sbar_L s)
DECAY   1000020    1.01752300e+00   # gluino decays
    1.55587600E-02     2    -2000003         3   # BR(~g -> ~sbar_R s)
    3.91391000E-02     2    -1000004         4   # BR(~g -> ~cbar_L c)
    1.74358200E-02     2    -2000004         4   # BR(~g -> ~cbar_R c)
";
        let err = Slha::parse(input).unwrap_err();
        if let Error(ErrorKind::InvalidDecay(pdg_id), _) = err {
            assert_eq!(pdg_id, 1000022);
        } else {
            panic!("Wrong error variant {:?} instead of InvalidDecay", err);
        }
    }

    #[test]
    fn test_parse_block_str_invalid_float() {
        let input = "\
BLOCK TEST
 1 3
 4 6
block Mass
  6  173.2
BloCk FooBar
  1 2 3 4 0x5
  1 assdf 3 4 8
  1 2 4 8.98
";
        let slha = Slha::parse(input).unwrap();
        println!("{:?}", slha);
        let block: BlockStr<i64> = slha.get_block("test").unwrap().unwrap();
        assert_eq!(block.map.len(), 2);
        assert_eq!(block.map[&vec!["1".to_string()]], 3);
        assert_eq!(block.map[&vec!["4".to_string()]], 6);
        let block: BlockStr<(i64, f64)> = slha.get_block("mass").unwrap().unwrap();
        assert_eq!(block.map.len(), 1);
        assert_eq!(block.map[&Vec::new()], (6, 173.2));
        let block: Result<BlockStr<f64>, Error> = slha.get_block("foobar").unwrap();
        let err = block.unwrap_err();
        if let Error(ErrorKind::InvalidBlock(name), _) = err {
            assert_eq!(name, "foobar");
        } else {
            panic!("Wrong error variant {:?} instead of InvalidBlock", err);
        }
    }

    #[test]
    fn test_parse_block_str_eol() {
        let input = "\
BLOCK TEST
 1 3
 4 6
block Mass
  6  173.2
BloCk FooBar
  1 2 3 4 0.5
  1 9 3 4 8
  1 2 4 8
";
        let slha = Slha::parse(input).unwrap();
        println!("{:?}", slha);
        let block: BlockStr<i64> = slha.get_block("test").unwrap().unwrap();
        assert_eq!(block.map.len(), 2);
        assert_eq!(block.map[&vec!["1".to_string()]], 3);
        assert_eq!(block.map[&vec!["4".to_string()]], 6);
        let block: BlockStr<(i64, f64)> = slha.get_block("mass").unwrap().unwrap();
        assert_eq!(block.map.len(), 1);
        assert_eq!(block.map[&Vec::new()], (6, 173.2));
        let block: Result<BlockStr<(i8, i8, i8, i8, f64)>, Error> = slha.get_block("foobar")
            .unwrap();
        let err = block.unwrap_err();
        if let Error(ErrorKind::InvalidBlock(name), _) = err {
            assert_eq!(name, "foobar");
        } else {
            panic!("Wrong error variant {:?} instead of InvalidBlock", err);
        }
    }

    #[test]
    fn test_parse_block_single_map() {
        let input = "\
BLOCK TEST
   3  9
  ";
        let slha = Slha::parse(input).unwrap();
        println!("{:?}", slha);
        let block: Result<BlockSingle<i64>, Error> = slha.get_block("test").unwrap();
        let err = block.unwrap_err();
        if let Error(ErrorKind::InvalidBlock(name), _) = err {
            assert_eq!(name, "test");
        } else {
            panic!("Wrong error variant {:?} instead of InvalidBlock", err);
        }
    }

    #[test]
    fn test_parse_block_single_empty() {
        let input = "\
BLOCK TEST
BLOCK Foo
   4 9
  ";
        let slha = Slha::parse(input).unwrap();
        println!("{:?}", slha);
        let block: Result<BlockSingle<i64>, Error> = slha.get_block("test").unwrap();
        let err = block.unwrap_err();
        if let Error(ErrorKind::InvalidBlock(name), _) = err {
            assert_eq!(name, "test");
        } else {
            panic!("Wrong error variant {:?} instead of InvalidBlock", err);
        }
    }

    #[test]
    fn test_parse_block_single_invalid() {
        let input = "\
BLOCK TEST
   59.7  ";
        let slha = Slha::parse(input).unwrap();
        println!("{:?}", slha);
        let block: Result<BlockSingle<i64>, Error> = slha.get_block("test").unwrap();
        let err = block.unwrap_err();
        if let Error(ErrorKind::InvalidBlock(name), _) = err {
            assert_eq!(name, "test");
        } else {
            panic!("Wrong error variant {:?} instead of InvalidBlock", err);
        }
    }

    #[test]
    fn test_parse_blocks_invalid_key() {
        let input = "\
BLOCK TEST
 1 3
 4 6
block foobar Q= 1
  6  173.2
block Mass Q= 3
  6  173.2
BloCk FooBar Q = 8
  1 2 0.5
  1 assdf 8
  1 2 8.98
";
        let slha = Slha::parse(input).unwrap();
        println!("{:?}", slha);
        let block: Result<Vec<Block<(i8, i8), f64>>, Error> = slha.get_blocks("foobar");
        let err = block.unwrap_err();
        if let Error(ErrorKind::InvalidBlock(name), _) = err {
            assert_eq!(name, "foobar");
        } else {
            panic!("Wrong error variant {:?} instead of InvalidBlock", err);
        }
    }

    #[test]
    fn test_parse_blocks_incomplete_parse() {
        let input = "\
BLOCK TEST
 1 3
 4 6
block foobar Q= 1
  6  173.2
block Mass Q= 3
  6  7 173.2
BloCk FooBar Q = 8
  1 2 0.5
  1 97 8
  1 2 8.98
";
        let slha = Slha::parse(input).unwrap();
        println!("{:?}", slha);
        let block: Result<Vec<Block<i8, f64>>, Error> = slha.get_blocks("mass");
        let err = block.unwrap_err();
        if let Error(ErrorKind::InvalidBlock(name), _) = err {
            assert_eq!(name, "mass");
        } else {
            panic!("Wrong error variant {:?} instead of InvalidBlock", err);
        }
    }

    #[test]
    fn test_parse_blocks_short_key() {
        let input = "\
BLOCK TEST
 1 3
 4 6
block foobar Q= 1
  6  173.2
block Mass Q= 3
  6  173.2
BloCk FooBar Q = 8
  1 2 0.5
  1
  1 2 8.98
";
        let slha = Slha::parse(input).unwrap();
        println!("{:?}", slha);
        let block: Result<Vec<Block<(i8, i8), f64>>, Error> = slha.get_blocks("foobar");
        let err = block.unwrap_err();
        if let Error(ErrorKind::InvalidBlock(name), _) = err {
            assert_eq!(name, "foobar");
        } else {
            panic!("Wrong error variant {:?} instead of InvalidBlock", err);
        }
    }

    #[test]
    fn test_parse_blocks_incompatible_blocks() {
        let input = "\
BLOCK TEST Q= 32
 1 3
 4 6
block foobar Q= 1
  6 9.2
block Mass Q= 3
  6  173.2
BloCk Test Q = 8
  1 2 0.5
  1 21 0.23
  1 2 8.98
";
        let slha = Slha::parse(input).unwrap();
        println!("{:?}", slha);
        let block: Result<Vec<Block<i8, i8>>, Error> = slha.get_blocks("test");
        let err = block.unwrap_err();
        if let Error(ErrorKind::InvalidBlock(name), _) = err {
            assert_eq!(name, "test");
        } else {
            panic!("Wrong error variant {:?} instead of InvalidBlock", err);
        }
    }
}
