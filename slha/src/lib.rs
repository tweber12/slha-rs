// Copyright 2017 Torsten Weber
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A crate to read [SUSY Les Houches Accord] (or SLHA) files.
//!
//! This crate aims to be as general as possible, in the sense that Blocks of arbitrary types are
//! supported.
//! In particular, all the Blocks defined in the SLHA [1] and [2] standards can be read using this
//! crate.
//!
//! There are two ways to use this crate, using the (automatically derivable) [`SlhaDeserialize`]
//! trait or using an [`Slha`] object.
//! Automatically deriving `SlhaDeserialize` is the recommended way to use this library, and the
//! `Slha` object approach should only be used if the other can not be used, most notably in the
//! case when the necessary Blocks are only known at runtime.
//!
//! # Usage example
//!
//! The following is a full example using the `SlhaDeserialize` method to extract blocks and decays
//! from an SLHA file. An example using the `Slha` object method can be found in the
//! [`Using an Slha object`] section.
//! section.
//! To be able to use this example, you will have to add
//!
//! ```toml
//! ...
//!
//! #[dependencies]
//! slha = "0.5"
//! slha-derive = "0.5"
//! error-chain = "0.1"
//! ```
//!
//! to your `Cargo.toml`.
//!
//! ```rust
//! extern crate slha;
//! #[macro_use]
//! extern crate slha_derive;
//! extern crate error_chain;
//!
//! use std::collections::HashMap;
//! use slha::{Block, DecayTable, SlhaDeserialize};
//! use error_chain::ChainedError;
//!
//! #[derive(Debug, SlhaDeserialize)]
//! struct Slha {
//!     mass: Block<i64, f64>,
//!     ye: Vec<Block<(i8, i8), f64>>,
//!     decays: HashMap<i64, DecayTable>,
//! }
//!
//! fn main() {
//!     // In a more realistic example, this would be read from a file somewhere.
//!     let input = "
//! BLOCK MASS
//!    6    173.2    # M_t
//! BLOCK ye Q= 1
//!    3   3    4.2
//! BLOCK ye Q= 2
//!    3   3    8.4
//! Decay 6 1.35
//!    1  2   5  24  # t > W+ b
//! ";
//!
//!     let slha = match Slha::deserialize(input) {
//!         Ok(slha) => slha,
//!         Err(err) => {
//!             eprintln!("{}", err.display_chain());
//!             return;
//!         },
//!     };
//!
//!     assert_eq!(slha.mass.map[&6], 173.2);
//!     assert_eq!(slha.mass.scale, None);
//!     assert_eq!(slha.ye.len(), 2);
//!     assert_eq!(slha.ye[0].scale, Some(1.));
//!     assert_eq!(slha.ye[1].map[&(3,3)], 8.4);
//!     assert_eq!(slha.decays[&6].width, 1.35);
//! }
//! ```
//!
//! # Using derive
//!
//! Together with the `slha-derive` crate, it is possible to deserialize a SLHA file directly into a rust
//! struct.
//!
//! ## Blocks
//!
//! Each field of the struct is treated as a block in the SLHA file with the same
//! (case-insensitive) name as the field. The block-name that should be deserialized into a field
//! can be customized using the `rename` attribute.
//!
//! While the fields can be of any type that implements the [`SlhaBlock`] trait, the most common
//! blocks, including all blocks defined in the SLHA 1 and 2 papers, can be expressed using two
//! block types defined in this crate, [`Block`] and [`BlockSingle`].
//!
//! All blocks declared in the struct must be present in the SLHA file, or an error is
//! returned.
//! Blocks that are included in the SLHA file but not in the struct are ignored.
//! Therefore it is possible to pick and choose the blocks that are necessary for a task without
//! having to include (and know the types of) all the others.
//!
//! ```rust
//! # extern crate slha;
//! # #[macro_use]
//! # extern crate slha_derive;
//! #
//! # use slha::{SlhaDeserialize, Block, BlockSingle};
//! #
//! #[derive(Debug, SlhaDeserialize)]
//! struct Slha {
//!     alpha: BlockSingle<f64>,
//!     mass: Block<i64, f64>,
//!     ye: Block<(u8, u8), f64>,
//! }
//! #
//! # fn main() {
//! let input = "
//! BLOCK MASS
//!    6    173.2    # M_t
//! BLOCK ye Q= 1
//!    3   3    4.2
//! BLOCK ALPHA   # Effective Higgs mixing parameter
//!      -1.1e-01   # alpha
//! ";
//!
//! let slha = Slha::deserialize(input).unwrap();
//! let mass = slha.mass;
//! assert_eq!(mass.scale, None);
//! assert_eq!(mass.map.len(), 1);
//! assert_eq!(mass.map[&6], 173.2);
//!
//! let ye = slha.ye;
//! assert_eq!(ye.scale, Some(1.));
//! assert_eq!(ye.map.len(), 1);
//! assert_eq!(ye.map[&(3, 3)], 4.2);
//!
//! let alpha = slha.alpha;
//! assert_eq!(alpha.scale, None);
//! assert_eq!(alpha.value, -1.1e-1);
//! # }
//! ```
//!
//! ### Optional blocks
//!
//! The default behaviour is to return an error if a block declared in the struct is not present in
//! the SLHA file.
//! Since this is not always desireable, it is possible to mark blocks as optional by wrapping the
//! type of the block in an `Option`.
//!
//! ```rust
//! # extern crate slha;
//! # #[macro_use]
//! # extern crate slha_derive;
//! #
//! # use slha::{Block, SlhaDeserialize};
//! #
//! #[derive(Debug, SlhaDeserialize)]
//! struct Slha {
//!     mass: Option<Block<i64, f64>>,
//! }
//! #
//! # fn main() {
//! let present = "
//! BLOCK MASS
//!    6    173.2    # M_t
//! ";
//!
//! let present = Slha::deserialize(present).unwrap();
//! assert!(present.mass.is_some());
//! let not_present = Slha::deserialize("").unwrap();
//! assert!(not_present.mass.is_none());
//! # }
//! ```
//!
//!
//! ### Repeated blocks
//!
//! How duplicate blocks are handled depends on the type of the field that the block is written
//! into.
//!
//! If the type is a Block, any block that appears more than once, even with different scale is an
//! error.
//! However, the SLHA standard allows for blocks (with different scales) to appear multiple times in an SLHA
//! file. To store all occurences of a block, the block can be wrapped in a `Vec`. An error is
//! returned if the blocks do not have different scales.
//!
//! ```rust
//! # extern crate slha;
//! # #[macro_use]
//! # extern crate slha_derive;
//! # extern crate error_chain;
//! #
//! # use std::collections::HashMap;
//! # use slha::{Block, SlhaDeserialize};
//! #
//! #[derive(Debug, SlhaDeserialize)]
//! struct Slha {
//!     ye: Vec<Block<(i8, i8), f64>>,
//! }
//! #
//! # fn main() {
//! let input = "
//! BLOCK ye Q= 1
//!    3   3    4.2
//! BLOCK ye Q= 2
//!    3   3    8.4
//! ";
//!
//! let slha = Slha::deserialize(input).unwrap();
//! assert_eq!(slha.ye.len(), 2);
//! assert_eq!(slha.ye[0].scale, Some(1.));
//! assert_eq!(slha.ye[0].map[&(3,3)], 4.2);
//! assert_eq!(slha.ye[1].scale, Some(2.));
//! assert_eq!(slha.ye[1].map[&(3,3)], 8.4);
//! # }
//! ```
//!
//! The `modifier` module contains additional types that can be used to obtain different behaviour
//! when duplicate blocks are encountered.
//!
//! * The `VecUnchecked` type works like a `Vec`, except that no sanity checks are performed at all.
//!   As such this type can be used to really collect all occurences of a block.
//! * The `TakeFirst` and `TakeLast` wrappers keep the first and last occurence of a block
//!   respectively.
//!
//!
//! Taking the first...
//!
//! ```rust
//! # extern crate slha;
//! # #[macro_use]
//! # extern crate slha_derive;
//! #
//! # use slha::{SlhaDeserialize, Block};
//! # use slha::modifier::TakeFirst;
//! #
//! # fn main() {
//! let input = "\
//! Block MODsel Q= 10 # Select model
//!      3     10     # tanb
//! Block MODsel  # Select model
//!      4      1     # sign(mu)
//! Block MODsel Q= 20 # Select model
//!      1    100     # m0
//! Block MODsel Q= 10 # Select model
//!      2    250     # m12
//! Block MODsel  # Select model
//!      5   -100     # A0 ";
//!
//! #[derive(Debug, SlhaDeserialize)]
//! struct MySlhaFirst {
//!     modsel: TakeFirst<Block<i8, i64>>,
//! }
//!
//! let first = MySlhaFirst::deserialize(input).unwrap();
//! let modsel = first.modsel;
//! assert_eq!(modsel.scale, Some(10.));
//! assert_eq!(modsel.map[&3], 10);
//! # }
//! ```
//!
//! ...or the last...
//!
//! ```rust
//! # extern crate slha;
//! # #[macro_use]
//! # extern crate slha_derive;
//! #
//! # use slha::{SlhaDeserialize, Block};
//! # use slha::modifier::TakeLast;
//! #
//! # fn main() {
//! # let input = "\
//! # Block MODsel Q= 10 # Select model
//! #      3     10     # tanb
//! # Block MODsel  # Select model
//! #      4      1     # sign(mu)
//! # Block MODsel Q= 20 # Select model
//! #      1    100     # m0
//! # Block MODsel Q= 10 # Select model
//! #      2    250     # m12
//! # Block MODsel  # Select model
//! #      5   -100     # A0 ";
//! #
//! #[derive(Debug, SlhaDeserialize)]
//! struct MySlhaLast {
//!     modsel: TakeLast<Block<i8, i64>>,
//! }
//!
//! let last = MySlhaLast::deserialize(input).unwrap();
//! let modsel = last.modsel;
//! assert_eq!(modsel.scale, None);
//! assert_eq!(modsel.map[&5], -100);
//! # }
//! ```
//!
//! ...or all blocks can easily be achieved using these modifiers.
//!
//! ```rust
//! # extern crate slha;
//! # #[macro_use]
//! # extern crate slha_derive;
//! #
//! # use slha::{SlhaDeserialize, Block};
//! # use slha::modifier::VecUnchecked;
//! #
//! # fn main() {
//! #    let input = "\
//! # Block MODsel Q= 10 # Select model
//! #      3     10     # tanb
//! # Block MODsel  # Select model
//! #      4      1     # sign(mu)
//! # Block MODsel Q= 20 # Select model
//! #      1    100     # m0
//! # Block MODsel Q= 10 # Select model
//! #      2    250     # m12
//! # Block MODsel  # Select model
//! #      5   -100     # A0 ";
//! #
//! #[derive(Debug, SlhaDeserialize)]
//! struct MySlhaAll {
//!     modsel: VecUnchecked<Block<i8, i64>>,
//! }
//!
//! let all = MySlhaAll::deserialize(input).unwrap();
//! let modsel = all.modsel;
//! assert_eq!(modsel.len(), 5);
//! assert_eq!(modsel[0].map[&3], 10);
//! assert_eq!(modsel[2].scale, Some(20.));
//! assert_eq!(modsel[3].map[&2], 250);
//! # }
//! ```
//!
//! ## Decays
//!
//! Decays can be read in as well.
//! For this a field with name `decays` and type `HashMap<i64, [`DecayTable`]>` has to be present in
//! the struct.
//! The `decays` field then contains a map from the pdg id of the decaying particle to the
//! `DecayTable` of the particle.
//! An error is returned if there are multiple decay tables for the same particle.
//!
//! ```rust
//! # extern crate slha;
//! # #[macro_use]
//! # extern crate slha_derive;
//! #
//! # use std::collections::HashMap;
//! # use slha::{DecayTable, SlhaDeserialize};
//! #
//! #[derive(Debug, SlhaDeserialize)]
//! struct Slha {
//!     decays: HashMap<i64, DecayTable>,
//! }
//! #
//! # fn main() {
//! let input = "
//! Decay 6 1.35
//!    1  2   5  24  # t > W+ b
//! ";
//!
//! let slha = Slha::deserialize(input).unwrap();
//! assert_eq!(slha.decays[&6].width, 1.35);
//! let decays = &slha.decays[&6].decays;
//! assert_eq!(decays.len(), 1);
//! assert_eq!(decays[0].branching_ratio, 1.);
//! assert_eq!(decays[0].daughters, vec![5, 24]);
//! # }
//! ```
//!
//!
//! # Using an `Slha` object
//!
//! Sometimes using the [`SlhaDeserialize`] trait is not an option, usually if the names and/or
//! types of the blocks in the SLHA file are not known at compile time.
//! In these cases blocks from the SLHA file can still be accessed using an [`Slha`] object, which
//! allows to access blocks by name with names that are only known at run time.
//!
//!
//! ## Accessing blocks
//!
//! ### Blocks with (partially) known type
//!
//! Since neither the names nor the types of the blocks contained in the SLHA file are known in
//! advance, blocks can only be converted into their corresponding rust type when the block is
//! accessed. Therefore all getter functions for blocks can return parse errors if the block is
//! invalid or cannot be converted into the desired type.
//!
//! The SLHA object provides several methods that can be used to convert blocks into any type that
//! implements the [`SlhaBlock`] trait.
//! If the full type is known, the [`get_block`] function can be used to convert the block into
//! either a [`Block`] or a [`BlockSingle`] object.
//! If only the value type is known but not the key type, then these blocks can be converted into a
//! `BlockStr`, which uses a vector of 'words' as key.
//!
//! ### Blocks with unknown type
//!
//! As a last resort, the [`get_raw_blocks`] method can be used to obtain a [`RawBlock`] object for
//! each block with a given name.
//! This object contains the raw (string) data that makes up the block.
//!
//! ### Repeated blocks
//!
//! The [`Slha`] objects has several methods to access blocks that may appear more than once.
//!
//! * The [`get_block`] method is supposed to be used for blocks that may only appear once in an SLHA
//!   file, or in cases where the calling code is not prepared to handle more than one block.
//!   If a block does appear multiple times, then this method will return an error when trying to
//!   access it.
//! * The [`get_blocks`] method. This method returns all occurences of a block in an SLHA file, as
//!   long as these blocks have different scales.
//!   As such, this method can be used to access e.g. grids for running parameters.
//! * The [`get_blocks_unchecked`] method is similar to the `get_blocks` method, but does not perform
//!   any sanity checks.
//!   Using this function it is for example possible to read duplicate blocks without scale.
//!
//! ### Examples
//!
//! ```rust
//! use slha::{Slha, Block, BlockStr};
//!
//! let input = "\
//! Block SMINPUTS   # Standard Model inputs
//!      3      0.1172  # alpha_s(MZ) SM MSbar
//!      5      4.25    # Mb(mb) SM MSbar
//!      6    174.3     # Mtop(pole)
//! DECAY 6 1.35
//!    1   2   5   24
//! ";
//!
//! let slha = match Slha::parse(input) {
//!     Ok(slha) => slha,
//!     Err(err) => panic!("Failed to deserialize SLHA file: {}", err),
//! };
//!
//! // Using `Block`
//! let sminputs: Block<i8, f64> = match slha.get_block("sminputs") {
//!     Some(block) => match block {
//!         Ok(sminputs) => sminputs,
//!         Err(err) => panic!("Failed to parse block 'smimputs':\n{}", err),
//!     },
//!     None => panic!("Missing block 'sminputs'."),
//! };
//! assert_eq!(sminputs.scale, None);
//! assert_eq!(sminputs.map.len(), 3);
//! assert_eq!(sminputs.map[&5], 4.25);
//!
//! // Using `BlockStr`
//! let sminputs: BlockStr<f64> = match slha.get_block("sminputs") {
//!     Some(block) => match block {
//!         Ok(sminputs) => sminputs,
//!         Err(err) => panic!("Failed to parse block 'smimputs':\n{}", err),
//!     },
//!     None => panic!("Missing block 'sminputs'."),
//! };
//! assert_eq!(sminputs.scale, None);
//! assert_eq!(sminputs.map.len(), 3);
//! assert_eq!(sminputs.map[&vec!["5".to_string()]], 4.25);
//!
//! // Using `RawBlock`
//! let blocks = slha.get_raw_blocks("sminputs");
//! assert_eq!(blocks.len(), 1);
//! let sminputs = &blocks[0];
//! assert_eq!(sminputs.scale, None);
//! assert_eq!(sminputs.lines.len(), 3);
//! assert_eq!(sminputs.lines[1].data, "5      4.25    ");
//! assert_eq!(sminputs.lines[1].comment, Some("# Mb(mb) SM MSbar"));
//! ```
//!
//! ## Accessing decays
//!
//! The decay tables contained in the SLHA file can be accessed using the [`get_decay`] method.
//!
//! ### Examples
//!
//! ```rust
//! use slha::{Slha, DecayTable};
//!
//! let input = "\
//! Block SMINPUTS   # Standard Model inputs
//!      3      0.1172  # alpha_s(MZ) SM MSbar
//!      5      4.25    # Mb(mb) SM MSbar
//!      6    174.3     # Mtop(pole)
//! DECAY 6 1.35
//!     1   2   5   24
//! ";
//!
//! let slha = match Slha::parse(input) {
//!     Ok(slha) => slha,
//!     Err(err) => panic!("Failed to deserialize SLHA file: {}", err),
//! };
//!
//! let decay_table = match slha.get_decay(6) {
//!     Some(dec) => dec,
//!     None => panic!("Missing decay table for the top quark."),
//! };
//! assert_eq!(decay_table.width, 1.35);
//! let decay = &decay_table.decays;
//! assert_eq!(decay.len(), 1);
//! assert_eq!(decay[0].branching_ratio, 1.);
//! assert_eq!(decay[0].daughters, vec![5, 24]);
//! ```
//!
//! [SUSY Les Houches Accord]: https://arxiv.org/abs/hep-ph/0311123
//! [1]: https://arxiv.org/abs/hep-ph/0311123
//! [2]: https://arxiv.org/abs/0801.0045
//! [`SlhaDeserialize`]: trait.SlhaDeserialize.html
//! [`Slha`]: struct.Slha.html
//! [`Using an Slha object`]: index.html#using-an-slha-object
//! [`SlhaBlock`]: trait.SlhaBlock.html
//! [`Block`]: struct.Block.html
//! [`BlockSingle`]: struct.BlockSingle.html
//! [`BlockStr`]: struct.BlockStr.html
//! [`RawBlock`]: struct.RawBlock.html
//! [`DecayTable`]: struct.DecayTable.html
//! [`get_block`]: struct.Slha.html#method.get_decay
//! [`get_blocks`]: struct.Slha.html#method.get_decay
//! [`get_blocks_unchecked`]: struct.Slha.html#method.get_decay
//! [`get_raw_blocks`]: struct.Slha.html#method.get_decay
//! [`get_decay`]: struct.Slha.html#method.get_decay

