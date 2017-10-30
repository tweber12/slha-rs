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
extern crate error_chain;

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

macro_rules! expect_invalid_block {
    ($name:expr, $inp:expr, $obj:ident) => {
        let err = $obj::deserialize($inp).unwrap_err();
        if let Error(ErrorKind::InvalidBlock(ref name), _) = err {
            if $name != name {
                panic!("Wrong block reported in InvalidBlock: {} instead of {}\nError: {}", &name, $name, err.display_chain());
            }
        } else {
            panic!("Wrong error variant {:?} instead of InvalidBlock", err);
        }
    }
}

macro_rules! test_invalid_block_1 {
    (block => $name:expr, $inp:expr) => {
        #[test]
        fn block() {
            #[derive(Debug, SlhaDeserialize)]
            struct MySlha {
                modsel: Block<i8, i8>,
                minpar: Block<i8, f64>,
                sminputs: Block<i8, f64>,
            }
            expect_invalid_block!($name, $inp, MySlha);
        }
    };
    (block_single => $name:expr, $inp:expr) => {
        #[test]
        fn block_single() {
            #[derive(Debug, SlhaDeserialize)]
            struct MySlha {
                modsel: BlockSingle<i8>,
                minpar: BlockSingle<f64>,
                sminputs: BlockSingle<f64>,
            }
            expect_invalid_block!($name, $inp, MySlha);
        }
    };
    (block_str => $name:expr, $inp:expr) => {
        #[test]
        fn block_str() {
            #[derive(Debug, SlhaDeserialize)]
            struct MySlha {
                modsel: BlockStr<i8>,
                minpar: BlockStr<f64>,
                sminputs: BlockStr<f64>,
            }
            expect_invalid_block!($name, $inp, MySlha);
        }
    };
    (wrapped => $name:expr, $inp:expr, $wrapper:ident, $fn_name:ident) => {
        #[test]
        fn $fn_name() {
            #[derive(Debug, SlhaDeserialize)]
            struct MySlha {
                modsel: $wrapper<Block<i8, i8>>,
                minpar: $wrapper<Block<i8, f64>>,
                sminputs: $wrapper<Block<i8, f64>>,
            }
            expect_invalid_block!($name, $inp, MySlha);
        }
    };
    (option => $name:expr, $inp:expr) => {
        test_invalid_block_1!(wrapped => $name, $inp, Option, option);
    };
    (vec => $name:expr, $inp:expr) => {
        test_invalid_block_1!(wrapped => $name, $inp, Vec, vec);
    };
    (vec_unchecked => $name:expr, $inp:expr) => {
        test_invalid_block_1!(wrapped => $name, $inp, VecUnchecked, vec_unchecked);
    };
    (take_first => $name:expr, $inp:expr) => {
        test_invalid_block_1!(wrapped => $name, $inp, TakeFirst, take_first);
    };
    (take_last => $name:expr, $inp:expr) => {
        test_invalid_block_1!(wrapped => $name, $inp, TakeLast, take_last);
    };
}

mod too_many_keys {
    use slha;
    use slha::{SlhaDeserialize, Block, BlockSingle, BlockStr};
    use slha::modifier::{VecUnchecked, TakeFirst, TakeLast};
    use slha::errors::*;
    use error_chain::ChainedError;

    const INPUT: &'static str = "\
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

    test_invalid_block_1!(block => "minpar", INPUT);
    test_invalid_block_1!(option => "minpar", INPUT);
    test_invalid_block_1!(vec => "minpar", INPUT);
    test_invalid_block_1!(vec_unchecked => "minpar", INPUT);
    test_invalid_block_1!(take_first => "minpar", INPUT);
    test_invalid_block_1!(take_last => "minpar", INPUT);

    #[test]
    fn block_str() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            modsel: BlockStr<i8>,
            minpar: BlockStr<f64>,
            sminputs: BlockStr<f64>,
        }
        let slha = MySlha::deserialize(INPUT).unwrap();
        let minpar = slha.minpar;
        assert_eq!(minpar.map[&vec!["1".to_string(), "1".to_string()]], 100.0);
    }

    const INPUT_SINGLE: &'static str = "\
Block MODSEL  # Select model
     1   # sugra
Block SMINPUTS   # Standard Model inputs
     1  0.1172  # alpha_s(MZ) SM MSbar
Block MINPAR  # SUSY breaking input parameters
     10.0     # tanb
    ";

    test_invalid_block_1!(block_single => "sminputs", INPUT_SINGLE);
}

mod not_enough_keys {
    use slha;
    use slha::{SlhaDeserialize, Block, BlockSingle, BlockStr};
    use slha::modifier::{VecUnchecked, TakeFirst, TakeLast};
    use slha::errors::*;
    use error_chain::ChainedError;

    const INPUT: &'static str = "\
Block MODSEL  # Select model
     1    1   # sugra
Block SMINPUTS   # Standard Model inputs
     3      0.1172  # alpha_s(MZ) SM MSbar
            4.25    # Mb(mb) SM MSbar
     6    174.3     # Mtop(pole)
Block MINPAR  # SUSY breaking input parameters
     3     10.0     # tanb
     4      1.0     # sign(mu)
     1    100.0     # m0
     2    250.0     # m12
     5   -100.0     # A0 ";
    const BLOCK: &'static str = "sminputs";

