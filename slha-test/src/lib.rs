// Copyright 2017 Torsten Weber
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg(test)]

extern crate slha;
#[macro_use]
extern crate slha_derive;

use std::collections::HashMap;
use slha::{Block, SlhaDeserialize, DecayTable, Decay, BlockSingle};
use slha::errors::{Error, ErrorKind};

#[test]
fn test_derive_basic() {

    #[derive(SlhaDeserialize)]
    struct Foo {
        #[allow(dead_code)]
        mass: slha::Block<i64, f64>,
    }
}

#[test]
fn test_example_1_derive() {
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

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        modsel: Block<i8, i8>,
        minpar: Block<i8, f64>,
        sminputs: Block<i8, f64>,
    }

    let slha = MySlha::deserialize(input).unwrap();
    println!("{:?}", slha);
    let sminputs = &slha.sminputs;
    assert_eq!(sminputs.map.len(), 3);
    assert_eq!(sminputs.map[&3], 0.1172);
    assert_eq!(sminputs.map[&5], 4.25);
    assert_eq!(sminputs.map[&6], 174.3);
    let modsel = &slha.modsel;
    assert_eq!(modsel.map.len(), 1);
    assert_eq!(modsel.map[&1], 1);
    let minpar = &slha.minpar;
    assert_eq!(minpar.map.len(), 5);
    assert_eq!(minpar.map[&3], 10.0);
    assert_eq!(minpar.map[&4], 1.0);
    assert_eq!(minpar.map[&1], 100.0);
    assert_eq!(minpar.map[&2], 250.0);
    assert_eq!(minpar.map[&5], -100.0);
}

#[test]
fn test_example_decay_derive() {
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

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        dcinfo: Block<i8, String>,
        decays: HashMap<i64, DecayTable>,
    }
    let slha = MySlha::deserialize(input).unwrap();
    let dcinfo = &slha.dcinfo;
    assert_eq!(dcinfo.map.len(), 2);
    assert_eq!(dcinfo.map[&1], "SDECAY");
    assert_eq!(dcinfo.map[&2], "1.0");
    let dec = &slha.decays[&1000021];
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
fn test_vec_derive() {
    let input = "\
Block yu Q= 4.64649125e+02
    3  3 8.88194465e-01   # Yt(Q)MSSM DRbar
Block yd Q= 40
    3  3 1.4e-01
Block yd Q= 50
    3  3 1.6e-01
Block ye Q= 4.64649125e+02
    3  3 9.97405356e-02   # Ytau(Q)MSSM DRbar
Block ye Q= 4.64649125e+03
    3  3 9.97405356e-03   # Ytau(Q)MSSM DRbar
";

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        yu: Vec<Block<(i8, i8), f64>>,
        yd: Vec<Block<(i8, i8), f64>>,
        ye: Vec<Block<(i8, i8), f64>>,
    }

    let slha = MySlha::deserialize(input).unwrap();
    println!("{:?}", slha);
    let yu = &slha.yu;
    assert_eq!(yu.len(), 1);
    assert_eq!(yu[0].map[&(3, 3)], 8.88194465e-01);
    let yd = &slha.yd;
    assert_eq!(yd.len(), 2);
    assert_eq!(yd[0].map[&(3, 3)], 1.4e-01);
    assert_eq!(yd[1].map[&(3, 3)], 1.6e-01);
    let ye = &slha.ye;
    assert_eq!(ye.len(), 2);
    assert_eq!(ye[0].map[&(3, 3)], 9.97405356e-02);
    assert_eq!(ye[1].map[&(3, 3)], 9.97405356e-03);
}

#[test]
fn test_single_derive() {
    let input = "\
Block yu Q= 4.64649125e+02
    3  3 8.88194465e-01   # Yt(Q)MSSM DRbar
Block yd Q= 40
    3  3 1.4e-01
Block yd Q= 50
    3  3 1.6e-01
Block ye Q= 4.64649125e+02
    9.97405356e-02   # Ytau(Q)MSSM DRbar
Block ye Q= 4.64649125e+03
    9.98405356e-03   # Ytau(Q)MSSM DRbar
";

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        yu: Vec<Block<(i8, i8), f64>>,
        yd: Vec<Block<(i8, i8), f64>>,
        ye: Vec<BlockSingle<f64>>,
    }

    let slha = MySlha::deserialize(input).unwrap();
    println!("{:?}", slha);
    let yu = &slha.yu;
    assert_eq!(yu.len(), 1);
    assert_eq!(yu[0].map[&(3, 3)], 8.88194465e-01);
    let yd = &slha.yd;
    assert_eq!(yd.len(), 2);
    assert_eq!(yd[0].map[&(3, 3)], 1.4e-01);
    assert_eq!(yd[1].map[&(3, 3)], 1.6e-01);
    let ye = &slha.ye;
    assert_eq!(ye.len(), 2);
    assert_eq!(ye[0].value, 9.97405356e-02);
    assert_eq!(ye[1].value, 9.98405356e-03);
}