#![recursion_limit = "256"]

#[macro_use]
extern crate error_chain;

use std::collections::HashMap;
use std::hash::Hash;
use std::str;

pub mod internal;
pub mod modifier;

use internal::{parse_block_body, Segment};

pub mod errors {
    //! Errors that may occur when parsing an SLHA file into rust types.
    use std::num::{ParseFloatError, ParseIntError};

    error_chain!{
        errors {
            /// A block without a name was found in the SLHA file.
            MissingBlockName {
                description("Missing block name")
            }
            /// A BLOCK in the SLHA file could not be parsed into a `Block`.
            ///
            /// The field of this variant is the name of the problematic block.
            InvalidBlock(name: String) {
                description("Malformed block")
                display("Malformed block: '{}'", name)
            }
            /// A BLOCK in the SLHA file could not be parsed into a `BlockSingle`.
            ///
            /// The field of this variant is the name of the problematic block.
            InvalidBlockSingle(name: String) {
                description("Malformed block single")
                display("Malformed block single: '{}'", name)
            }
            /// The pdg id of a DECAY segment could not be read.
            InvalidDecayingPdgId {
                description("Failed to parse the pdg id of the decaying particle")
            }
            /// A parse error occured while parsing a DECAY segment.
            ///
            /// The field of this variant is the pdg id of the problematic DECAY.
            InvalidDecay(pdg_id: i64) {
                description("Invalid decay table")
                display("Invalid decay table for particle {}", pdg_id)
            }
            /// A (sub)parser did not consume a whole line.
            ///
            /// This error is returned if e.g. a line from a block is parsed and there is
            /// unexpected input at the end of the line.
            IncompleteParse(rest: Vec<String>) {
                description("The parser did not consume all words of the input.")
                display("The parser did not consume the whole line, '{:?}' was left over", rest)
            }
            /// There was not enough data to parse a line in the SLHA file into the desired rust
            /// type.
            UnexpectedEol {
                description("The parser reached the end of the line before finishing")
            }
            /// An integer type was expected but could not be read from the file.
            InvalidInt(err: ParseIntError) {
                description("Failed to parse an integer")
                display("Failed to parse an integer: {}", err)
            }
            /// An floating point type was expected but could not be read from the file.
            InvalidFloat(err: ParseFloatError) {
                description("Failed to parse a floating point number")
                display("Failed to parse a floating point number: {}", err)
            }
            /// A top level segment other than "BLOCK" or "DECAY" was encountered.
            UnknownSegment(segment: String) {
                description("Unknown top level segment encountered")
                display("Unknown top level segment encountered: '{}'", segment)
            }
            /// The beginning of a new segment was expected, but a (idented) data line was found.
            UnexpectedIdent(line: String) {
                description("Expected the beginning of a segment, found an indented line instead")
                display("Expected the beginning of a segment, found an indented line instead: '{}'", line)
            }
            MalformedBlockHeader(rest: String) {
                description("Encountered trailing non-whitespace characters after block header")
                display("Encountered trailing non-whitespace characters after block header: '{}'", rest)
            }
            /// A line in the body of a block could not be parsed into the desired type.
            ///
            /// The field in this variant is the number of the _data_ line _in the block_ where the error
            /// occured.
            InvalidBlockLine(n: usize) {
                description("Failed to parse a line in the body")
                display("Failed to parse the {}th data line in the body", n)
            }
            /// The key part of a block could not be read.
            InvalidBlockKey {
                description("Failed to parse the key of a block")
            }
            /// The value part of a block could not be read.
            InvalidBlockValue {
                description("Failed to parse the value of a block")
            }
            /// A map-like block has a key appear more than once.
            DuplicateKey(line: usize) {
                description("There was a duplicate key in a block")
                display("The key in line {} appears more than once in the block", line)
            }
            /// A block (without scale) appeared more than once in the SLHA file.
            ///
            /// The field contains the name of the block.
            DuplicateBlock(name: String) {
                description("Found a duplicate block")
                display("Found a duplicate block: '{}'", name)
            }
            /// A block with scale appeared more than once in the SLHA file with the same scale.
            ///
            /// The two fields contain the name of the block and the scale.
            DuplicateBlockScale(name: String, scale: f64) {
                description("Found a duplicate block with equal scale")
                display("Found a duplicate block with name '{}' and scale '{}'", name, scale)
            }
            /// A block appears in the SLHA file both with and without scale.
            ///
            /// The field contains the name of the block.
            RedefinedBlockWithQ(name: String) {
                description("Found a duplicate block with and without scale")
                display("Found a duplicate block with and without scale: '{}'", name)
            }
            /// The scale of a block could not be read.
            ///
            /// The field contains the name of the block.
            InvalidScale {
                description("Failed to parse the scale")
            }
            /// The SLHA file contains to DECAY tables for the same particle.
            ///
            /// The field contains the pdg id of the particle.
            DuplicateDecay(pdg_id: i64) {
                description("Found multiple decay tables for the same particle")
                display("Found multiple decay tables for the same particle: '{}'", pdg_id)
            }
            /// A data line from a DECAY table could not be read.
            ///
            /// The field contains the number of the _data_ line _in the DECAY table_.
            InvalidDecayLine(n: usize) {
                description("Failed to parse a line in the body")
                display("Failed to parse the {}th data line in the body", n)
            }
            /// The width in a DECAY table could not be read.
            InvalidWidth {
                description("Failed to parse the width")
            }
            /// The branching ratio in a DECAY could not be read.
            InvalidBranchingRatio {
                description("Failed to parse the branching ratio")
            }
            /// The number of daughters in a DECAY could not be read.
            InvalidNumOfDaughters {
                description("Failed to parse the number of daughter particles")
            }
            /// There where less daughters in a decay than declared.
            ///
            /// The two field gives the number of expected and found daughters in that order.
            NotEnoughDaughters(expected: u8, found: u8) {
                description("Did not find enough daughter particles")
                display("Did not find enough daughter particles, expected {} but found {}", expected, found)
            }
            /// The pdg id of a daughter particle in a decay could not be read.
            InvalidDaughterId {
                description("Failed to parse the pdg id of a daughter particle")
            }
            /// A block read into a `BlockSingle` contains more than one data line.
            ///
            /// The field gives the number of data lines found.
            WrongNumberOfValues(n: usize) {
                description("Found too many values in a single valued block")
                display("Found {} values in a single valued block", n)
            }
            /// A required block was not included in the SLHA file.
            ///
            /// The field gives the name of the block.
            MissingBlock(name: String) {
                description("A block is missing")
                display("Did not find the block with name '{}'", name)
            }
        }
    }
}