    test_invalid_block_1!(block => BLOCK, INPUT);
    test_invalid_block_1!(option => BLOCK, INPUT);
    test_invalid_block_1!(vec => BLOCK, INPUT);
    test_invalid_block_1!(vec_unchecked => BLOCK, INPUT);
    test_invalid_block_1!(take_first => BLOCK, INPUT);
    test_invalid_block_1!(take_last => BLOCK, INPUT);

    #[test]
    fn block_str() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            modsel: BlockStr<i8>,
            minpar: BlockStr<f64>,
            sminputs: BlockStr<f64>,
        }
        let slha = MySlha::deserialize(INPUT).unwrap();
        let sminputs = slha.sminputs;
        assert_eq!(sminputs.map[&vec![]], 4.25);
    }

    const INPUT_SINGLE: &'static str = "\
Block MODSEL  # Select model
        # sugra
Block SMINPUTS   # Standard Model inputs
     1  0.1172  # alpha_s(MZ) SM MSbar
Block MINPAR  # SUSY breaking input parameters
     10.0     # tanb
    ";

    test_invalid_block_1!(block_single => "modsel", INPUT_SINGLE);
}

mod invalid_key {
    use slha;
    use slha::{SlhaDeserialize, Block, BlockStr};
    use slha::modifier::{VecUnchecked, TakeFirst, TakeLast};
    use slha::errors::*;
    use error_chain::ChainedError;

    const INPUT: &'static str = "\
Block MODSEL  # Select model
     1.5    1   # sugra
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
    const BLOCK: &'static str = "modsel";

    test_invalid_block_1!(block => BLOCK, INPUT);
    test_invalid_block_1!(option => BLOCK, INPUT);
    test_invalid_block_1!(vec => BLOCK, INPUT);
    test_invalid_block_1!(vec_unchecked => BLOCK, INPUT);
    test_invalid_block_1!(take_first => BLOCK, INPUT);
    test_invalid_block_1!(take_last => BLOCK, INPUT);

    #[test]
    fn block_str() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            modsel: BlockStr<i8>,
            minpar: BlockStr<f64>,
            sminputs: BlockStr<f64>,
        }
        let slha = MySlha::deserialize(INPUT).unwrap();
        let block = slha.modsel;
        assert_eq!(block.map[&vec!["1.5".to_string()]], 1);
    }
}

mod invalid_value {
    use slha;
    use slha::{SlhaDeserialize, Block, BlockSingle, BlockStr};
    use slha::modifier::{VecUnchecked, TakeFirst, TakeLast};
    use slha::errors::*;
    use error_chain::ChainedError;

    const INPUT: &'static str = "\
Block MODSEL  # Select model
     1    1   # sugra
Block SMINPUTS   # Standard Model inputs
     3      0.1172  # alpha_s(MZ) SM MSbar
     5      4.25    # Mb(mb) SM MSbar
     6    174.3     # Mtop(pole)
Block MINPAR  # SUSY breaking input parameters
     3     10x0     # tanb
     4      1.0     # sign(mu)
     1    100.0     # m0
     2    250.0     # m12
     5   -100.0     # A0 ";
    const BLOCK: &'static str = "minpar";

    test_invalid_block_1!(block => BLOCK, INPUT);
    test_invalid_block_1!(option => BLOCK, INPUT);
    test_invalid_block_1!(vec => BLOCK, INPUT);
    test_invalid_block_1!(vec_unchecked => BLOCK, INPUT);
    test_invalid_block_1!(take_first => BLOCK, INPUT);
    test_invalid_block_1!(take_last => BLOCK, INPUT);
    test_invalid_block_1!(block_str => BLOCK, INPUT);


    const INPUT_SINGLE: &'static str = "\
Block MODSEL  # Select model
     1   # sugra
Block SMINPUTS   # Standard Model inputs
     0.1172  # alpha_s(MZ) SM MSbar
Block MINPAR  # SUSY breaking input parameters
     10x0     # tanb
    ";

    test_invalid_block_1!(block_single => BLOCK, INPUT_SINGLE);
}

mod malformed_block_header {
    use slha;
    use slha::{SlhaDeserialize, Block, BlockSingle, BlockStr};
    use slha::modifier::{VecUnchecked, TakeFirst, TakeLast};
    use slha::errors::*;
    use error_chain::ChainedError;

    const INPUT: &'static str = "\
# SUSY Les Houches Accord 1.0 - example input file
# Snowmsas point 1a
Block MODSEL  # Select model
     1    1   # sugra
Block SMINPUTS foo   # Standard Model inputs
     3      0.1172  # alpha_s(MZ) SM MSbar
     5      1.23    # Mb(mb) SM MSbar
     6    174.3     # Mtop(pole)
Block MINPAR  # SUSY breaking input parameters
     3     10.0     # tanb
     4      1.0     # sign(mu)
     1    100.0     # m0
     2    250.0     # m12
     5   -100.0     # A0 ";
    const BLOCK: &'static str = "sminputs";

