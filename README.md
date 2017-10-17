[![Build Status](https://travis-ci.org/tweber12/slha-rs.svg?branch=master)](https://travis-ci.org/tweber12/slha-rs)
[![Build status](https://ci.appveyor.com/api/projects/status/xhr92ei8tkptlnap?svg=true)](https://ci.appveyor.com/project/TorstenWeber/slha-rs)

# The `slha` crate

The `slha` crate is a crate to read [SUSY Les Houches Accord] (or SLHA) files.
It aims to be as general as possible, in the sense that Blocks of arbitrary types are
supported.
In particular, all the Blocks defined in the SLHA [1] and [2] standards can be read using this
crate.

There are two ways to use this crate, using the (automatically derivable) `SlhaDeserialize`
trait or using an `Slha` object.
Automatically deriving `SlhaDeserialize` is the recommended way to use this library, and the
`Slha` object approach should only be used if the other can not be used, most notably in the
case when the necessary Blocks are only known at runtime.

## Usage example

The following is a full example using the `SlhaDeserialize` method to extract blocks and decays
from an SLHA file. An example using the `Slha` object method can be found in the
`Using an Slha object` section.
section.
To be able to use this example, you will have to add

```toml
...

#[dependencies]
slha = "0.5"
slha-derive = "0.5"
error-chain = "0.1"
```

to your `Cargo.toml`.

```rust
extern crate slha;
#[macro_use]
extern crate slha_derive;
extern crate error_chain;

use std::collections::HashMap;
use slha::{Block, DecayTable, SlhaDeserialize};
use error_chain::ChainedError;

#[derive(Debug, SlhaDeserialize)]
struct Slha {
    mass: Block<i64, f64>,
    ye: Vec<Block<(i8, i8), f64>>,
    decays: HashMap<i64, DecayTable>,
}

fn main() {
    // In a more realistic example, this would be read from a file somewhere.
    let input = "
BLOCK MASS
   6    173.2    # M_t
BLOCK ye Q= 1
   3   3    4.2
BLOCK ye Q= 2
   3   3    8.4
Decay 6 1.35
   1  2   5  24  # t > W+ b
";

    let slha = match Slha::deserialize(input) {
        Ok(slha) => slha,
        Err(err) => {
            eprintln!("{}", err.display_chain());
            return;
        },
    };

    assert_eq!(slha.mass.map[&6], 173.2);
    assert_eq!(slha.mass.scale, None);
    assert_eq!(slha.ye.len(), 2);
    assert_eq!(slha.ye[0].scale, Some(1.));
    assert_eq!(slha.ye[1].map[&(3,3)], 8.4);
    assert_eq!(slha.decays[&6].width, 1.35);
}
```

## Using derive

Together with the `slha-derive` crate, it is possible to deserialize a SLHA file directly into a rust
struct.

### Blocks

Each field of the struct is treated as a block in the SLHA file with the same
(case-insensitive) name as the field. The block-name that should be deserialized into a field
can be customized using the `rename` attribute.

While the fields can be of any type that implements the `SlhaBlock` trait, the most common
blocks, including all blocks defined in the SLHA 1 and 2 papers, can be expressed using two
block types defined in this crate, `Block` and `BlockSingle`.

All blocks declared in the struct must be present in the SLHA file, or an error is
returned.
Blocks that are included in the SLHA file but not in the struct are ignored.
Therefore it is possible to pick and choose the blocks that are necessary for a task without
having to include (and know the types of) all the others.

```rust
# extern crate slha;
# #[macro_use]
# extern crate slha_derive;
#
# use slha::{SlhaDeserialize, Block, BlockSingle};
#
#[derive(Debug, SlhaDeserialize)]
struct Slha {
    alpha: BlockSingle<f64>,
    mass: Block<i64, f64>,
    ye: Block<(u8, u8), f64>,
}
#
# fn main() {
let input = "
BLOCK MASS
   6    173.2    # M_t
BLOCK ye Q= 1
   3   3    4.2
BLOCK ALPHA   # Effective Higgs mixing parameter
     -1.1e-01   # alpha
";

let slha = Slha::deserialize(input).unwrap();
let mass = slha.mass;
assert_eq!(mass.scale, None);
assert_eq!(mass.map.len(), 1);
assert_eq!(mass.map[&6], 173.2);

let ye = slha.ye;
assert_eq!(ye.scale, Some(1.));
assert_eq!(ye.map.len(), 1);
assert_eq!(ye.map[&(3, 3)], 4.2);

let alpha = slha.alpha;
assert_eq!(alpha.scale, None);
assert_eq!(alpha.value, -1.1e-1);
# }
```

#### Optional blocks

The default behaviour is to return an error if a block declared in the struct is not present in
the SLHA file.
Since this is not always desireable, it is possible to mark blocks as optional by wrapping the
type of the block in an `Option`.

```rust
# extern crate slha;
# #[macro_use]
# extern crate slha_derive;
#
# use slha::{Block, SlhaDeserialize};
#
#[derive(Debug, SlhaDeserialize)]
struct Slha {
    mass: Option<Block<i64, f64>>,
}
#
# fn main() {
let present = "
BLOCK MASS
   6    173.2    # M_t
";

let present = Slha::deserialize(present).unwrap();
assert!(present.mass.is_some());
let not_present = Slha::deserialize("").unwrap();
assert!(not_present.mass.is_none());
# }
```


#### Repeated blocks

How duplicate blocks are handled depends on the type of the field that the block is written
into.

If the type is a Block, any block that appears more than once, even with different scale is an
error.
However, the SLHA standard allows for blocks (with different scales) to appear multiple times in an SLHA
file. To store all occurences of a block, the block can be wrapped in a `Vec`. An error is
returned if the blocks do not have different scales.

```rust
# extern crate slha;
# #[macro_use]
# extern crate slha_derive;
# extern crate error_chain;
#
# use std::collections::HashMap;
# use slha::{Block, SlhaDeserialize};
#
#[derive(Debug, SlhaDeserialize)]
struct Slha {
    ye: Vec<Block<(i8, i8), f64>>,
}
#
# fn main() {
let input = "
BLOCK ye Q= 1
   3   3    4.2
BLOCK ye Q= 2
   3   3    8.4
";

let slha = Slha::deserialize(input).unwrap();
assert_eq!(slha.ye.len(), 2);
assert_eq!(slha.ye[0].scale, Some(1.));
assert_eq!(slha.ye[0].map[&(3,3)], 4.2);
assert_eq!(slha.ye[1].scale, Some(2.));
assert_eq!(slha.ye[1].map[&(3,3)], 8.4);
# }
```


### Decays

Decays can be read in as well.
For this a field with name `decays` and type `HashMap<i64, DecayTable>` has to be present in
the struct.
The `decays` field then contains a map from the pdg id of the decaying particle to the
`DecayTable` of the particle.
An error is returned if there are multiple decay tables for the same particle.

```rust
# extern crate slha;
# #[macro_use]
# extern crate slha_derive;
#
# use std::collections::HashMap;
# use slha::{DecayTable, SlhaDeserialize};
#
#[derive(Debug, SlhaDeserialize)]
struct Slha {
    decays: HashMap<i64, DecayTable>,
}
#
# fn main() {
let input = "
Decay 6 1.35
   1  2   5  24  # t > W+ b
";

let slha = Slha::deserialize(input).unwrap();
assert_eq!(slha.decays[&6].width, 1.35);
let decays = &slha.decays[&6].decays;
assert_eq!(decays.len(), 1);
assert_eq!(decays[0].branching_ratio, 1.);
assert_eq!(decays[0].daughters, vec![5, 24]);
# }
```


## Using an `Slha` object

Sometimes using the `SlhaDeserialize` trait is not an option, usually if the names and/or types
of the blocks in the SLHA file are not known at compile time.
In these cases blocks from the SLHA file can still be accessed using an SLHA object, which
allows to access blocks by name with names that are only known at run time.
Accessing blocks with unknown type can be done by converting these blocks into a `BlockStr` if
only the key type of the block is unknown but the type of the value is known.
If nothing is known about the types, there is still the possibility of switching over all
possible types, or using the `RawBlock` type, which contains the unparsed body of the block.


### Accessing blocks

#### Blocks with (partially) known type

Since neither the names nor the types of the blocks contained in the SLHA file are known in
advance, blocks can only be converted into their corresponding rust type when the block is
accessed. Therefore all getter functions for blocks can return parse errors if the block is
invalid or cannot be converted into the desired type.

The SLHA object provides several methods that can be used to convert blocks into any type that
implements the `SlhaBlock` trait.
If the full type is known, the `get_block` function can be used to convert the block into
either a `Block` or a `BlockSingle` object.
If only the value type is known but not the key type, then these blocks can be converted into a
`BlockStr`, which uses a vector of 'words' as key.

#### Blocks with unknown type

As a last resort, the `get_raw_blocks` method can be used to obtain a `RawBlock` object for
each block with a given name.
This object contains the raw (string) data that makes up the block.

#### Repeated blocks

The `Slha` objects has several methods to access blocks that may appear more than once.

* The `get_block` method is supposed to be used for blocks that may only appear once in an SLHA
  file, or in cases where the calling code is not prepared to handle more than one block.
  If a block does appear multiple times, then this method will return an error when trying to
  access it.
* The `get_blocks` method. This method returns all occurences of a block in an SLHA file, as
  long as these blocks have different scales.
  As such, this method can be used to access e.g. grids for running parameters.
* The `get_blocks_unchecked` method is similar to the `get_blocks` method, but does not perform
  any sanity checks.
  Using this function it is for example possible to read duplicate blocks without scale.

#### Examples

```rust
use slha::{Slha, Block, BlockStr};

let input = "\
Block SMINPUTS   # Standard Model inputs
     3      0.1172  # alpha_s(MZ) SM MSbar
     5      4.25    # Mb(mb) SM MSbar
     6    174.3     # Mtop(pole)
DECAY 6 1.35
   1   2   5   24
";

let slha = match Slha::parse(input) {
    Ok(slha) => slha,
    Err(err) => panic!("Failed to deserialize SLHA file: {}", err),
};

// Using `Block`
let sminputs: Block<i8, f64> = match slha.get_block("sminputs") {
    Some(block) => match block {
        Ok(sminputs) => sminputs,
        Err(err) => panic!("Failed to parse block 'smimputs':\n{}", err),
    },
    None => panic!("Missing block 'sminputs'."),
};
assert_eq!(sminputs.scale, None);
assert_eq!(sminputs.map.len(), 3);
assert_eq!(sminputs.map[&5], 4.25);

// Using `BlockStr`
let sminputs: BlockStr<f64> = match slha.get_block("sminputs") {
    Some(block) => match block {
        Ok(sminputs) => sminputs,
        Err(err) => panic!("Failed to parse block 'smimputs':\n{}", err),
    },
    None => panic!("Missing block 'sminputs'."),
};
assert_eq!(sminputs.scale, None);
assert_eq!(sminputs.map.len(), 3);
assert_eq!(sminputs.map[&vec!["5".to_string()]], 4.25);

// Using `RawBlock`
let blocks = slha.get_raw_blocks("sminputs");
assert_eq!(blocks.len(), 1);
let sminputs = &blocks[0];
assert_eq!(sminputs.scale, None);
assert_eq!(sminputs.lines.len(), 3);
assert_eq!(sminputs.lines[1].data, "5      4.25    ");
assert_eq!(sminputs.lines[1].comment, Some("# Mb(mb) SM MSbar"));
```

### Accessing decays

The decay tables contained in the SLHA file can be accessed using the `get_decay` method.

#### Examples

```rust
use slha::{Slha, DecayTable};

let input = "\
Block SMINPUTS   # Standard Model inputs
     3      0.1172  # alpha_s(MZ) SM MSbar
     5      4.25    # Mb(mb) SM MSbar
     6    174.3     # Mtop(pole)
DECAY 6 1.35
    1   2   5   24
";

let slha = match Slha::parse(input) {
    Ok(slha) => slha,
    Err(err) => panic!("Failed to deserialize SLHA file: {}", err),
};

let decay_table = match slha.get_decay(6) {
    Some(dec) => dec,
    None => panic!("Missing decay table for the top quark."),
};
assert_eq!(decay_table.width, 1.35);
let decay = &decay_table.decays;
assert_eq!(decay.len(), 1);
assert_eq!(decay[0].branching_ratio, 1.);
assert_eq!(decay[0].daughters, vec![5, 24]);
```

[SUSY Les Houches Accord]: https://arxiv.org/abs/hep-ph/0311123
[1]: https://arxiv.org/abs/hep-ph/0311123
[2]: https://arxiv.org/abs/0801.0045