use errors::*;

/// A trait for structs that can be deserialized from an SLHA file.
///
/// This trait should not be derived manually, instead, if possible, it should be automatically
/// derived using the `slha-derive` crate.
pub trait SlhaDeserialize: Sized {
    /// Deserialize a SLHA file into a rust struct.
    ///
    /// # Errors
    ///
    /// If the deserialization fails an `Error` should be returned.
    fn deserialize(&str) -> Result<Self>;
}

/// A trait for types that can be created from a block in an SLHA file.
///
/// This trait is used by the custom derive macro and by the `get_block(s)` method of the `Slha`
/// struct to convert the body of a block read from the file into a rust type.
/// Therefore this trait must be implemented for any type that you want to directly read from an
/// SLHA block.
pub trait SlhaBlock: Sized {
    /// Parses the block from an SLHA file.
    ///
    /// The first argument of the `parse` function are all the data lines that belong
    /// to the block.
    /// All empty lines or lines containing only comments are not included.
    /// The second argument is the scale at which the contents of the block have been evaluated for
    /// running parameters.
    /// If no scale was given in the block header, this is `None`.
    ///
    /// # Errors
    ///
    /// An error should be returned if the body of the block can not be parsed into an object of
    /// of the implementing type.
    fn parse<'a>(&[Line<'a>], scale: Option<f64>) -> Result<Self>;

    /// Returns the scale at which the block contents are defined, if any.
    fn scale(&self) -> Option<f64>;
}

pub trait ParseableWord: Sized {
    /// Parse a value from the input string.
    ///
    /// The difference to `std::str::parse` is that partially consuming the input is allowed.
    /// The remaining input is included in the returned `ParseResult` and therefore still available
    /// for chaining parsers.
    fn parse_word<'input>(&'input str) -> Result<Self>;
}

impl ParseableWord for String {
    fn parse_word(input: &str) -> Result<String> {
        Ok(input.to_string())
    }
}

macro_rules! impl_parseable_word {
    ($int:ty, $err:ident) => {
        impl ParseableWord for $int {
            fn parse_word<'input>(input: &'input str) -> Result<$int> {
                input.parse().map_err(|err| ErrorKind::$err(err).into())
            }
        }
    }
}
impl_parseable_word!(i8, InvalidInt);
impl_parseable_word!(i16, InvalidInt);
impl_parseable_word!(i32, InvalidInt);
impl_parseable_word!(i64, InvalidInt);
impl_parseable_word!(u8, InvalidInt);
impl_parseable_word!(u16, InvalidInt);
impl_parseable_word!(u32, InvalidInt);
impl_parseable_word!(u64, InvalidInt);
impl_parseable_word!(f32, InvalidFloat);
impl_parseable_word!(f64, InvalidFloat);

pub trait Parseable: Sized {
    /// The lenght of the value to read, in words.
    const LENGTH: Option<u8>;
    /// Parse a value from the input string.
    ///
    /// The difference to `std::str::parse` is that partially consuming the input is allowed.
    /// The remaining input is included in the returned `ParseResult` and therefore still available
    /// for chaining parsers.
    fn parse<'input, I>(&mut I) -> Result<Self>
    where
        I: Iterator<Item = &'input str>;