    test_invalid_block_1!(block => BLOCK, INPUT);
    test_invalid_block_1!(option => BLOCK, INPUT);
    test_invalid_block_1!(vec => BLOCK, INPUT);
    test_invalid_block_1!(vec_unchecked => BLOCK, INPUT);
    test_invalid_block_1!(take_first => BLOCK, INPUT);
    test_invalid_block_1!(take_last => BLOCK, INPUT);
    test_invalid_block_1!(block_str => BLOCK, INPUT);


    const INPUT_SINGLE: &'static str = "\
Block MODSEL  # Select model
     1   # sugra
Block SMINPUTS foo   # Standard Model inputs
     0.1172  # alpha_s(MZ) SM MSbar
Block MINPAR  # SUSY breaking input parameters
     10.0     # tanb
    ";

    test_invalid_block_1!(block_single => BLOCK, INPUT_SINGLE);
}

mod duplicate_block {
    use slha;
    use slha::{SlhaDeserialize, Block, BlockSingle, BlockStr};
    use slha::modifier::{VecUnchecked, TakeFirst, TakeLast};
    use slha::errors::*;

    const INPUT: &'static str = "\
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
    const BLOCK: &'static str = "modsel";

    #[test]
    fn block() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            modsel: Block<i8, i64>,
            sminputs: Block<i8, f64>,
        }

        let err = MySlha::deserialize(INPUT).unwrap_err();
        if let Error(ErrorKind::DuplicateBlock(name), _) = err {
            assert_eq!(&name, BLOCK);
        } else {
            panic!("Wrong error variant {:?} instead of DuplicateBlock", err);
        }
    }

    #[test]
    fn option() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            modsel: Option<Block<i8, i64>>,
            sminputs: Option<Block<i8, f64>>,
        }

        let err = MySlha::deserialize(INPUT).unwrap_err();
        if let Error(ErrorKind::DuplicateBlock(name), _) = err {
            assert_eq!(&name, BLOCK);
        } else {
            panic!("Wrong error variant {:?} instead of DuplicateBlock", err);
        }
    }

    #[test]
    fn vec() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            modsel: Vec<Block<i8, i64>>,
            sminputs: Vec<Block<i8, f64>>,
        }

        let err = MySlha::deserialize(INPUT).unwrap_err();
        if let Error(ErrorKind::DuplicateBlock(name), _) = err {
            assert_eq!(&name, BLOCK);
        } else {
            panic!("Wrong error variant {:?} instead of DuplicateBlock", err);
        }
    }

    #[test]
    fn vec_unchecked() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            modsel: VecUnchecked<Block<i8, i64>>,
            sminputs: VecUnchecked<Block<i8, f64>>,
        }

        let slha = MySlha::deserialize(INPUT).unwrap();
        let modsel = slha.modsel;
        assert_eq!(modsel.len(), 2);
        assert_eq!(modsel[0].map.len(), 1);
        assert_eq!(modsel[0].map[&1], 1);
        assert_eq!(modsel[1].map.len(), 5);
        assert_eq!(modsel[1].map[&1], 100);
        assert_eq!(modsel[1].map[&5], -100);
    }

    #[test]
    fn take_first() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            modsel: TakeFirst<Block<i8, i64>>,
            sminputs: TakeFirst<Block<i8, f64>>,
        }

        let slha = MySlha::deserialize(INPUT).unwrap();
        let modsel = slha.modsel;
        assert_eq!(modsel.map.len(), 1);
        assert_eq!(modsel.map[&1], 1);
    }

    #[test]
    fn take_last() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            modsel: TakeLast<Block<i8, i64>>,
            sminputs: TakeLast<Block<i8, f64>>,
        }

        let slha = MySlha::deserialize(INPUT).unwrap();
        let modsel = slha.modsel;
        assert_eq!(modsel.map.len(), 5);
        assert_eq!(modsel.map[&1], 100);
        assert_eq!(modsel.map[&5], -100);
    }

    #[test]
    fn block_str() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            modsel: BlockStr<i64>,
            sminputs: BlockStr<f64>,
        }

        let err = MySlha::deserialize(INPUT).unwrap_err();
        if let Error(ErrorKind::DuplicateBlock(name), _) = err {
            assert_eq!(&name, BLOCK);
        } else {
            panic!("Wrong error variant {:?} instead of DuplicateBlock", err);
        }
    }

    const INPUT_SINGLE: &'static str = "\
Block MODSEL  # Select model
     1   # sugra
Block SMINPUTS   # Standard Model inputs
     0.1172  # alpha_s(MZ) SM MSbar
Block MODSEL  # SUSY breaking input parameters
     10      # tanb
    ";

    #[test]
    fn block_single() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            modsel: Vec<BlockSingle<i64>>,
            sminputs: Vec<BlockSingle<f64>>,
        }

        let err = MySlha::deserialize(INPUT_SINGLE).unwrap_err();
        if let Error(ErrorKind::DuplicateBlock(name), _) = err {
            assert_eq!(&name, BLOCK);
        } else {
            panic!("Wrong error variant {:?} instead of DuplicateBlock", err);
        }
    }
}

mod duplicate_block_different_scale {
    use slha;
    use slha::{SlhaDeserialize, Block, BlockSingle, BlockStr};
    use slha::modifier::{VecUnchecked, TakeFirst, TakeLast};
    use slha::errors::*;

