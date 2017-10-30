// Copyright 2017 Torsten Weber
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains wrapper types that allow to modify the behaviour of the derived code.
//!
//! The types defined in this module can be used as field types in structs that `SlhaDeserialize`
//! is derived for.
//! Compared to plain `Block`s, `Option`s and `Vec`s, they have different behaviour with regard to
//! how duplicate blocks are handled.
//!
//! # Available modifiers
//!
//! There are currently two wrappers available to modify the deserialization of single blocks:
//!
//! * `TakeFirst` will ignore all but the first occurence of a block.
//! * `TakeLast` allows blocks to be 'overridden', i.e. the last occurence of a block is returned.
//!
//! The `VecUnchecked` type allows to collect all occurences of a block, without the sanity checks
//! that are performed when using a plain `Vec`.
//!
//! # Adding more (internal)
//!
//! To define additional wrapper types like this, it is sufficient to have them implement the
//! `internal::WrappedBlock` trait.
//! Any type implementing this trait can be used as a field type by the `derive` macro.

use {RawBlock, SlhaBlock};
use internal::WrappedBlock;
use errors::*;

use std::ops::Deref;

/// An alternative to `Vec` that does not check for duplicate blocks.
///
/// This type can be used to collect all occurences of a block independent of the scales that these
/// blocks may have.
///
/// # Examples
///
/// ```rust
/// extern crate slha;
/// #[macro_use]
/// extern crate slha_derive;
///
/// use slha::{SlhaDeserialize, Block};
/// use slha::modifier::VecUnchecked;
///
/// fn main() {
///    let input = "\
/// Block MODsel Q= 10 # Select model
///      3     10     # tanb
/// Block MODsel  # Select model
///      4      1     # sign(mu)
/// Block MODsel Q= 20 # Select model
///      1    100     # m0
/// Block MODsel Q= 10 # Select model
///      2    250     # m12
/// Block MODsel  # Select model
///      5   -100     # A0 ";
///
///     #[derive(Debug, SlhaDeserialize)]
///     struct MySlha {
///         modsel: VecUnchecked<Block<i8, i64>>,
///     }
///
///     let slha = MySlha::deserialize(input).unwrap();
///     let modsel = slha.modsel;
///     assert_eq!(modsel.len(), 5);
///     assert_eq!(modsel[0].scale, Some(10.));
///     assert_eq!(modsel[0].map[&3], 10);
///     assert_eq!(modsel[1].scale, None);
///     assert_eq!(modsel[1].map[&4], 1);
///     assert_eq!(modsel[2].scale, Some(20.));
///     assert_eq!(modsel[2].map[&1], 100);
///     assert_eq!(modsel[3].scale, Some(10.));
///     assert_eq!(modsel[3].map[&2], 250);
///     assert_eq!(modsel[4].scale, None);
///     assert_eq!(modsel[4].map[&5], -100);
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VecUnchecked<T>(pub Vec<T>);
impl<T> WrappedBlock<Error> for VecUnchecked<T>
where
    T: SlhaBlock,
{
    type Wrapper = Vec<T>;
    fn parse_into<'a>(block: &RawBlock<'a>, wrapped: &mut Vec<T>, name: &str) -> Result<()> {
        wrapped.push(block.to_block(name)?);
        Ok(())
    }
    fn unwrap(_name: &str, wrapped: Vec<T>) -> Result<VecUnchecked<T>> {
        Ok(VecUnchecked(wrapped))
    }
}
impl<T> Deref for VecUnchecked<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        let VecUnchecked(ref vec) = *self;
        vec
    }
}

/// A modifier that ignores all but the first occurence of a block in an SLHA file.
///
/// # Examples
///
/// ```rust
/// extern crate slha;
/// #[macro_use]
/// extern crate slha_derive;
///
/// use slha::{SlhaDeserialize, Block};
/// use slha::modifier::TakeFirst;
///
/// fn main() {
///    let input = "\
/// Block MODsel Q= 10 # Select model
///      3     10     # tanb
/// Block MODsel  # Select model
///      4      1     # sign(mu)
/// Block MODsel Q= 20 # Select model
///      1    100     # m0
/// Block MODsel Q= 10 # Select model
///      2    250     # m12
/// Block MODsel  # Select model
///      5   -100     # A0 ";
///
///     #[derive(Debug, SlhaDeserialize)]
///     struct MySlha {
///         modsel: TakeFirst<Block<i8, i64>>,
///     }
///
///     let slha = MySlha::deserialize(input).unwrap();
///     let modsel = slha.modsel;
///     assert_eq!(modsel.scale, Some(10.));
///     assert_eq!(modsel.map[&3], 10);
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TakeFirst<T>(pub T);
impl<T> WrappedBlock<Error> for TakeFirst<T>
where
    T: SlhaBlock,
{
    type Wrapper = Option<T>;
    fn parse_into<'a>(block: &RawBlock<'a>, wrapped: &mut Option<T>, name: &str) -> Result<()> {
        if wrapped.is_none() {
            *wrapped = Some(block.to_block(name)?);
        }
        Ok(())
    }
    fn unwrap(name: &str, wrapped: Option<T>) -> Result<TakeFirst<T>> {
        match wrapped {
            Some(block) => Ok(TakeFirst(block)),
            None => Err(ErrorKind::MissingBlock(name.to_string()).into()),
        }
    }
}
impl<T> Deref for TakeFirst<T> {
    type Target = T;
    fn deref(&self) -> &T {
        let TakeFirst(ref value) = *self;
        value
    }
}

/// A modifier that ignores all but the last occurence of a block in an SLHA file.
///
/// # Examples
///
/// ```rust
/// extern crate slha;
/// #[macro_use]
/// extern crate slha_derive;
///
/// use slha::{SlhaDeserialize, Block};
/// use slha::modifier::TakeLast;
///
/// fn main() {
///    let input = "\
/// Block MODsel Q= 10 # Select model
///      3     10     # tanb
/// Block MODsel  # Select model
///      4      1     # sign(mu)
/// Block MODsel Q= 20 # Select model
///      1    100     # m0
/// Block MODsel Q= 10 # Select model
///      2    250     # m12
/// Block MODsel  # Select model
///      5   -100     # A0 ";
///
///     #[derive(Debug, SlhaDeserialize)]
///     struct MySlha {
///         modsel: TakeLast<Block<i8, i64>>,
///     }
///
///     let slha = MySlha::deserialize(input).unwrap();
///     let modsel = slha.modsel;
///     assert_eq!(modsel.scale, None);
///     assert_eq!(modsel.map[&5], -100);
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TakeLast<T>(pub T);
impl<T> WrappedBlock<Error> for TakeLast<T>
where
    T: SlhaBlock,
{
    type Wrapper = Option<T>;
    fn parse_into<'a>(block: &RawBlock<'a>, wrapped: &mut Option<T>, name: &str) -> Result<()> {
        let block = block.to_block(name)?;
        *wrapped = Some(block);
        Ok(())
    }
    fn unwrap(name: &str, wrapped: Option<T>) -> Result<TakeLast<T>> {
        match wrapped {
            Some(block) => Ok(TakeLast(block)),
            None => Err(ErrorKind::MissingBlock(name.to_string()).into()),
        }
    }
}
impl<T> Deref for TakeLast<T> {
    type Target = T;
    fn deref(&self) -> &T {
        let TakeLast(ref value) = *self;
        value
    }
}