    fn parse_str<'input>(input: &'input str) -> Result<Self> {
        Self::parse_all(&mut input.split_whitespace())
    }

    fn parse_all<'input, I>(input: &mut I) -> Result<Self>
    where
        I: Iterator<Item = &'input str>
    {
        let value = Self::parse(input)?;
        let rest: Vec<_> = input.map(|x| x.to_string()).collect();
        if !rest.is_empty() {
            bail!(ErrorKind::IncompleteParse(rest));
        }
        Ok(value)
    }
}
impl<T> Parseable for T
where
    T: ParseableWord,
{
    const LENGTH: Option<u8> = Some(1);
    fn parse<'input, I>(input: &mut I) -> Result<Self>
    where
        I: Iterator<Item = &'input str>,
    {
        let word = match input.next() {
            Some(word) => word,
            None => bail!(ErrorKind::UnexpectedEol),
        };
        T::parse_word(word)
    }
}
impl<T> Parseable for Vec<T>
where
    T: ParseableWord,
{
    const LENGTH: Option<u8> = None;
    fn parse<'input, I>(input: &mut I) -> Result<Self>
    where
        I: Iterator<Item = &'input str>,
    {
        input.map(T::parse_word).collect()
    }
}

macro_rules! impl_parseable_tuple {
    ($($name:ident),+; $n:expr) => {
        #[allow(non_snake_case)]
        #[allow(unused_assignments)]
        impl<$($name),*> Parseable for ($($name),*)
        where
            $($name: ParseableWord),*
        {
            const LENGTH: Option<u8> = Some($n);
            fn parse<'input, I>(input: &mut I) -> Result<($($name),*)>
            where
                I: Iterator<Item = &'input str>,
            {
                $(
                    let $name = match input.next() {
                        Some(word) => $name::parse_word(word)?,
                        None => bail!(ErrorKind::UnexpectedEol),
                    };
                )*
                Ok(($($name),*))
            }
        }
    }
}
impl_parseable_tuple!(K1, K2; 2);
impl_parseable_tuple!(K1, K2, K3; 3);
impl_parseable_tuple!(K1, K2, K3, K4; 4);
impl_parseable_tuple!(K1, K2, K3, K4, K5; 5);
impl_parseable_tuple!(K1, K2, K3, K4, K5, K6; 6);
impl_parseable_tuple!(K1, K2, K3, K4, K5, K6, K7; 7);
impl_parseable_tuple!(K1, K2, K3, K4, K5, K6, K7, K8; 8);
impl_parseable_tuple!(K1, K2, K3, K4, K5, K6, K7, K8, K9; 9);
impl_parseable_tuple!(K1, K2, K3, K4, K5, K6, K7, K8, K9, K10; 10);
impl_parseable_tuple!(K1, K2, K3, K4, K5, K6, K7, K8, K9, K10, K11; 11);
impl_parseable_tuple!(K1, K2, K3, K4, K5, K6, K7, K8, K9, K10, K11, K12; 12);

/// A block from an SLHA file treated as a map.
///
/// Most blocks from SLHA files represent blocks from some key(s) to a value.
/// In fact, only one block defined in the SLHA 1 standard does not belong to this category.
/// These 'map-type' blocks can be represented (and parsed) by the `Block` type.
///
/// The `Block` type can represent maps from any key type to any value type, as long as both key
/// and value implement the `Parseable` trait.
/// Implementations of this trait for all numeric types as well as for Strings and tuples are
/// included in this crate, which is enough to cover all blocks defined in the SLHA 1 and 2 papers.
/// There is however one restriction when using Strings. The parseable impl of String takes the
/// whole line, which means that String can not be used as a key.
///
/// Duplicate keys in a block are treated as a parse error.
///
/// # Reading blocks
///
/// `Block` implements the `SlhaBlock` trait and therefore can be read from an SLHA file.
/// This can be done in two different ways, the recommended way using `slha-derive` or using an
/// `Slha` object.
///
/// ## Using derive
///
/// The easiest way to read an SLHA file is to automatically derive the `SlhaDerive` trait on  a
/// struct.
///
/// ```rust
/// # extern crate slha;
/// # #[macro_use]
/// # extern crate slha_derive;
/// #
/// # use slha::{Block, SlhaDeserialize};
/// #
/// #[derive(Debug, SlhaDeserialize)]
/// struct Slha {
///     mass: Block<i64, f64>,
/// }
/// #
/// # fn main() {
/// let input = "
/// BLOCK MASS
///    6    173.2    # M_t
/// ";
///
/// let slha = Slha::deserialize(input).unwrap();
/// let mass = slha.mass;
/// assert_eq!(mass.scale, None);
/// assert_eq!(mass.map.len(), 1);
/// assert_eq!(mass.map[&6], 173.2);
/// # }
/// ```
///
///
/// ## Using an `Slha` object
///
/// Using an `Slha` object to parse an SLHA file is less convenient than using the `SlhaDerive`
/// trait, but more flexible.
/// See the crate level documentation for more information.
///
/// ```rust
/// # extern crate slha;
/// #
/// # use slha::{Slha, Block, SlhaDeserialize};
/// #
/// # fn main() {
/// let input = "
/// BLOCK MASS
///    6    173.2    # M_t
/// ";
///
/// let slha = Slha::parse(input).unwrap();
/// let mass: Block<i64, f64> = match slha.get_block("mass") {
///     Some(block) => match block {
///         Ok(mass) => mass,
///         Err(err) => panic!("Parse error while parsing block 'mass': {}", err),
///     },
///     None => panic!("Missing block 'mass'"),
/// };
/// assert_eq!(mass.scale, None);
/// assert_eq!(mass.map.len(), 1);
/// assert_eq!(mass.map[&6], 173.2);
/// # }
/// ```
///
/// # Nested maps
///
/// Nested maps, i.e. blocks that represent a map where the values are again maps, can not be
/// represented directly by this `Block` type.
/// They can however be simulated using tuples as keys.
/// For e.g. mixing matrices instead of having two nested maps, one for each index, a single tuple
/// is used as index into the block, where the tuple contains both indices into the matrix.
/// The following example shows how to access the stop mixing matrix using this technique.
///
/// ```rust
/// # extern crate slha;
/// # #[macro_use]
/// # extern crate slha_derive;
/// #
/// # use slha::{Block, SlhaDeserialize};
/// #
/// #[derive(SlhaDeserialize)]
/// struct Slha {
///     stopmix: Block<(u8,u8), f64>,
/// }
///
/// # fn main() {
/// let input = "\
/// Block stopmix  # stop mixing matrix
///    1  1     5.37975095e-01   # O_{11}
///    1  2     8.42960733e-01   # O_{12}
///    2  1     8.42960733e-01   # O_{21}
///    2  2    -5.37975095e-01   # O_{22}
/// ";
///
/// let slha = Slha::deserialize(input).unwrap();
/// let stopmix = &slha.stopmix;
/// assert_eq!(stopmix.map.len(), 4);
/// assert_eq!(stopmix.map[&(1, 1)], 5.37975095e-01);
/// assert_eq!(stopmix.map[&(1, 2)], 8.42960733e-01);
/// assert_eq!(stopmix.map[&(2, 1)], 8.42960733e-01);
/// assert_eq!(stopmix.map[&(2, 2)], -5.37975095e-01);
/// # }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Block<Key, Value>
where
    Key: Hash + Eq,
{
    /// The scale at which this block is defined, if any.
    pub scale: Option<f64>,
    /// The map from keys to values.
    pub map: HashMap<Key, Value>,
}
impl<Key, Value> SlhaBlock for Block<Key, Value>
where
    Key: Hash + Eq + Parseable,
    Value: Parseable,
{
    fn parse<'input>(lines: &[Line<'input>], scale: Option<f64>) -> Result<Self> {
        let map = parse_block_body(lines)?;
        Ok(Block { map, scale })
    }
    fn scale(&self) -> Option<f64> {
        self.scale
    }
}

/// `BlockStr` is a more flexible but less typesafe version of `Block`.
///
/// It represents a block from an SLHA file as a map from a vector of string keys to a value.
/// This type of `Block` can still be used even if only the output type is known at compile time while the
/// key types (and the number of keys) are only known at runtime.
/// This is for example the case when working with the `UFO` format, where each external parameter contains a block name in an SLHA file and
/// the keys into the block in string form.
///
/// The split between value and key is such that the value is the longest expression at the end of
/// of a line that can be parsed to the `Value` type, and everything before, split at whitespace,
/// is used as the key.
/// For example, the line
///
/// ```none
/// 1 this    5   bar   7.3 3
/// ```
///
/// would be split into `vec!["1", "this", "5", "bar"]` and `(7.3, 3.0)` if `Value` = `(f64, f64)`
/// or `vec!["1", "this", "5", "bar", "7.3"]` and `3` if `Value` = `i8`.
///
/// Duplicate keys in a block are treated as a parse error.
///
/// # Example
///
/// ```rust
/// use slha::{Slha, BlockStr};
/// # fn main() {
/// let input = "\
/// BLOCK TEST
///    1 3
///    4 6
/// block Mass
///    6  173.2
/// BloCk FooBar
///    1 2 3 4 0.5
///    1 assdf 3 4 8
///    1 2 4 8.98
/// ";
/// let slha = Slha::parse(input).unwrap();
///
/// let test: BlockStr<i64> = match slha.get_block("test") {
///     Some(block) => match block {
///         Ok(test) => test,
///         Err(err) => panic!("There was a parse error while parsing block 'test': {}", err),
///     },
///     None => panic!("Missing block 'test'"),
/// };
/// assert_eq!(test.map.len(), 2);
/// assert_eq!(test.map[&vec!["1".to_string()]], 3);
/// assert_eq!(test.map[&vec!["4".to_string()]], 6);
///
/// let mass: BlockStr<(i64, f64)> = match slha.get_block("mass") {
///     Some(block) => match block {
///         Ok(mass) => mass,
///         Err(err) => panic!("There was a parse error while parsing block 'mass': {}", err),
///     },
///     None => panic!("Missing block 'mass'"),
/// };
/// assert_eq!(mass.map.len(), 1);
/// assert_eq!(mass.map[&Vec::new()], (6, 173.2));
///
/// let foobar: BlockStr<f64> = match slha.get_block("foobar") {
///     Some(block) => match block {
///         Ok(foobar) => foobar,
///         Err(err) => panic!("There was a parse error while parsing block 'foobar': {}", err),
///     },
///     None => panic!("Missing block 'foobar'"),
/// };
/// assert_eq!(foobar.map.len(), 3);
/// assert_eq!(
///     foobar.map[&vec![
///         "1".to_string(),
///         "2".to_string(),
///         "3".to_string(),
///         "4".to_string(),
///     ]],
///     0.5
/// );
/// assert_eq!(
///     foobar.map[&vec![
///         "1".to_string(),
///         "assdf".to_string(),
///         "3".to_string(),
///         "4".to_string(),
///     ]],
///     8.
///     );
/// assert_eq!(
///     foobar.map[&vec!["1".to_string(), "2".to_string(), "4".to_string()]],
///     8.98
/// );
/// # }
/// ```
pub type BlockStr<Value> = Block<Vec<String>, Value>;