    const INPUT: &'static str = "\
Block yu Q= 4.64649125e+02
    3  3 8.88193465e-01   # Yt(Q)MSSM DRbar
Block yu Q= 8
    3  3 1.4e-01
Block yd Q= 50
    3  3 1.4e-01
Block yf Q= 4.64649125e+02
    3  3 9.97405356e-02   # Ytau(Q)MSSM DRbar
    ";
    const BLOCK: &'static str = "yu";

    #[test]
    fn block() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: Block<(i8, i8), f64>,
            yd: Block<(i8, i8), f64>,
            yf: Block<(i8, i8), f64>,
        }

        let err = MySlha::deserialize(INPUT).unwrap_err();
        if let Error(ErrorKind::DuplicateBlock(name), _) = err {
            assert_eq!(&name, BLOCK);
        } else {
            panic!("Wrong error variant {:?} instead of DuplicateBlock", err);
        }
    }

    #[test]
    fn option() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: Option<Block<(i8, i8), f64>>,
            yd: Option<Block<(i8, i8), f64>>,
            yf: Option<Block<(i8, i8), f64>>,
        }

        let err = MySlha::deserialize(INPUT).unwrap_err();
        if let Error(ErrorKind::DuplicateBlock(name), _) = err {
            assert_eq!(&name, BLOCK);
        } else {
            panic!("Wrong error variant {:?} instead of DuplicateBlock", err);
        }
    }

    #[test]
    fn vec() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: Vec<Block<(i8, i8), f64>>,
            yd: Vec<Block<(i8, i8), f64>>,
            yf: Vec<Block<(i8, i8), f64>>,
        }

        let slha = MySlha::deserialize(INPUT).unwrap();
        let block = slha.yu;
        assert_eq!(block.len(), 2);
        assert_eq!(block[0].scale, Some(4.64649125e+02));
        assert_eq!(block[0].map[&(3, 3)], 8.88193465e-01);
        assert_eq!(block[1].scale, Some(8.));
        assert_eq!(block[1].map[&(3, 3)], 0.14);
    }

    #[test]
    fn vec_unchecked() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: VecUnchecked<Block<(i8, i8), f64>>,
            yd: VecUnchecked<Block<(i8, i8), f64>>,
            yf: VecUnchecked<Block<(i8, i8), f64>>,
        }

        let slha = MySlha::deserialize(INPUT).unwrap();
        let block = slha.yu;
        assert_eq!(block.len(), 2);
        assert_eq!(block[0].scale, Some(4.64649125e+02));
        assert_eq!(block[0].map[&(3, 3)], 8.88193465e-01);
        assert_eq!(block[1].scale, Some(8.));
        assert_eq!(block[1].map[&(3, 3)], 0.14);
    }

    #[test]
    fn take_first() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: TakeFirst<Block<(i8, i8), f64>>,
            yd: TakeFirst<Block<(i8, i8), f64>>,
            yf: TakeFirst<Block<(i8, i8), f64>>,
        }

        let slha = MySlha::deserialize(INPUT).unwrap();
        let block = slha.yu;
        assert_eq!(block.scale, Some(4.64649125e+02));
        assert_eq!(block.map[&(3, 3)], 8.88193465e-01);
    }

    #[test]
    fn take_last() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: TakeLast<Block<(i8, i8), f64>>,
            yd: TakeLast<Block<(i8, i8), f64>>,
            yf: TakeLast<Block<(i8, i8), f64>>,
        }

        let slha = MySlha::deserialize(INPUT).unwrap();
        let block = slha.yu;
        assert_eq!(block.scale, Some(8.));
        assert_eq!(block.map[&(3, 3)], 0.14);
    }

    #[test]
    fn block_str() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: BlockStr<f64>,
            yd: BlockStr<f64>,
            yf: BlockStr<f64>,
        }

        let err = MySlha::deserialize(INPUT).unwrap_err();
        if let Error(ErrorKind::DuplicateBlock(name), _) = err {
            assert_eq!(&name, BLOCK);
        } else {
            panic!("Wrong error variant {:?} instead of DuplicateBlock", err);
        }
    }

    const INPUT_SINGLE: &'static str = "\
Block yu Q= 4.64649125e+02
    8.88193465e-01   # Yt(Q)MSSM DRbar
Block yu Q= 8
    1.4e-01
Block yd Q= 50
    1.4e-01
Block yf Q= 4.64649125e+02
    9.97405356e-02   # Ytau(Q)MSSM DRbar
    ";

    #[test]
    fn block_single() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: BlockSingle<f64>,
            yd: BlockSingle<f64>,
            yf: BlockSingle<f64>,
        }

        let err = MySlha::deserialize(INPUT_SINGLE).unwrap_err();
        if let Error(ErrorKind::DuplicateBlock(name), _) = err {
            assert_eq!(&name, BLOCK);
        } else {
            panic!("Wrong error variant {:?} instead of DuplicateBlock", err);
        }
    }
}

mod duplicate_block_equal_scale {
    use slha;
    use slha::{SlhaDeserialize, Block, BlockSingle, BlockStr};
    use slha::modifier::{VecUnchecked, TakeFirst, TakeLast};
    use slha::errors::*;

    const INPUT: &'static str = "\
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
    const BLOCK: &'static str = "yf";

