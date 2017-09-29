#![cfg(test)]

extern crate slha;
#[macro_use]
extern crate slha_derive;

use std::collections::HashMap;
use slha::{Block, SlhaDeserialize, DecayTable, Decay};

#[test]
fn test_derive_basic() {

    #[derive(SlhaDeserialize)]
    struct Foo {
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

    let slha = MySlha::deserialize(input);
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
    let slha = MySlha::deserialize(input);
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

    let slha = MySlha::deserialize(input);
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