#[test]
fn test_example_1_option_some() {
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

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        modsel: Block<i8, i8>,
        minpar: Option<Block<i8, f64>>,
        sminputs: Option<Block<i8, f64>>,
    }

    let slha = MySlha::deserialize(input).unwrap();
    println!("{:?}", slha);
    let sminputs = &slha.sminputs.unwrap();
    assert_eq!(sminputs.map.len(), 3);
    assert_eq!(sminputs.map[&3], 0.1172);
    assert_eq!(sminputs.map[&5], 4.25);
    assert_eq!(sminputs.map[&6], 174.3);
    let modsel = &slha.modsel;
    assert_eq!(modsel.map.len(), 1);
    assert_eq!(modsel.map[&1], 1);
    let minpar = &slha.minpar.unwrap();
    assert_eq!(minpar.map.len(), 5);
    assert_eq!(minpar.map[&3], 10.0);
    assert_eq!(minpar.map[&4], 1.0);
    assert_eq!(minpar.map[&1], 100.0);
    assert_eq!(minpar.map[&2], 250.0);
    assert_eq!(minpar.map[&5], -100.0);
}

#[test]
fn test_example_1_option_none() {
    // Example file from appendix D.1 of the slha1 paper(arXiv:hep-ph/0311123)
    let input = "\
# SUSY Les Houches Accord 1.0 - example input file
# Snowmsas point 1a
Block MODSEL  # Select model
     1    1   # sugra
Block MINPAR  # SUSY breaking input parameters
     3     10.0     # tanb
     4      1.0     # sign(mu)
     1    100.0     # m0
     2    250.0     # m12
     5   -100.0     # A0 ";

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        modsel: Block<i8, i8>,
        minpar: Option<Block<i8, f64>>,
        sminputs: Option<Block<i8, f64>>,
    }

    let slha = MySlha::deserialize(input).unwrap();
    println!("{:?}", slha);
    let sminputs = &slha.sminputs;
    assert!(sminputs.is_none());
    let modsel = &slha.modsel;
    assert_eq!(modsel.map.len(), 1);
    assert_eq!(modsel.map[&1], 1);
    let minpar = &slha.minpar.unwrap();
    assert_eq!(minpar.map.len(), 5);
    assert_eq!(minpar.map[&3], 10.0);
    assert_eq!(minpar.map[&4], 1.0);
    assert_eq!(minpar.map[&1], 100.0);
    assert_eq!(minpar.map[&2], 250.0);
    assert_eq!(minpar.map[&5], -100.0);
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

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        modsel: Block<i8, i8>,
        minpar: Block<i8, f64>,
        sminputs: Block<i8, f64>,
    }

    let err = MySlha::deserialize(input).unwrap_err();
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

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        modsel: Block<i8, i8>,
        minpar: Block<i8, f64>,
        sminputs: Block<i8, f64>,
    }

    let err = MySlha::deserialize(input).unwrap_err();
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
Block yd Q= 40
    3  3 1.4e-01
Block yd Q= 50
    3
Block ye Q= 4.64649125e+02
    3  3 9.97405356e-02   # Ytau(Q)MSSM DRbar