    #[test]
    fn block() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: Block<(i8, i8), f64>,
            yd: Block<(i8, i8), f64>,
            yf: Block<(i8, i8), f64>,
        }

        let err = MySlha::deserialize(INPUT).unwrap_err();
        if let Error(ErrorKind::DuplicateBlock(name), _) = err {
            assert_eq!(&name, BLOCK);
        } else {
            panic!("Wrong error variant {:?} instead of DuplicateBlock", err);
        }
    }

    #[test]
    fn option() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: Option<Block<(i8, i8), f64>>,
            yd: Option<Block<(i8, i8), f64>>,
            yf: Option<Block<(i8, i8), f64>>,
        }

        let err = MySlha::deserialize(INPUT).unwrap_err();
        if let Error(ErrorKind::DuplicateBlock(name), _) = err {
            assert_eq!(&name, BLOCK);
        } else {
            panic!("Wrong error variant {:?} instead of DuplicateBlock", err);
        }
    }

    #[test]
    fn vec() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: Vec<Block<(i8, i8), f64>>,
            yd: Vec<Block<(i8, i8), f64>>,
            yf: Vec<Block<(i8, i8), f64>>,
        }

        let err = MySlha::deserialize(INPUT).unwrap_err();
        if let Error(ErrorKind::DuplicateBlockScale(name, scale), _) = err {
            assert_eq!(&name, BLOCK);
            assert_eq!(scale, 4.64649125e+02);
        } else {
            panic!(
                "Wrong error variant {:?} instead of DuplicateBlockScale",
                err
            );
        }
    }

    #[test]
    fn vec_unchecked() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: VecUnchecked<Block<(i8, i8), f64>>,
            yd: VecUnchecked<Block<(i8, i8), f64>>,
            yf: VecUnchecked<Block<(i8, i8), f64>>,
        }

        let slha = MySlha::deserialize(INPUT).unwrap();
        let block = slha.yf;
        assert_eq!(block.len(), 2);
        assert_eq!(block[0].scale, Some(4.64649125e+02));
        assert_eq!(block[0].map[&(3, 3)], 8.88193465e-01);
        assert_eq!(block[1].scale, Some(4.64649125e+02));
        assert_eq!(block[1].map[&(3, 3)], 9.97405356e-02);
    }

    #[test]
    fn take_first() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: TakeFirst<Block<(i8, i8), f64>>,
            yd: TakeFirst<Block<(i8, i8), f64>>,
            yf: TakeFirst<Block<(i8, i8), f64>>,
        }

        let slha = MySlha::deserialize(INPUT).unwrap();
        let block = slha.yf;
        assert_eq!(block.scale, Some(4.64649125e+02));
        assert_eq!(block.map[&(3, 3)], 8.88193465e-01);
    }

    #[test]
    fn take_last() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: TakeLast<Block<(i8, i8), f64>>,
            yd: TakeLast<Block<(i8, i8), f64>>,
            yf: TakeLast<Block<(i8, i8), f64>>,
        }

        let slha = MySlha::deserialize(INPUT).unwrap();
        let block = slha.yf;
        assert_eq!(block.scale, Some(4.64649125e+02));
        assert_eq!(block.map[&(3, 3)], 9.97405356e-02);
    }

    #[test]
    fn block_str() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: BlockStr<f64>,
            yd: BlockStr<f64>,
            yf: BlockStr<f64>,
        }

        let err = MySlha::deserialize(INPUT).unwrap_err();
        if let Error(ErrorKind::DuplicateBlock(name), _) = err {
            assert_eq!(&name, BLOCK);
        } else {
            panic!("Wrong error variant {:?} instead of DuplicateBlock", err);
        }
    }

    const INPUT_SINGLE: &'static str = "\
Block yf Q= 4.64649125e+02
    8.88193465e-01   # Yt(Q)MSSM DRbar
Block yu Q= 8
    1.4e-01
Block yd Q= 50
    1.4e-01
Block yf Q= 4.64649125e+02
    9.97405356e-02   # Ytau(Q)MSSM DRbar
Block flup Q= 4.64649125e+03
    9.97405356e-03   # Ytau(Q)MSSM DRbar
    ";

    #[test]
    fn block_single() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: BlockSingle<f64>,
            yd: BlockSingle<f64>,
            yf: BlockSingle<f64>,
        }

        let err = MySlha::deserialize(INPUT_SINGLE).unwrap_err();
        if let Error(ErrorKind::DuplicateBlock(name), _) = err {
            assert_eq!(&name, BLOCK);
        } else {
            panic!("Wrong error variant {:?} instead of DuplicateBlock", err);
        }
    }
}

mod duplicate_block_noscale_scale {
    use slha;
    use slha::{SlhaDeserialize, Block, BlockSingle, BlockStr};
    use slha::modifier::{VecUnchecked, TakeFirst, TakeLast};
    use slha::errors::*;

    const INPUT: &'static str = "\
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
    const BLOCK: &'static str = "yf";