/// A block type that only contains a single value.
///
/// This type of block does not represent a map like `Block` but just a single value and an
/// optional scale.
/// This is the second type necessary to cover all blocks defined in the SLHA 1 and 2 papers.
///
/// # Reading blocks
///
/// `BlockSingle` implements the `SlhaBlock` trait and therefore can be read from an SLHA file.
/// This can be done in two different ways, the recommended way using `slha-derive` or using an
/// `Slha` object.
///
/// ## Using derive
///
/// The easiest way to read an SLHA file is to automatically derive the `SlhaDerive` trait on  a
/// struct.
///
/// ```rust
/// # extern crate slha;
/// # #[macro_use]
/// # extern crate slha_derive;
/// #
/// # use slha::{BlockSingle, SlhaDeserialize};
/// #
/// #[derive(Debug, SlhaDeserialize)]
/// struct Slha {
///     alpha: BlockSingle<f64>,
/// }
/// #
/// # fn main() {
/// let input = "
/// BLOCK ALPHA   # Effective Higgs mixing parameter
///      -1.13716828e-01   # alpha
/// ";
///
/// let slha = Slha::deserialize(input).unwrap();
/// let alpha = slha.alpha;
/// assert_eq!(alpha.scale, None);
/// assert_eq!(alpha.value, -1.13716828e-01);
/// # }
/// ```
///
///
/// ## Using an `Slha` object
///
/// Using an `Slha` object to parse an SLHA file is less convenient than using the `SlhaDerive`
/// trait, but more flexible.
/// See the crate level documentation for more information.
///
/// ```rust
/// # extern crate slha;
/// #
/// # use slha::{Slha, BlockSingle, SlhaDeserialize};
/// #
/// # fn main() {
/// let input = "
/// BLOCK ALPHA   # Effective Higgs mixing parameter
///      -1.13716828e-01   # alpha
/// ";
///
/// let slha = Slha::parse(input).unwrap();
/// let alpha: BlockSingle<f64> = match slha.get_block("alpha") {
///     Some(block) => match block {
///         Ok(alpha) => alpha,
///         Err(err) => panic!("Parse error while parsing block 'alpha': {}", err),
///     },
///     None => panic!("Missing block 'alpha'"),
/// };
/// assert_eq!(alpha.scale, None);
/// assert_eq!(alpha.value, -1.13716828e-01);
/// # }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct BlockSingle<Value> {
    pub value: Value,
    pub scale: Option<f64>,
}
impl<Value> SlhaBlock for BlockSingle<Value>
where
    Value: Parseable,
{
    fn parse<'input>(lines: &[Line<'input>], scale: Option<f64>) -> Result<Self> {
        if lines.len() != 1 {
            bail!(ErrorKind::WrongNumberOfValues(lines.len()));
        }
        let value = Value::parse_str(lines[0].data)?;
        Ok(BlockSingle { value, scale })
    }
    fn scale(&self) -> Option<f64> {
        self.scale
    }
}

/// The decay table of a particle.
///
/// The decay table as read from an SLHA file.
///
/// # Reading the decay table.
///
/// The `DecayTables` can be read from an SLHA file in two different ways, the recommended way using
/// `slha-derive` or using an `Slha` object.
///
/// ## Using derive
///
/// The easiest way to read an SLHA file is to automatically derive the `SlhaDerive` trait on  a
/// struct.
/// To also include decays in the struct add a field named `decays`.
/// This field must have a type of `HashMap<i64, DecayTable>`.
///
/// ```rust
/// extern crate slha;
/// #[macro_use]
/// extern crate slha_derive;
/// extern crate error_chain;
///
/// use slha::{DecayTable, SlhaDeserialize};
/// use error_chain::ChainedError;
/// use std::collections::HashMap;
///
/// #[derive(SlhaDeserialize)]
/// struct Slha {
///     decays: HashMap<i64, DecayTable>,
/// }
///
/// fn main() {
///     let input = "\
/// DECAY 6 1.35
///     1   2   5   24
///     ";
///
///     let slha = match Slha::deserialize(input) {
///         Ok(slha) => slha,
///         Err(err) => panic!("Failed to deserialize SLHA file:\n{}", err.display_chain()),
///     };
///     let decays = &slha.decays;
///     assert_eq!(decays.len(), 1);
///     assert_eq!(decays[&6].width, 1.35);
///     let decay = &decays[&6].decays;
///     assert_eq!(decay.len(), 1);
///     assert_eq!(decay[0].branching_ratio, 1.);
///     assert_eq!(decay[0].daughters, vec![5, 24]);
/// }
/// ```
///
/// ## Using an `Slha` object
///
/// Using an `Slha` object to parse an SLHA file is less convenient than using the `SlhaDerive`
/// trait, but more flexible.
/// See the crate level documentation for more information.
///
/// ```rust
/// use slha::{Slha, DecayTable};
///
/// let input = "\
/// DECAY 6 1.35
///     1   2   5   24
/// ";
///
/// let slha = match Slha::parse(input) {
///     Ok(slha) => slha,
///     Err(err) => panic!("Failed to deserialize SLHA file: {}", err),
/// };
/// let decay_table = match slha.get_decay(6) {
///     Some(dec) => dec,
///     None => panic!("Missing decay table for the top quark."),
/// };
/// assert_eq!(decay_table.width, 1.35);
/// let decay = &decay_table.decays;
/// assert_eq!(decay.len(), 1);
/// assert_eq!(decay[0].branching_ratio, 1.);
/// assert_eq!(decay[0].daughters, vec![5, 24]);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct DecayTable {
    /// The width of the particle.
    pub width: f64,
    /// All decay modes of the particle.
    pub decays: Vec<Decay>,
}

/// A single decay mode of a particle.
#[derive(Clone, Debug, PartialEq)]
pub struct Decay {
    /// The branching ratio of this decay mode.
    pub branching_ratio: f64,
    /// A vector of all daughter particles.
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

/// An unparsed block from an SLHA file.
///
/// `RawBlock` contains all the non-comment, non-whitespace lines that belong to a block as well as
/// the scale that the contents of the block are defined at, if any.
///
/// # Examples
///
/// ```rust
/// use slha::Slha;
///
/// let input = "\
/// Block SMINPUTS   # Standard Model inputs
///      3      0.1172  # alpha_s(MZ) SM MSbar
///      5      4.25    # Mb(mb) SM MSbar
///      6    174.3     # Mtop(pole)
/// DECAY 6 1.35
///     1   2   5   24
/// ";
///
/// let slha = match Slha::parse(input) {
///     Ok(slha) => slha,
///     Err(err) => panic!("Failed to deserialize SLHA file: {}", err),
/// };
///
/// let blocks = slha.get_raw_blocks("sminputs");
/// assert_eq!(blocks.len(), 1);
/// let sminputs = &blocks[0];
/// assert_eq!(sminputs.scale, None);
/// assert_eq!(sminputs.lines.len(), 3);
/// assert_eq!(sminputs.lines[1].data, "5      4.25    ");
/// assert_eq!(sminputs.lines[1].comment, Some("# Mb(mb) SM MSbar"));
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct RawBlock<'a> {
    /// The scale contained in the block header.
    pub scale: Option<f64>,
    /// The data lines that make up the block, in the order they appear in the SLHA file.
    pub lines: Vec<Line<'a>>,
}
impl<'a> RawBlock<'a> {
    /// Convert a `RawBlock` into a rust object.
    ///
    /// # Examples
    /// ```rust
    /// use slha::{Slha, Block};
    ///
    /// let input = "\
    /// Block SMINPUTS   # Standard Model inputs
    ///      3      0.1172  # alpha_s(MZ) SM MSbar
    ///      5      4.25    # Mb(mb) SM MSbar
    ///      6    174.3     # Mtop(pole)
    /// DECAY 6 1.35
    ///     1   2   5   24
    /// ";
    ///
    /// let slha = match Slha::parse(input) {
    ///     Ok(slha) => slha,
    ///     Err(err) => panic!("Failed to read SLHA file: {}", err),
    /// };
    ///
    /// let blocks = slha.get_raw_blocks("sminputs");
    /// assert_eq!(blocks.len(), 1);
    /// let sminputs_raw = &blocks[0];
    ///
    /// let sminputs: Block<u8, f64> = match sminputs_raw.to_block("sminputs") {
    ///     Ok(block) => block,
    ///     Err(err) => panic!("Failed to parse block 'sminputs': {}", err),
    /// };
    ///
    /// assert_eq!(sminputs.scale, None);
    /// assert_eq!(sminputs.map.len(), 3);
    /// assert_eq!(sminputs.map[&5], 4.25);
    /// ```
    pub fn to_block<B>(&self, name: &str) -> Result<B>
    where
        B: SlhaBlock,
    {
        B::parse(&self.lines, self.scale).chain_err(|| ErrorKind::InvalidBlock(name.to_string()))
    }
}