Block ye Q= 4.64649125e+03
    3  3 9.97405356e-03   # Ytau(Q)MSSM DRbar
     ";

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        yu: Vec<Block<(i8, i8), f64>>,
        yd: Vec<Block<(i8, i8), f64>>,
        ye: Vec<Block<(i8, i8), f64>>,
    }

    let err = MySlha::deserialize(input).unwrap_err();
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

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        modsel: Block<i8, i8>,
        minpar: Block<i8, f64>,
        sminputs: Block<i8, f64>,
    }

    let err = MySlha::deserialize(input).unwrap_err();
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

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        yu: Vec<Block<(i8, i8), f64>>,
        yd: Vec<Block<(i8, i8), f64>>,
        ye: Vec<Block<(i8, i8), f64>>,
    }

    let err = MySlha::deserialize(input).unwrap_err();
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

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        yu: Vec<Block<(i8, i8), f64>>,
        yd: Vec<Block<(i8, i8), f64>>,
        ye: Vec<Block<(i8, i8), f64>>,
    }

    let err = MySlha::deserialize(input).unwrap_err();
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

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        modsel: Block<i8, i8>,
        minpar: Block<i8, f64>,
        sminputs: Block<i8, f64>,
    }

    let err = MySlha::deserialize(input).unwrap_err();
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

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        yu: Vec<Block<(i8, i8), f64>>,
        yd: Vec<Block<(i8, i8), f64>>,
        ye: Block<(i8, i8), f64>,
    }

    let err = MySlha::deserialize(input).unwrap_err();
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

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        modsel: Block<i8, i8>,
        minpar: Block<i8, f64>,
        sminputs: Block<i8, f64>,
    }

    let err = MySlha::deserialize(input).unwrap_err();

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
     3     10     # tanb
     4      1     # sign(mu)
     1    100     # m0
     2    250     # m12
     5   -100     # A0 ";

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        modsel: Block<i8, i64>,
        minpar: Block<i8, f64>,
    }

    let err = MySlha::deserialize(input).unwrap_err();
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
     3     10     # tanb
     4      1     # sign(mu)
     1    100     # m0
     2    250     # m12
     5   -100     # A0 ";

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        modsel: Vec<Block<i8, i64>>,
        minpar: Vec<Block<i8, f64>>,
    }

    let err = MySlha::deserialize(input).unwrap_err();
    if let Error(ErrorKind::DuplicateBlock(name), _) = err {
        assert_eq!(&name, "modsel");
    } else {
        panic!("Wrong error variant {:?} instead of DuplicateBlock", err);
    }
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

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        yu: Block<(i8, i8), f64>,
        yd: Block<(i8, i8), f64>,
        ye: Block<(i8, i8), f64>,
    }

    let err = MySlha::deserialize(input).unwrap_err();
    if let Error(ErrorKind::DuplicateBlock(name), _) = err {
        assert_eq!(&name, "yu");
    } else {
        panic!("Wrong error variant {:?} instead of DuplicateBlock", err);
    }
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

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        yu: Vec<Block<(i8, i8), f64>>,
        yd: Vec<Block<(i8, i8), f64>>,
        yf: Vec<Block<(i8, i8), f64>>,
    }

    let err = MySlha::deserialize(input).unwrap_err();
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

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        yu: Vec<Block<(i8, i8), f64>>,
        yd: Vec<Block<(i8, i8), f64>>,
        yf: Vec<Block<(i8, i8), f64>>,
    }

    let err = MySlha::deserialize(input).unwrap_err();
    if let Error(ErrorKind::RedefinedBlockWithQ(name), _) = err {
        assert_eq!(&name, "yf");
    } else {
        panic!(
            "Wrong error variant {:?} instead of DuplicateBlockWithQ",
            err
        );
    }
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

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        yu: Vec<Block<(i8, i8), f64>>,
        yd: Vec<Block<(i8, i8), f64>>,
        yf: Vec<Block<(i8, i8), f64>>,
    }

    let err = MySlha::deserialize(input).unwrap_err();
    if let Error(ErrorKind::RedefinedBlockWithQ(name), _) = err {
        assert_eq!(&name, "yf");
    } else {
        panic!(
            "Wrong error variant {:?} instead of DuplicateBlockWithQ",
            err
        );
    }
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

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        modsel: Block<i8, i8>,
        minpar: Block<i8, f64>,
        sminputs: Block<i8, f64>,
    }

    let err = MySlha::deserialize(input).unwrap_err();
    if let Error(ErrorKind::InvalidBlock(name), _) = err {
        assert_eq!(&name, "sminputs");
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

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        yu: Vec<Block<(i8, i8), f64>>,
        yd: Vec<Block<(i8, i8), f64>>,
        ye: Block<(i8, i8), f64>,
    }

    let err = MySlha::deserialize(input).unwrap_err();
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

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        yu: Vec<Block<(i8, i8), f64>>,
        yd: Vec<Block<(i8, i8), f64>>,
        ye: Block<(i8, i8), f64>,
    }

    let err = MySlha::deserialize(input).unwrap_err();
    if let Error(ErrorKind::InvalidBlock(name), _) = err {
        assert_eq!(&name, "ye");
    } else {
        panic!("Wrong error variant {:?} instead of InvalidBlock", err);
    }
}