    #[test]
    fn block() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: Block<(i8, i8), f64>,
            yd: Block<(i8, i8), f64>,
            yf: Block<(i8, i8), f64>,
        }

        let err = MySlha::deserialize(INPUT).unwrap_err();
        if let Error(ErrorKind::DuplicateBlock(name), _) = err {
            assert_eq!(&name, BLOCK);
        } else {
            panic!("Wrong error variant {:?} instead of DuplicateBlock", err);
        }
    }

    #[test]
    fn option() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: Option<Block<(i8, i8), f64>>,
            yd: Option<Block<(i8, i8), f64>>,
            yf: Option<Block<(i8, i8), f64>>,
        }

        let err = MySlha::deserialize(INPUT).unwrap_err();
        if let Error(ErrorKind::DuplicateBlock(name), _) = err {
            assert_eq!(&name, BLOCK);
        } else {
            panic!("Wrong error variant {:?} instead of DuplicateBlock", err);
        }
    }

    #[test]
    fn vec() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: Vec<Block<(i8, i8), f64>>,
            yd: Vec<Block<(i8, i8), f64>>,
            yf: Vec<Block<(i8, i8), f64>>,
        }

        let err = MySlha::deserialize(INPUT).unwrap_err();
        if let Error(ErrorKind::RedefinedBlockWithQ(name), _) = err {
            assert_eq!(&name, BLOCK);
        } else {
            panic!(
                "Wrong error variant {:?} instead of RedefinedBlockWithQ",
                err
            );
        }
    }

    #[test]
    fn vec_unchecked() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: VecUnchecked<Block<(i8, i8), f64>>,
            yd: VecUnchecked<Block<(i8, i8), f64>>,
            yf: VecUnchecked<Block<(i8, i8), f64>>,
        }

        let slha = MySlha::deserialize(INPUT).unwrap();
        let block = slha.yf;
        assert_eq!(block.len(), 2);
        assert_eq!(block[0].scale, None);
        assert_eq!(block[0].map[&(3, 3)], 8.88193465e-01);
        assert_eq!(block[1].scale, Some(4.64649125e+02));
        assert_eq!(block[1].map[&(3, 3)], 9.97405356e-02);
    }

    #[test]
    fn take_first() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: TakeFirst<Block<(i8, i8), f64>>,
            yd: TakeFirst<Block<(i8, i8), f64>>,
            yf: TakeFirst<Block<(i8, i8), f64>>,
        }

        let slha = MySlha::deserialize(INPUT).unwrap();
        let block = slha.yf;
        assert_eq!(block.scale, None);
        assert_eq!(block.map[&(3, 3)], 8.88193465e-01);
    }

    #[test]
    fn take_last() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: TakeLast<Block<(i8, i8), f64>>,
            yd: TakeLast<Block<(i8, i8), f64>>,
            yf: TakeLast<Block<(i8, i8), f64>>,
        }

        let slha = MySlha::deserialize(INPUT).unwrap();
        let block = slha.yf;
        assert_eq!(block.scale, Some(4.64649125e+02));
        assert_eq!(block.map[&(3, 3)], 9.97405356e-02);
    }

    #[test]
    fn block_str() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: BlockStr<f64>,
            yd: BlockStr<f64>,
            yf: BlockStr<f64>,
        }

        let err = MySlha::deserialize(INPUT).unwrap_err();
        if let Error(ErrorKind::DuplicateBlock(name), _) = err {
            assert_eq!(&name, BLOCK);
        } else {
            panic!("Wrong error variant {:?} instead of DuplicateBlock", err);
        }
    }

    const INPUT_SINGLE: &'static str = "\
Block yf
    8.88193465e-01   # Yt(Q)MSSM DRbar
Block yu Q= 8
    1.4e-01
Block yd Q= 50
    1.4e-01
Block yf Q= 4.64649125e+02
    9.97405356e-02   # Ytau(Q)MSSM DRbar
Block flup Q= 4.64649125e+03
    9.97405356e-03   # Ytau(Q)MSSM DRbar
    ";

    #[test]
    fn block_single() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: BlockSingle<f64>,
            yd: BlockSingle<f64>,
            yf: BlockSingle<f64>,
        }

        let err = MySlha::deserialize(INPUT_SINGLE).unwrap_err();
        if let Error(ErrorKind::DuplicateBlock(name), _) = err {
            assert_eq!(&name, BLOCK);
        } else {
            panic!("Wrong error variant {:?} instead of DuplicateBlock", err);
        }
    }
}

mod duplicate_block_scale_noscale {
    use slha;
    use slha::{SlhaDeserialize, Block, BlockSingle, BlockStr};
    use slha::modifier::{VecUnchecked, TakeFirst, TakeLast};
    use slha::errors::*;

    const INPUT: &'static str = "\
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
    const BLOCK: &'static str = "yf";