/// A partially parsed SLHA file.
///
/// `Slha` objects are another way to parse SLHA files without using the `SlhaDeserialize` trait.
/// The advantage that using an `SLHA` object has over deriving `SlhaDeserialize` for a struct is
/// that the names of the blocks and their types do not have to be known at compile time.
/// This is useful in a few special cases, for exmaple when dealing with the `UFO` format, where every
/// external parameter comes with the name of an SLHA block and a string containing the keys where
/// to find the value for this parameter in an SLHA file.
/// Since the blocks and their types are only known when reading the model file, the
/// `SlhaDeserialize` approach cannot be used.
///
/// The `Slha` object uses a two step parsing approach, because the types of Blocks aren't known
/// when reading the file and the SLHA format isn't self describing so that the information can not
/// be extracted from there.
/// Therefore the `parse` function will extract all blocks (and decays) but only parses the headers
/// of the blocks and stores their bodies.
/// The various `get_block` functions then parse the body of the block into the desired type 'on
/// demand' when accessing a block.
///
/// The `get_block` methods can parse a block into any type that implements the `SlhaBlock` trait,
/// which include `Block` (if both key and value type are known) and `BlockStr` (if only the value
/// type is known).
/// If even the value type is unknown at compile time, the `get_raw_block` methods can be used which
/// return the raw block body read from the file.
///
/// # Example
///
/// The following example shows all three ways how to access a single block, as well as how to
/// access the decay table for a given particle.
///
/// First, extract all blocks and decays into an `Slha` object:
///
/// ```rust
/// use slha::{Slha, Block, BlockStr, DecayTable};
///
/// let input = "\
/// Block SMINPUTS   # Standard Model inputs
///      3      0.1172  # alpha_s(MZ) SM MSbar
///      5      4.25    # Mb(mb) SM MSbar
///      6    174.3     # Mtop(pole)
/// DECAY 6 1.35
///     1   2   5   24
/// ";
///
/// let slha = match Slha::parse(input) {
///     Ok(slha) => slha,
///     Err(err) => panic!("Failed to deserialize SLHA file: {}", err),
/// };
/// ```
///
/// If both the value type and the key type are known, blocks can be extracted into a `Block`
/// structure:
///
/// ```rust
/// # use slha::{Slha, Block, BlockStr, DecayTable};
/// #
/// # let input = "\
/// # Block SMINPUTS   # Standard Model inputs
/// #      3      0.1172  # alpha_s(MZ) SM MSbar
/// #      5      4.25    # Mb(mb) SM MSbar
/// #      6    174.3     # Mtop(pole)
/// # DECAY 6 1.35
/// #     1   2   5   24
/// # ";
/// #
/// # let slha = match Slha::parse(input) {
/// #     Ok(slha) => slha,
/// #     Err(err) => panic!("Failed to deserialize SLHA file: {}", err),
/// # };
/// #
/// let sminputs: Block<i8, f64> = match slha.get_block("sminputs") {
///     Some(block) => match block {
///         Ok(sminputs) => sminputs,
///         Err(err) => panic!("Failed to parse block 'smimputs':\n{}", err),
///     },
///     None => panic!("Missing block 'sminputs'."),
/// };
/// assert_eq!(sminputs.scale, None);
/// assert_eq!(sminputs.map.len(), 3);
/// assert_eq!(sminputs.map[&5], 4.25);
/// ```
///
/// If only the value type is known but the key type is not, then it is still possible to use a
/// `BlockStr` to extract the block and access the values using string keys:
///
/// ```rust
/// # use slha::{Slha, Block, BlockStr, DecayTable};
/// #
/// # let input = "\
/// # Block SMINPUTS   # Standard Model inputs
/// #      3      0.1172  # alpha_s(MZ) SM MSbar
/// #      5      4.25    # Mb(mb) SM MSbar
/// #      6    174.3     # Mtop(pole)
/// # DECAY 6 1.35
/// #     1   2   5   24
/// # ";
/// #
/// # let slha = match Slha::parse(input) {
/// #     Ok(slha) => slha,
/// #     Err(err) => panic!("Failed to deserialize SLHA file: {}", err),
/// # };
/// #
/// let sminputs: BlockStr<f64> = match slha.get_block("sminputs") {
///     Some(block) => match block {
///         Ok(sminputs) => sminputs,
///         Err(err) => panic!("Failed to parse block 'smimputs':\n{}", err),
///     },
///     None => panic!("Missing block 'sminputs'."),
/// };
/// assert_eq!(sminputs.scale, None);
/// assert_eq!(sminputs.map.len(), 3);
/// assert_eq!(sminputs.map[&vec!["5".to_string()]], 4.25);
/// ```
///
/// If even the value type is not known, it is at least still possible to access the raw data lines
/// of the block:
///
/// ```rust
/// # use slha::{Slha, Block, BlockStr, DecayTable};
/// #
/// # let input = "\
/// # Block SMINPUTS   # Standard Model inputs
/// #      3      0.1172  # alpha_s(MZ) SM MSbar
/// #      5      4.25    # Mb(mb) SM MSbar
/// #      6    174.3     # Mtop(pole)
/// # DECAY 6 1.35
/// #     1   2   5   24
/// # ";
/// #
/// # let slha = match Slha::parse(input) {
/// #     Ok(slha) => slha,
/// #     Err(err) => panic!("Failed to deserialize SLHA file: {}", err),
/// # };
/// #
/// let blocks = slha.get_raw_blocks("sminputs");
/// assert_eq!(blocks.len(), 1);
/// let sminputs = &blocks[0];
/// assert_eq!(sminputs.scale, None);
/// assert_eq!(sminputs.lines.len(), 3);
/// assert_eq!(sminputs.lines[1].data, "5      4.25    ");
/// assert_eq!(sminputs.lines[1].comment, Some("# Mb(mb) SM MSbar"));
/// ```
///
/// Access the decays using `get_decay`:
///
/// ```rust
/// # use slha::{Slha, DecayTable};
/// #
/// # let input = "\
/// # Block SMINPUTS   # Standard Model inputs
/// #      3      0.1172  # alpha_s(MZ) SM MSbar
/// #      5      4.25    # Mb(mb) SM MSbar
/// #      6    174.3     # Mtop(pole)
/// # DECAY 6 1.35
/// #     1   2   5   24
/// # ";
/// #
/// # let slha = match Slha::parse(input) {
/// #     Ok(slha) => slha,
/// #     Err(err) => panic!("Failed to deserialize SLHA file: {}", err),
/// # };
/// #
/// let decay_table = match slha.get_decay(6) {
///     Some(dec) => dec,
///     None => panic!("Missing decay table for the top quark."),
/// };
/// assert_eq!(decay_table.width, 1.35);
/// let decay = &decay_table.decays;
/// assert_eq!(decay.len(), 1);
/// assert_eq!(decay[0].branching_ratio, 1.);
/// assert_eq!(decay[0].daughters, vec![5, 24]);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Slha<'a> {
    blocks: HashMap<String, Vec<RawBlock<'a>>>,
    decays: HashMap<i64, DecayTable>,
}
impl<'a> Slha<'a> {
    /// Create a new Slha object from the contents of an SLHA file.
    ///
    /// The SLHA file passed to this function is parsed down into its basic building blocks, so
    /// that decays and blocks can be easily accessed using the various getter functions.
    /// Decays are parsed completely, for blocks only the raw data is stored and only parsed into
    /// the desired form by the three `get_block` functions.
    ///
    /// # Errors
    ///
    /// Some errors can already be caught at this stage, even though most checking is only done by
    /// the accessor functions.
    /// Errors reported by this function are all errors regarding decays, as well as errors from
    /// malformed block headers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use slha::{Slha, Block, BlockStr, DecayTable};
    ///
    /// let input = "\
    /// Block SMINPUTS   # Standard Model inputs
    ///      3      0.1172  # alpha_s(MZ) SM MSbar
    ///      5      4.25    # Mb(mb) SM MSbar
    ///      6    174.3     # Mtop(pole)
    /// DECAY 6 1.35
    ///     1   2   5   24
    /// ";
    ///
    /// let slha = match Slha::parse(input) {
    ///     Ok(slha) => slha,
    ///     Err(err) => panic!("Failed to deserialize SLHA file: {}", err),
    /// };
    ///
    /// let sminputs: Block<i8, f64> = match slha.get_block("sminputs") {
    ///     Some(block) => match block {
    ///         Ok(sminputs) => sminputs,
    ///         Err(err) => panic!("Failed to parse block 'smimputs':\n{}", err),
    ///     },
    ///     None => panic!("Missing block 'sminputs'."),
    /// };
    /// assert_eq!(sminputs.scale, None);
    /// assert_eq!(sminputs.map.len(), 3);
    /// assert_eq!(sminputs.map[&5], 4.25);
    /// ```
    pub fn parse(input: &'a str) -> Result<Slha<'a>> {
        let mut slha = Slha {
            blocks: HashMap::new(),
            decays: HashMap::new(),
        };
        let mut lines = input.lines().peekable();
        while let Some(segment) = internal::parse_segment(&mut lines) {
            match segment? {
                Segment::Block { name, block } => {
                    let blocks = slha.blocks.entry(name).or_insert_with(Vec::new);
                    blocks.push(block)
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

    /// Lookup a single block by name and parse it into the required rust type.
    ///
    /// If there is no block with the given name, None is returned. If there is more than one
    /// block with the name, an error is returned inside a `Some`. Otherwise the body of the block
    /// is converted into an object of type `B` and the result is returned.
    /// If a block may appear more than once in an SLHA file, the methods `get_blocks` and
    /// `get_blocks_unchecked` may be used to access all of them.
    ///
    /// # Errors
    ///
    /// It is an error if there is more than one block with name `name` in the file.
    /// This is independently of the scale.
    /// Additionally, errors encountered while parsing the raw body of the block into an object of
    /// type `B` are returned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use slha::{Slha, Block, BlockStr, DecayTable};
    ///
    /// let input = "\
    /// Block SMINPUTS   # Standard Model inputs
    ///      3      0.1172  # alpha_s(MZ) SM MSbar
    ///      5      4.25    # Mb(mb) SM MSbar
    ///      6    174.3     # Mtop(pole)
    /// DECAY 6 1.35
    ///     1   2   5   24
    /// ";
    ///
    /// let slha = match Slha::parse(input) {
    ///     Ok(slha) => slha,
    ///     Err(err) => panic!("Failed to deserialize SLHA file: {}", err),
    /// };
    ///
    /// let sminputs: Block<i8, f64> = match slha.get_block("sminputs") {
    ///     Some(block) => match block {
    ///         Ok(sminputs) => sminputs,
    ///         Err(err) => panic!("Failed to parse block 'smimputs':\n{}", err),
    ///     },
    ///     None => panic!("Missing block 'sminputs'."),
    /// };
    /// assert_eq!(sminputs.scale, None);
    /// assert_eq!(sminputs.map.len(), 3);
    /// assert_eq!(sminputs.map[&5], 4.25);
    /// ```
    pub fn get_block<B: SlhaBlock>(&self, name: &str) -> Option<Result<B>> {
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

    /// Lookup all blocks with a given name but different scale and parse them into a vector of
    /// rust objects.
    ///
    /// This function checks that all blocks have different scales.
    /// If one block does have a scale, then all of them must have one.
    /// Only one block without a scale is allowed.
    /// As an alternative the method `get_blocks_unchecked` exists, which does not perform these
    /// sanity checks.
    /// If there is no block with the given name, the returned vector is empty.
    ///
    /// The returned blocks are in the same order as they appear in the SLHA file.
    ///
    /// # Errors
    ///
    /// If (at least) one of the blocks could not be parsed into an object of type `B`.
    /// If the blocks do not have unique scales as described above.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use slha::{Slha, Block};
    ///
    /// let input = "\
    /// BLOCK Mass
    ///     6    173.2
    /// Block ye Q= 20
    ///     3  3 9.0e-02   # Ytau(Q)MSSM DRbar
    /// Block yu Q= 10
    ///     3  3 8.88194465e-01   # Yt(Q)MSSM DRbar
    /// Block ye Q= 40
    ///     3  3 7.0e-03   # Ytau(Q)MSSM DRbar
    /// ";
    ///
    /// let slha = Slha::parse(input).unwrap();
    /// let mass: Vec<Block<i64, f64>> = match slha.get_blocks("mass") {
    ///     Ok(mass) => mass,
    ///     Err(err) => panic!("Error while parsing block mass: {}", err),
    /// };
    /// assert_eq!(mass.len(), 1);
    /// assert_eq!(mass[0].map[&6], 173.2);
    ///
    /// let ye: Vec<Block<(i8,i8), f64>> = match slha.get_blocks("ye") {
    ///     Ok(ye) => ye,
    ///     Err(err) => panic!("Error while parsing block ye: {}", err),
    /// };
    /// assert_eq!(ye.len(), 2);
    /// assert_eq!(ye[0].scale, Some(20.));
    /// assert_eq!(ye[0].map[&(3,3) ], 9.0e-02);
    /// assert_eq!(ye[1].scale, Some(40.));
    /// assert_eq!(ye[1].map[&(3,3) ], 7.0e-03);
    /// ```
    pub fn get_blocks<B: SlhaBlock>(&self, name: &str) -> Result<Vec<B>> {
        let blocks: Vec<B> = self.get_blocks_unchecked(name)?;
        let mut no_scale = false;
        let mut seen_scales = Vec::new();
        for block in &blocks {
            match block.scale() {
                Some(scale) => seen_scales.push(scale),
                None if no_scale => bail!(ErrorKind::DuplicateBlock(name.to_lowercase())),
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

    /// Lookup all blocks with a given name and parse them into a vector of rust objects.
    ///
    /// Unlike `get_blocks` this function does not check if the scales of the blocks are consistent
    /// with each other.
    /// If there is no block with the given name, the returned vector is empty.
    ///
    /// The returned blocks are in the same order as they appear in the SLHA file.
    ///
    /// # Errors
    ///
    /// If (at least) one of the blocks could not be parsed into an object of type `B`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use slha::{Slha, Block};
    ///
    /// let input = "\
    /// BLOCK Mass
    ///     6    173.2
    /// Block ye Q= 20
    ///     3  3 9.0e-02   # Ytau(Q)MSSM DRbar
    /// Block ye Q= 30
    ///     3  3 8.0e-01   # Yt(Q)MSSM DRbar
    /// BLOCK Mass
    ///     5    5.2
    /// Block ye Q= 20
    ///     3  3 7.0e-03   # Ytau(Q)MSSM DRbar
    /// ";
    ///
    /// let slha = Slha::parse(input).unwrap();
    /// let mass: Vec<Block<i64, f64>> = match slha.get_blocks_unchecked("mass") {
    ///     Ok(mass) => mass,
    ///     Err(err) => panic!("Error while parsing block mass: {}", err),
    /// };
    /// assert_eq!(mass.len(), 2);
    /// assert_eq!(mass[0].map[&6], 173.2);
    /// assert_eq!(mass[1].map[&5], 5.2);
    ///
    /// let ye: Vec<Block<(i8,i8), f64>> = match slha.get_blocks_unchecked("ye") {
    ///     Ok(ye) => ye,
    ///     Err(err) => panic!("Error while parsing block ye: {}", err),
    /// };
    /// assert_eq!(ye.len(), 3);
    /// assert_eq!(ye[0].scale, Some(20.));
    /// assert_eq!(ye[0].map[&(3,3) ], 9.0e-02);
    /// assert_eq!(ye[1].scale, Some(30.));
    /// assert_eq!(ye[1].map[&(3,3) ], 8.0e-01);
    /// assert_eq!(ye[2].scale, Some(20.));
    /// assert_eq!(ye[2].map[&(3,3) ], 7.0e-03);
    /// ```
    pub fn get_blocks_unchecked<B: SlhaBlock>(&self, name: &str) -> Result<Vec<B>> {
        let name = name.to_lowercase();
        let blocks = match self.blocks.get(&name) {
            Some(blocks) => blocks,
            None => return Ok(Vec::new()),
        };
        blocks.iter().map(|block| block.to_block(&name)).collect()
    }

    /// Returns the raw bodies of all blocks with the given names.
    ///
    /// The returned `RawBlock` objects contain all non-whitespace, non-comment lines that belong
    /// to the block.
    /// While leading whitespace is not included and neither is the newline at the end of the line,
    /// all other whitespace is still present.
    ///
    /// The returned blocks are in the same order as they appear in the SLHA file.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use slha::{Slha, Line};
    ///
    /// let input = "\
    /// BLOCK Mass
    ///     6    173.2
    /// Block ye Q= 20
    ///     3  3 9.0e-02   # First line
    /// Block ye Q= 30
    ///     3  3 8.0e-01   #    Second line
    /// BLOCK Mass
    ///     5    5.2
    /// Block ye Q= 20
    ///     3  3 7.0e-03   # Third
    /// ";
    ///
    /// let slha = Slha::parse(input).unwrap();
    /// let mass = slha.get_raw_blocks("mass");
    /// assert_eq!(mass.len(), 2);
    /// assert_eq!(mass[0].lines[0], Line { data: "6    173.2", comment: None });
    /// assert_eq!(mass[1].lines[0], Line { data: "5    5.2", comment: None });
    ///
    /// let ye = slha.get_raw_blocks("ye");
    /// assert_eq!(ye.len(), 3);
    /// assert_eq!(ye[0].scale, Some(20.));
    /// assert_eq!(ye[0].lines[0], Line { data: "3  3 9.0e-02   ", comment: Some("# First line") });
    /// assert_eq!(ye[1].scale, Some(30.));
    /// assert_eq!(ye[1].lines[0], Line { data: "3  3 8.0e-01   ", comment: Some("#    Second line") });
    /// assert_eq!(ye[2].scale, Some(20.));
    /// assert_eq!(ye[2].lines[0], Line { data: "3  3 7.0e-03   ", comment: Some("# Third") });
    /// ```
    pub fn get_raw_blocks<'s>(&'s self, name: &str) -> &'s [RawBlock<'a>] {
        let name = name.to_lowercase();
        match self.blocks.get(&name) {
            Some(blocks) => blocks,
            None => &[],
        }
    }

    /// Returns the decay table of the particle with the given pdg id.
    ///
    /// If there is no decay table for the given particle in the SLHA file, then `None` is
    /// returned.
    ///
    /// ```rust
    /// use slha::{Slha, DecayTable};
    ///
    /// let input = "\
    /// Block SMINPUTS   # Standard Model inputs
    ///      3      0.1172  # alpha_s(MZ) SM MSbar
    ///      5      4.25    # Mb(mb) SM MSbar
    ///      6    174.3     # Mtop(pole)
    /// DECAY 6 1.35
    ///     1   2   5   24
    /// ";
    ///
    /// let slha = match Slha::parse(input) {
    ///     Ok(slha) => slha,
    ///     Err(err) => panic!("Failed to deserialize SLHA file: {}", err),
    /// };
    ///
    /// let decay_table = match slha.get_decay(6) {
    ///     Some(dec) => dec,
    ///     None => panic!("Missing decay table for the top quark."),
    /// };
    /// assert_eq!(decay_table.width, 1.35);
    /// let decay = &decay_table.decays;
    /// assert_eq!(decay.len(), 1);
    /// assert_eq!(decay[0].branching_ratio, 1.);
    /// assert_eq!(decay[0].daughters, vec![5, 24]);
    /// ```
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
    use super::{Block, BlockSingle, BlockStr, Decay, Line, Parseable, Slha};
    use super::errors::{Error, ErrorKind};

    #[test]
    fn test_parse_tuple() {
        type T2 = (u8, u8);
        assert_eq!(T2::parse(&mut ["1", "2"].iter().cloned()).unwrap(), (1, 2));
        type T3 = (u8, u8, f64);
        assert_eq!(
            T3::parse(&mut ["1", "2", "9.8"].iter().cloned()).unwrap(),
            (1, 2, 9.8)
        );
        type T4 = (u8, u8, f64, String);
        assert_eq!(
            T4::parse(&mut ["1", "2", "9.8", "foo"].iter().cloned()).unwrap(),
            (1, 2, 9.8, "foo".to_string())
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
    fn test_scales() {
        let input = "\
Block T1
Block T2 Q= 17.3
Block T3 q= 9
Block T4 Q = 1e-3
Block T5 q = -4.1
Block T6 Q=8
Block T7 q=-3.7e-9
        ";

        let slha = Slha::parse(input).unwrap();
        let t1 = &slha.get_raw_blocks("T1")[0];
        assert_eq!(t1.scale, None);
        let t2 = &slha.get_raw_blocks("T2")[0];
        assert_eq!(t2.scale, Some(17.3));
        let t3 = &slha.get_raw_blocks("T3")[0];
        assert_eq!(t3.scale, Some(9.));
        let t4 = &slha.get_raw_blocks("T4")[0];
        assert_eq!(t4.scale, Some(1e-3));
        let t5 = &slha.get_raw_blocks("T5")[0];
        assert_eq!(t5.scale, Some(-4.1));
        let t6 = &slha.get_raw_blocks("T6")[0];
        assert_eq!(t6.scale, Some(8.));
        let t7 = &slha.get_raw_blocks("T7")[0];
        assert_eq!(t7.scale, Some(-3.7e-9));
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
        let sminputs: Block<i8, Vec<String>> = slha.get_block("sminputs").unwrap().unwrap();
        assert_eq!(sminputs.map.len(), 3);
        assert_eq!(
            sminputs.map[&3],
            vec![
                "0.1172".to_string(),
                "$".to_string(),
                "alpha_s(MZ)".to_string(),
                "SM".to_string(),
                "MSbar".to_string(),
            ]
        );
        assert_eq!(
            sminputs.map[&5],
            vec![
                "4.25".to_string(),
                "$".to_string(),
                "Mb(mb)".to_string(),
                "SM".to_string(),
                "MSbar".to_string(),
            ]
        );
        assert_eq!(
            sminputs.map[&6],
            vec![
                "174.3".to_string(),
                "$".to_string(),
                "Mtop(pole)".to_string(),
            ]
        );
        let modsel: Block<i8, Vec<String>> = slha.get_block("modsel").unwrap().unwrap();
        assert_eq!(modsel.map.len(), 1);
        assert_eq!(
            modsel.map[&1],
            vec!["1".to_string(), "$".to_string(), "sugra".to_string()]
        );
        let minpar: Block<i8, Vec<String>> = slha.get_block("minpar").unwrap().unwrap();
        assert_eq!(minpar.map.len(), 5);
        assert_eq!(
            minpar.map[&3],
            vec!["10.0".to_string(), "$".to_string(), "tanb".to_string()]
        );
        assert_eq!(
            minpar.map[&4],
            vec!["1.0".to_string(), "$".to_string(), "sign(mu)".to_string()]
        );
        assert_eq!(
            minpar.map[&1],
            vec!["100.0".to_string(), "$".to_string(), "m0".to_string()]
        );
        assert_eq!(
            minpar.map[&2],
            vec!["250.0".to_string(), "$".to_string(), "m12".to_string()]
        );
        assert_eq!(
            minpar.map[&5],
            vec!["-100.0".to_string(), "$".to_string(), "A0".to_string()]
        );
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
    fn test_duplicate_block_vec() {
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
        let err: Result<Vec<Block<i8, f64>>, Error> = slha.get_blocks("modsel");
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
    fn test_duplicate_key_block() {
        // Example file from appendix D.1 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
# SUSY Les Houches Accord 1.0 - example input file
# Snowmsas point 1a
Block MODSEL  # Select model
     1    1   # sugra
Block SMINPUTS   # Standard Model inputs
     6      0.1172  # alpha_s(MZ) SM MSbar
     5      4.25    # Mb(mb) SM MSbar
     6    174.3     # Mtop(pole)
Block MINPAR  # SUSY breaking input parameters
     3     10.0     # tanb
     4      1.0     # sign(mu)
     1    100.0     # m0
     2    250.0     # m12
     5   -100.0     # A0 ";

        let slha = Slha::parse(input).unwrap();
        let block: Result<Block<u8, f64>, Error> = slha.get_block("sminputs").unwrap();
        let err = block.unwrap_err();
        if let Error(ErrorKind::InvalidBlock(name), _) = err {
            assert_eq!(&name, "sminputs");
        } else {
            panic!("Wrong error variant {:?} instead of InvalidBlock", err);
        }
    }

    #[test]
    fn test_duplicate_key_blocks() {
        // Example file from appendix D.1 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
# SUSY Les Houches Accord 1.0 - example input file
# Snowmsas point 1a
Block MODSEL  # Select model
     1    1   # sugra
Block SMINPUTS   # Standard Model inputs
     6      0.1172  # alpha_s(MZ) SM MSbar
     5      4.25    # Mb(mb) SM MSbar
     6    174.3     # Mtop(pole)
Block MINPAR  # SUSY breaking input parameters
     3     10.0     # tanb
     4      1.0     # sign(mu)
     1    100.0     # m0
     2    250.0     # m12
     5   -100.0     # A0 ";

        let slha = Slha::parse(input).unwrap();
        let block: Result<Vec<Block<u8, f64>>, Error> = slha.get_blocks("sminputs");
        let err = block.unwrap_err();
        if let Error(ErrorKind::InvalidBlock(name), _) = err {
            assert_eq!(&name, "sminputs");
        } else {
            panic!("Wrong error variant {:?} instead of InvalidBlock", err);
        }
    }

    #[test]
    fn test_duplicate_key_blocksu() {
        // Example file from appendix D.1 of the slha1 paper(arXiv:hep-ph/0311123)
        let input = "\
# SUSY Les Houches Accord 1.0 - example input file
# Snowmsas point 1a
Block MODSEL  # Select model
     1    1   # sugra
Block SMINPUTS   # Standard Model inputs
     6      0.1172  # alpha_s(MZ) SM MSbar
     5      4.25    # Mb(mb) SM MSbar
     6    174.3     # Mtop(pole)
Block MINPAR  # SUSY breaking input parameters
     3     10.0     # tanb
     4      1.0     # sign(mu)
     1    100.0     # m0
     2    250.0     # m12
     5   -100.0     # A0 ";

        let slha = Slha::parse(input).unwrap();
        let block: Result<Vec<Block<u8, f64>>, Error> = slha.get_blocks_unchecked("sminputs");
        let err = block.unwrap_err();
        if let Error(ErrorKind::InvalidBlock(name), _) = err {
            assert_eq!(&name, "sminputs");
        } else {
            panic!("Wrong error variant {:?} instead of InvalidBlock", err);
        }
    }

    #[test]
    fn test_duplicate_key_blockstr() {
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
     3  5    10.0     # tanb
     4  9     1.0     # sign(mu)
     1  13  100.0     # m0
       4      9     250.0     # m12
     5  21 -100.0     # A0 ";

        let slha = Slha::parse(input).unwrap();
        let block: Result<BlockStr<f64>, Error> = slha.get_block("minpar").unwrap();
        let err = block.unwrap_err();
        if let Error(ErrorKind::InvalidBlock(name), _) = err {
            assert_eq!(&name, "minpar");
        } else {
            panic!("Wrong error variant {:?} instead of InvalidBlock", err);
        }
    }

    #[test]
    fn test_duplicate_key_blockstrs() {
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
     3  5    10.0     # tanb
     4  9     1.0     # sign(mu)
     1  13  100.0     # m0
       4      9     250.0     # m12
     5  21 -100.0     # A0 ";

        let slha = Slha::parse(input).unwrap();
        let block: Result<Vec<BlockStr<f64>>, Error> = slha.get_blocks("minpar");
        let err = block.unwrap_err();
        if let Error(ErrorKind::InvalidBlock(name), _) = err {
            assert_eq!(&name, "minpar");
        } else {
            panic!("Wrong error variant {:?} instead of InvalidBlock", err);
        }
    }

    #[test]
    fn test_duplicate_key_blockstrsu() {
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
     3  5    10.0     # tanb
     4  9     1.0     # sign(mu)
     1  13  100.0     # m0
       4      9     250.0     # m12
     5  21 -100.0     # A0 ";

        let slha = Slha::parse(input).unwrap();
        let block: Result<Vec<BlockStr<f64>>, Error> = slha.get_blocks_unchecked("minpar");
        let err = block.unwrap_err();
        if let Error(ErrorKind::InvalidBlock(name), _) = err {
            assert_eq!(&name, "minpar");
        } else {
            panic!("Wrong error variant {:?} instead of InvalidBlock", err);
        }
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
        let block: Result<BlockStr<(i8, i8, i8, i8, f64)>, Error> =
            slha.get_block("foobar").unwrap();
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