#[test]
fn test_missing_block() {
    // Example file from appendix D.1 of the slha1 paper(arXiv:hep-ph/0311123)
    let input = "\
Block yu Q= 4.64649125e+02
    3  3 8.88193465e-01   # Yt(Q)MSSM DRbar
Block yd Q= 40
    3  3 1.4e-01
Block yd Q= 50
    3  3 1.4e-01
Block yf Q= 4.64649125e+02
    3  3 9.97405356e-02   # Ytau(Q)MSSM DRbar
Block flup Q= 4.64649125e+03
    3  3 9.97405356e-03   # Ytau(Q)MSSM DRbar
     ";

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        yu: Vec<Block<(i8, i8), f64>>,
        yd: Vec<Block<(i8, i8), f64>>,
        ye: Block<(i8, i8), f64>,
    }

    let err = MySlha::deserialize(input).unwrap_err();
    if let Error(ErrorKind::MissingBlock(name), _) = err {
        assert_eq!(&name, "ye");
    } else {
        panic!("Wrong error variant {:?} instead of MissingBlock", err);
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

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        decays: HashMap<i64, DecayTable>,
    }
    let err = MySlha::deserialize(input).unwrap_err();
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

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        decays: HashMap<i64, DecayTable>,
    }
    let err = MySlha::deserialize(input).unwrap_err();
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

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        decays: HashMap<i64, DecayTable>,
    }
    let err = MySlha::deserialize(input).unwrap_err();
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

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        decays: HashMap<i64, DecayTable>,
    }
    let err = MySlha::deserialize(input).unwrap_err();
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

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        decays: HashMap<i64, DecayTable>,
    }
    let err = MySlha::deserialize(input).unwrap_err();
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
    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        decays: HashMap<i64, DecayTable>,
    }
    let err = MySlha::deserialize(input).unwrap_err();
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
    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        decays: HashMap<i64, DecayTable>,
    }
    let err = MySlha::deserialize(input).unwrap_err();
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
    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        decays: HashMap<i64, DecayTable>,
    }
    let err = MySlha::deserialize(input).unwrap_err();
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
    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        decays: HashMap<i64, DecayTable>,
    }
    let err = MySlha::deserialize(input).unwrap_err();
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
    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        decays: HashMap<i64, DecayTable>,
    }
    let err = MySlha::deserialize(input).unwrap_err();
    if let Error(ErrorKind::InvalidDecay(pdg_id), _) = err {
        assert_eq!(pdg_id, 1000022);
    } else {
        panic!("Wrong error variant {:?} instead of InvalidDecay", err);
    }
}

#[test]
fn test_parse_block_single_map() {
    let input = "\
BLOCK TEST
    3  9
";
    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        test: BlockSingle<i64>,
    }
    let err = MySlha::deserialize(input).unwrap_err();
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
    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        test: BlockSingle<i64>,
    }
    let err = MySlha::deserialize(input).unwrap_err();
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
    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        test: BlockSingle<i64>,
    }
    let err = MySlha::deserialize(input).unwrap_err();
    if let Error(ErrorKind::InvalidBlock(name), _) = err {
        assert_eq!(name, "test");
    } else {
        panic!("Wrong error variant {:?} instead of InvalidBlock", err);
    }
}

#[test]
fn test_example_1_rename() {
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

    #[derive(Debug, SlhaDeserialize)]
    struct MySlha {
        #[slha(rename = "modsel")]
        mod_sel: Block<i8, i8>,
        #[slha(rename = "minpar")]
        min_par: Block<i8, f64>,
        #[slha(rename = "sminputs")]
        sm_inputs: Block<i8, f64>,
    }

    let slha = MySlha::deserialize(input).unwrap();
    println!("{:?}", slha);
    let sminputs = &slha.sm_inputs;
    assert_eq!(sminputs.map.len(), 3);
    assert_eq!(sminputs.map[&3], 0.1172);
    assert_eq!(sminputs.map[&5], 4.25);
    assert_eq!(sminputs.map[&6], 174.3);
    let modsel = &slha.mod_sel;
    assert_eq!(modsel.map.len(), 1);
    assert_eq!(modsel.map[&1], 1);
    let minpar = &slha.min_par;
    assert_eq!(minpar.map.len(), 5);
    assert_eq!(minpar.map[&3], 10.0);
    assert_eq!(minpar.map[&4], 1.0);
    assert_eq!(minpar.map[&1], 100.0);
    assert_eq!(minpar.map[&2], 250.0);
    assert_eq!(minpar.map[&5], -100.0);
}