    #[test]
    fn block() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: Block<(i8, i8), f64>,
            yd: Block<(i8, i8), f64>,
            yf: Block<(i8, i8), f64>,
        }

        let err = MySlha::deserialize(INPUT).unwrap_err();
        if let Error(ErrorKind::DuplicateBlock(name), _) = err {
            assert_eq!(&name, BLOCK);
        } else {
            panic!("Wrong error variant {:?} instead of DuplicateBlock", err);
        }
    }

    #[test]
    fn option() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: Option<Block<(i8, i8), f64>>,
            yd: Option<Block<(i8, i8), f64>>,
            yf: Option<Block<(i8, i8), f64>>,
        }

        let err = MySlha::deserialize(INPUT).unwrap_err();
        if let Error(ErrorKind::DuplicateBlock(name), _) = err {
            assert_eq!(&name, BLOCK);
        } else {
            panic!("Wrong error variant {:?} instead of DuplicateBlock", err);
        }
    }

    #[test]
    fn vec() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: Vec<Block<(i8, i8), f64>>,
            yd: Vec<Block<(i8, i8), f64>>,
            yf: Vec<Block<(i8, i8), f64>>,
        }

        let err = MySlha::deserialize(INPUT).unwrap_err();
        if let Error(ErrorKind::RedefinedBlockWithQ(name), _) = err {
            assert_eq!(&name, BLOCK);
        } else {
            panic!(
                "Wrong error variant {:?} instead of RedefinedBlockWithQ",
                err
            );
        }
    }

    #[test]
    fn vec_unchecked() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: VecUnchecked<Block<(i8, i8), f64>>,
            yd: VecUnchecked<Block<(i8, i8), f64>>,
            yf: VecUnchecked<Block<(i8, i8), f64>>,
        }

        let slha = MySlha::deserialize(INPUT).unwrap();
        let block = slha.yf;
        assert_eq!(block.len(), 2);
        assert_eq!(block[0].scale, Some(4.64649125e+02));
        assert_eq!(block[0].map[&(3, 3)], 8.88193465e-01);
        assert_eq!(block[1].scale, None);
        assert_eq!(block[1].map[&(3, 3)], 9.97405356e-02);
    }

    #[test]
    fn take_first() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: TakeFirst<Block<(i8, i8), f64>>,
            yd: TakeFirst<Block<(i8, i8), f64>>,
            yf: TakeFirst<Block<(i8, i8), f64>>,
        }

        let slha = MySlha::deserialize(INPUT).unwrap();
        let block = slha.yf;
        assert_eq!(block.scale, Some(4.64649125e+02));
        assert_eq!(block.map[&(3, 3)], 8.88193465e-01);
    }

    #[test]
    fn take_last() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: TakeLast<Block<(i8, i8), f64>>,
            yd: TakeLast<Block<(i8, i8), f64>>,
            yf: TakeLast<Block<(i8, i8), f64>>,
        }

        let slha = MySlha::deserialize(INPUT).unwrap();
        let block = slha.yf;
        assert_eq!(block.scale, None);
        assert_eq!(block.map[&(3, 3)], 9.97405356e-02);
    }

    #[test]
    fn block_str() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: BlockStr<f64>,
            yd: BlockStr<f64>,
            yf: BlockStr<f64>,
        }

        let err = MySlha::deserialize(INPUT).unwrap_err();
        if let Error(ErrorKind::DuplicateBlock(name), _) = err {
            assert_eq!(&name, BLOCK);
        } else {
            panic!("Wrong error variant {:?} instead of DuplicateBlock", err);
        }
    }

    const INPUT_SINGLE: &'static str = "\
Block yf Q= 4.64649125e+02
    8.88193465e-01   # Yt(Q)MSSM DRbar
Block yu Q= 8
    1.4e-01
Block yd Q= 50
    1.4e-01
Block yf
    9.97405356e-02   # Ytau(Q)MSSM DRbar
Block flup Q= 4.64649125e+03
    9.97405356e-03   # Ytau(Q)MSSM DRbar
    ";

    #[test]
    fn block_single() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: BlockSingle<f64>,
            yd: BlockSingle<f64>,
            yf: BlockSingle<f64>,
        }

        let err = MySlha::deserialize(INPUT_SINGLE).unwrap_err();
        if let Error(ErrorKind::DuplicateBlock(name), _) = err {
            assert_eq!(&name, BLOCK);
        } else {
            panic!("Wrong error variant {:?} instead of DuplicateBlock", err);
        }
    }
}

mod duplicate_key_in_block {
    use slha;
    use slha::{SlhaDeserialize, Block, BlockSingle, BlockStr};
    use slha::modifier::{VecUnchecked, TakeFirst, TakeLast};
    use slha::errors::*;
    use error_chain::ChainedError;

    const INPUT: &'static str = "\
# SUSY Les Houches Accord 1.0 - example input file
# Snowmsas point 1a
Block MODSEL  # Select model
     1    1   # sugra
Block SMINPUTS   # Standard Model inputs
     6      0.1172  # alpha_s(MZ) SM MSbar
     5      1.23    # Mb(mb) SM MSbar
     6    174.3     # Mtop(pole)
Block MINPAR  # SUSY breaking input parameters
     3     10.0     # tanb
     4      1.0     # sign(mu)
     1    100.0     # m0
     2    250.0     # m12
     5   -100.0     # A0 ";
    const BLOCK: &'static str = "sminputs";

    test_invalid_block_1!(block => BLOCK, INPUT);
    test_invalid_block_1!(option => BLOCK, INPUT);
    test_invalid_block_1!(vec => BLOCK, INPUT);
    test_invalid_block_1!(vec_unchecked => BLOCK, INPUT);
    test_invalid_block_1!(take_first => BLOCK, INPUT);
    test_invalid_block_1!(take_last => BLOCK, INPUT);
    test_invalid_block_1!(block_str => BLOCK, INPUT);


    const INPUT_SINGLE: &'static str = "\
Block MODSEL  # Select model
     1   # sugra
Block SMINPUTS   # Standard Model inputs
     0.1172  # alpha_s(MZ) SM MSbar
     1.23    # Mb(mb) SM MSbar
Block MINPAR  # SUSY breaking input parameters
     10.0     # tanb
    ";

    test_invalid_block_1!(block_single => BLOCK, INPUT_SINGLE);
}

mod missing_block {
    use slha;
    use slha::{SlhaDeserialize, Block, BlockSingle, BlockStr};
    use slha::modifier::{VecUnchecked, TakeFirst, TakeLast};
    use slha::errors::*;

    const INPUT: &'static str = "\
Block yf Q= 4.64649125e+02
    3  3 8.88193465e-01   # Yt(Q)MSSM DRbar
Block yr Q= 8
    3  3 1.4e-01
Block yd Q= 50
    3  3 1.4e-01
Block yq Q= 9.3
    3  3 9.97405356e-02   # Ytau(Q)MSSM DRbar
Block flup Q= 4.64649125e+03
    3  3 9.97405356e-03   # Ytau(Q)MSSM DRbar
    ";
    const BLOCK: &'static str = "yu";

    #[test]
    fn block() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: Block<(i8, i8), f64>,
            yd: Block<(i8, i8), f64>,
            yf: Block<(i8, i8), f64>,
        }

        let err = MySlha::deserialize(INPUT).unwrap_err();
        if let Error(ErrorKind::MissingBlock(name), _) = err {
            assert_eq!(&name, BLOCK);
        } else {
            panic!("Wrong error variant {:?} instead of MissingBlock", err);
        }
    }

    #[test]
    fn option() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: Option<Block<(i8, i8), f64>>,
            yd: Option<Block<(i8, i8), f64>>,
            yf: Option<Block<(i8, i8), f64>>,
        }

        let slha = MySlha::deserialize(INPUT).unwrap();
        assert_eq!(slha.yu, None);
    }

    #[test]
    fn vec() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: Vec<Block<(i8, i8), f64>>,
            yd: Vec<Block<(i8, i8), f64>>,
            yf: Vec<Block<(i8, i8), f64>>,
        }

        let slha = MySlha::deserialize(INPUT).unwrap();
        assert_eq!(slha.yu.len(), 0);
    }

    #[test]
    fn vec_unchecked() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: VecUnchecked<Block<(i8, i8), f64>>,
            yd: VecUnchecked<Block<(i8, i8), f64>>,
            yf: VecUnchecked<Block<(i8, i8), f64>>,
        }

        let slha = MySlha::deserialize(INPUT).unwrap();
        assert_eq!(slha.yu.len(), 0);
    }

    #[test]
    fn take_first() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: TakeFirst<Block<(i8, i8), f64>>,
            yd: TakeFirst<Block<(i8, i8), f64>>,
            yf: TakeFirst<Block<(i8, i8), f64>>,
        }

        let err = MySlha::deserialize(INPUT).unwrap_err();
        if let Error(ErrorKind::MissingBlock(name), _) = err {
            assert_eq!(&name, BLOCK);
        } else {
            panic!("Wrong error variant {:?} instead of MissingBlock", err);
        }
    }

    #[test]
    fn take_last() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: TakeLast<Block<(i8, i8), f64>>,
            yd: TakeLast<Block<(i8, i8), f64>>,
            yf: TakeLast<Block<(i8, i8), f64>>,
        }

        let err = MySlha::deserialize(INPUT).unwrap_err();
        if let Error(ErrorKind::MissingBlock(name), _) = err {
            assert_eq!(&name, BLOCK);
        } else {
            panic!("Wrong error variant {:?} instead of MissingBlock", err);
        }
    }

    #[test]
    fn block_str() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: BlockStr<f64>,
            yd: BlockStr<f64>,
            yf: BlockStr<f64>,
        }

        let err = MySlha::deserialize(INPUT).unwrap_err();
        if let Error(ErrorKind::MissingBlock(name), _) = err {
            assert_eq!(&name, BLOCK);
        } else {
            panic!("Wrong error variant {:?} instead ofMissingBlock", err);
        }
    }

    const INPUT_SINGLE: &'static str = "\
Block yf Q= 4.64649125e+02
    8.88193465e-01   # Yt(Q)MSSM DRbar
Block yr Q= 8
    1.4e-01
Block yd Q= 50
    1.4e-01
Block yq Q= 9.32
    9.97405356e-02   # Ytau(Q)MSSM DRbar
Block flup Q= 4.64649125e+03
    9.97405356e-03   # Ytau(Q)MSSM DRbar
    ";

    #[test]
    fn block_single() {
        #[derive(Debug, SlhaDeserialize)]
        struct MySlha {
            yu: BlockSingle<f64>,
            yd: BlockSingle<f64>,
            yf: BlockSingle<f64>,
        }

        let err = MySlha::deserialize(INPUT_SINGLE).unwrap_err();
        if let Error(ErrorKind::MissingBlock(name), _) = err {
            assert_eq!(&name, BLOCK);
        } else {
            panic!("Wrong error variant {:?} instead of MissingBlock", err);
        }
    }
}
