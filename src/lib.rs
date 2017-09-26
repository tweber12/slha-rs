#[macro_use]
extern crate nom;
use nom::{double, line_ending, not_line_ending};

use std::collections::HashMap;
use std::hash::Hash;
use std::i64;
use std::str;
use std::str::FromStr;

#[derive(Debug)]
pub struct NamedBlock<'input> {
    name: String,
    block: AnyBlock<'input>,
}

#[derive(Debug)]
pub enum AnyBlock<'input> {
    II(Block<'input, i64, i64>),
    IF(Block<'input, i64, f64>),
    IIF(Block<'input, (i64, i64), f64>),
    IIQ(f64, Block<'input, i64, i64>),
    IFQ(f64, Block<'input, i64, f64>),
    IIFQ(f64, Block<'input, (i64, i64), f64>),
    Alpha(Payload<'input, f64>),
    IS(Block<'input, i64, &'input [u8]>),
    Unknown(Vec<Payload<'input, &'input [u8]>>),
}

pub type Block<'input, Key, Value> = HashMap<Key, Payload<'input, Value>>;

pub struct SLHA<'a> {
    blocks: Vec<NamedBlock<'a>>,
}

named!(parse_slha<SLHA>,
    do_parse!(
        blocks: many0!(parse_anyblock) >>
        (SLHA { blocks })
    )
);

named!(parse_anyblock<NamedBlock>, alt!(block_modsel | block_sminputs | block_minpar | block_extpar | block_mass | block_nmix
    | block_umix | block_vmix | block_stopmix | block_sbotmix | block_staumix | block_alpha | block_hmix | block_gauge
    | block_msoft | block_au | block_ad | block_ae | block_yu | block_yd | block_ye | block_spinfo | unknown_block));

named!(block_modsel<NamedBlock>, apply!(block_ii, "modsel"));
named!(block_sminputs<NamedBlock>, apply!(block_if, "sminputs"));
named!(block_minpar<NamedBlock>, apply!(block_if, "minpar"));
named!(block_extpar<NamedBlock>, apply!(block_if, "extpar"));
named!(block_mass<NamedBlock>, apply!(block_if, "mass"));
named!(block_nmix<NamedBlock>, apply!(block_iif, "nmix"));
named!(block_umix<NamedBlock>, apply!(block_iif, "umix"));
named!(block_vmix<NamedBlock>, apply!(block_iif, "vmix"));
named!(block_stopmix<NamedBlock>, apply!(block_iif, "stopmix"));
named!(block_sbotmix<NamedBlock>, apply!(block_iif, "sbotmix"));
named!(block_staumix<NamedBlock>, apply!(block_iif, "staumix"));
named!(block_alpha<NamedBlock>,
    do_parse!(
        ws!(tag_no_case!("block")) >>
        tag_no_case!("alpha") >>
        eol_or_eof >>
        line: ws!(parse_line_f) >>
        (NamedBlock { name: "alpha".to_string(), block: AnyBlock::Alpha(line) })
        )
);
named!(block_hmix<NamedBlock>, apply!(block_q_if, "hmix"));
named!(block_gauge<NamedBlock>, apply!(block_q_if, "gauge"));
named!(block_msoft<NamedBlock>, apply!(block_q_if, "msoft"));
named!(block_au<NamedBlock>, apply!(block_q_iif, "au"));
named!(block_ad<NamedBlock>, apply!(block_q_iif, "ad"));
named!(block_ae<NamedBlock>, apply!(block_q_iif, "ae"));
named!(block_yu<NamedBlock>, apply!(block_q_iif, "yu"));
named!(block_yd<NamedBlock>, apply!(block_q_iif, "yd"));
named!(block_ye<NamedBlock>, apply!(block_q_iif, "ye"));
named!(block_spinfo<NamedBlock>, apply!(block_is, "spinfo"));

named!(unknown_block<NamedBlock>,
    do_parse!(
        ws!(tag_no_case!("block")) >>
        name: parse_unknown_block_name >>
        parse_opt_comment >>
        eol_or_eof >>
        skip_lines >>
        lines: many0!(preceded!(skip_lines, parse_line_s)) >>
        (NamedBlock { name, block: AnyBlock::Unknown(lines) })
    )
);
named!(parse_unknown_block_name<String>,
    map_res!(
        take_till1_s!(end_of_string),
        |n: &[u8]| str::from_utf8(n).map(|n| n.trim().to_lowercase())
    )
);

fn parse_block<'input, K: ::std::hash::Hash + Eq, T>(
    input: &'input [u8],
    name: &str,
    parse_line: fn(&'input [u8]) -> nom::IResult<&'input [u8], (K, Payload<'input, T>)>,
    fun: fn(Block<'input, K, T>) -> AnyBlock,
) -> nom::IResult<&'input [u8], NamedBlock<'input>> {
    do_parse!(input,
        ws!(tag_no_case!("block")) >>
        tag_no_case!(name) >>
        skip_whitespace >>
        parse_opt_comment >>
        eol_or_eof >>
        lines: apply!(parse_block_body, parse_line) >>
        (NamedBlock { name: name.to_string(), block: fun(lines) })
    )
}

fn parse_block_q<'input, K: Hash + Eq, T>(
    input: &'input [u8],
    name: &str,
    parse_line: fn(&'input [u8]) -> nom::IResult<&'input [u8], (K, Payload<'input, T>)>,
    fun: fn(f64, Block<'input, K, T>) -> AnyBlock,
) -> nom::IResult<&'input [u8], NamedBlock<'input>> {
    do_parse!(input,
        ws!(tag_no_case!("block")) >>
        ws!(tag_no_case!(name)) >>
        ws!(tag_no_case!("q")) >>
        ws!(tag_no_case!("=")) >>
        scale: double >>
        skip_whitespace >>
        parse_opt_comment >>
        eol_or_eof >>
        lines: apply!(parse_block_body, parse_line) >>
        (NamedBlock { name: name.to_string(), block: fun(scale, lines) })
    )
}

fn parse_block_body<'input, K: Hash + Eq, T>(
    input: &'input [u8],
    parse_line: fn(&'input [u8]) -> nom::IResult<&'input [u8], (K, Payload<'input, T>)>,
) -> nom::IResult<&'input [u8], Block<'input, K, T>> {
    do_parse!(input,
        skip_lines >>
        lines: many0!(preceded!(skip_lines, ws!(parse_line))) >>
        (lines.into_iter().collect())
    )
}

fn block_ii<'input>(
    input: &'input [u8],
    name: &str,
) -> nom::IResult<&'input [u8], NamedBlock<'input>> {
    parse_block(input, name, parse_line_ii, AnyBlock::II)
}

fn block_if<'input>(
    input: &'input [u8],
    name: &str,
) -> nom::IResult<&'input [u8], NamedBlock<'input>> {
    parse_block(input, name, parse_line_if, AnyBlock::IF)
}

fn block_q_if<'input>(
    input: &'input [u8],
    name: &str,
) -> nom::IResult<&'input [u8], NamedBlock<'input>> {
    parse_block_q(input, name, parse_line_if, AnyBlock::IFQ)
}

fn block_iif<'input>(
    input: &'input [u8],
    name: &str,
) -> nom::IResult<&'input [u8], NamedBlock<'input>> {
    parse_block(input, name, parse_line_iif, AnyBlock::IIF)
}

fn block_q_iif<'input>(
    input: &'input [u8],
    name: &str,
) -> nom::IResult<&'input [u8], NamedBlock<'input>> {
    parse_block_q(input, name, parse_line_iif, AnyBlock::IIFQ)
}

fn block_is<'input>(
    input: &'input [u8],
    name: &str,
) -> nom::IResult<&'input [u8], NamedBlock<'input>> {
    parse_block(input, name, parse_line_is, AnyBlock::IS)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Payload<'a, T> {
    value: T,
    comment: Option<&'a [u8]>,
}

named!(parse_line_f<Payload<f64>>,
    do_parse!(
        skip_whitespace >>
        value: double >>
        skip_whitespace >>
        comment: parse_opt_comment >>
        eol_or_eof >>
        (Payload { value, comment: comment })
    )
);

named!(parse_line_if<(i64, Payload<f64>)>,
    do_parse!(
        int: ws!(parse_i64) >>
        value: double >>
        skip_whitespace >>
        comment: parse_opt_comment >>
        eol_or_eof >>
        ((int, Payload { value, comment }))
    )
);

named!(parse_line_ii<(i64, Payload<i64>)>,
    do_parse!(
        int: ws!(parse_i64) >>
        value: parse_i64 >>
        skip_whitespace >>
        comment: parse_opt_comment >>
        eol_or_eof >>
        ((int, Payload { value, comment }))
    )
);

named!(parse_line_iif<((i64, i64), Payload<f64>)>,
    do_parse!(
        int1: ws!(parse_i64) >>
        int2: ws!(parse_i64) >>
        value: double >>
        skip_whitespace >>
        comment: parse_opt_comment >>
        eol_or_eof >>
        ((int1, int2), Payload { value, comment })
    )
);

fn end_of_string(c: u8) -> bool {
    c == b'#' || c == b'\n' || c == b'\r'
}

named!(parse_line_is<(i64, Payload<&[u8]>)>,
    do_parse!(
        int: ws!(parse_i64) >>
        value: take_till1_s!(end_of_string) >>
        comment: parse_opt_comment >>
        eol_or_eof >>
        ((int, Payload { value, comment }))
    )
);

named!(parse_line_s<Payload<&[u8]>>,
    do_parse!(
        value: take_till1_s!(end_of_string) >>
        comment: parse_opt_comment >>
        eol_or_eof >>
        (Payload { value, comment })
    )
);

named!(parse_i64<i64>,
    do_parse!(
        sign: opt!(alt!(
            tag!("+") => {|_| 1} |
            tag!("-") => {|_| -1}
        )) >>
        val: parse_digits_i >>
        (sign.unwrap_or(1) * (val as i64))
    )
);

named!(parse_digits_i<i64>,
    map_res!(
        map_res!(
            take_while1!(nom::is_digit),
            str::from_utf8
        ),
        i64::from_str
    )
);

named!(parse_opt_comment<Option<&[u8]>>,
    opt!(complete!(parse_comment))
);

named!(parse_comment<&[u8]>,
    do_parse!(
        tag!("#") >>
        comment: not_line_ending >>
        (comment)
    )
);

named!(eol_or_eof, alt!(eof!() | complete!(line_ending)));

named!(skip_whitespace, take_while!(nom::is_space));

named!(blank_line<()>, do_parse!(skip_whitespace >> eol_or_eof >> (())));
named!(comment_line<()>, do_parse!(skip_whitespace >> parse_comment >> eol_or_eof >> (())));
named!(skip_lines<Vec<()>>, many0!(alt!(blank_line | comment_line)));

#[cfg(test)]
mod tests {
    use nom;
    use nom::IResult;
    use super::{parse_slha, block_if, block_ii, block_iif, block_is, block_q_if, block_q_iif,
                unknown_block, parse_anyblock, parse_comment, parse_opt_comment, parse_line_f,
                parse_line_if, parse_line_ii, parse_line_iif, parse_line_is, parse_line_s,
                AnyBlock, Payload};

    macro_rules! test_iresult {
        ($result:expr, $expected:expr) => {
            match $result {
                i @ IResult::Incomplete(..) => panic!("Incomplete parse: {:?} (Expected {:?})", i, $expected),
                e @ IResult::Error(..) => panic!("Parse error: {:?} (Expected {:?})", e, $expected),
                IResult::Done(rest, _) if rest != &b""[..] => panic!("The parser did not consume the full input. {:?} was left over. (Expected {:?})", rest, $expected),
                IResult::Done(_, result) => assert_eq!(result, $expected),
            }
        }
    }

    macro_rules! unwrap_anyblock {
        ($result:expr, $variant:ident) => {{
            let result = match $result {
                i @ IResult::Incomplete(..) => panic!("Incomplete parse: {:?}", i),
                e @ IResult::Error(..) => panic!("Parse error: {:?}", e),
                IResult::Done(rest, _) if rest != &b""[..] => panic!("The parser did not consume the full input. {:?} was left over.)", rest),
                IResult::Done(_, result) => result,
            };
            if let AnyBlock::$variant(res) = result.block {
                (result.name, res)
            } else {
                panic!("Wrong AnyResult variant {:?}", result);
            }
        }}
    }

    macro_rules! unwrap_anyblock_q {
        ($result:expr, $variant:ident) => {{
            let result = match $result {
                i @ IResult::Incomplete(..) => panic!("Incomplete parse: {:?}", i),
                e @ IResult::Error(..) => panic!("Parse error: {:?}", e),
                IResult::Done(rest, _) if rest != &b""[..] => panic!("The parser did not consume the full input. {:?} was left over.)", rest),
                IResult::Done(_, result) => result,
            };
            if let AnyBlock::$variant(q,res) = result.block {
                (result.name, q, res)
            } else {
                panic!("Wrong AnyResult variant {:?}", result);
            }
        }}
    }

    #[test]
    fn test_parse_comment() {
        assert_eq!(parse_opt_comment(b"# foo\n").unwrap().1, Some(b" foo".as_ref()));
        assert_eq!(parse_opt_comment(b"").unwrap().1, None);
        assert_eq!(parse_comment(b"# foo\n").unwrap().1, b" foo");
        assert_eq!(parse_comment(b"# foo").unwrap().1, b" foo");
    }

    #[test]
    fn test_parse_line_f() {
        test_iresult!(parse_line_f(b"78.129\n"), Payload { value: 78.129, comment: None });
        test_iresult!(parse_line_f(b"    68.129\n"), Payload { value: 68.129, comment: None });
        test_iresult!(parse_line_f(b"77.127    \n"), Payload { value: 77.127, comment: None });
        test_iresult!(parse_line_f(b"  76.129    \n"), Payload { value: 76.129, comment: None });
        test_iresult!(parse_line_f(b"75.129"), Payload { value: 75.129, comment: None });
        test_iresult!(parse_line_f(b"  74.129"), Payload { value: 74.129, comment: None });
        test_iresult!(parse_line_f(b"73.129  "), Payload { value: 73.129, comment: None });
        test_iresult!(parse_line_f(b"  73.119  "), Payload { value: 73.119, comment: None });
        test_iresult!(parse_line_f(b"78.129# foo\n"), Payload { value: 78.129, comment: Some(b" foo".as_ref()) });
        test_iresult!(parse_line_f(b"    68.129# foo\n"), Payload { value: 68.129, comment: Some(b" foo".as_ref()) });
        test_iresult!(parse_line_f(b"77.127    # foo\n"), Payload { value: 77.127, comment: Some(b" foo".as_ref()) });
        test_iresult!(parse_line_f(b"  76.129    # foo\n"), Payload { value: 76.129, comment: Some(b" foo".as_ref()) });
        test_iresult!(parse_line_f(b"75.129# foo"), Payload { value: 75.129, comment: Some(b" foo".as_ref()) });
        test_iresult!(parse_line_f(b"  74.129# foo"), Payload { value: 74.129, comment: Some(b" foo".as_ref()) });
        test_iresult!(parse_line_f(b"73.129  # foo"), Payload { value: 73.129, comment: Some(b" foo".as_ref()) });
        test_iresult!(parse_line_f(b"  73.119  # foo"), Payload { value: 73.119, comment: Some(b" foo".as_ref()) });
        test_iresult!(parse_line_f(b"1.0e-9\n"), Payload { value: 1e-9, comment: None });
    }

    #[test]
    fn test_parse_line_if() {
        test_iresult!(parse_line_if(b"12 78.129\n"), (12, Payload { value: 78.129, comment: None }));
        test_iresult!(parse_line_if(b"    12   68.129\n"), (12, Payload { value: 68.129, comment: None }));
        test_iresult!(parse_line_if(b"12 77.127    \n"), (12, Payload { value: 77.127, comment: None }));
        test_iresult!(parse_line_if(b"  22   76.129    \n"), (22, Payload { value: 76.129, comment: None }));
        test_iresult!(parse_line_if(b"12   75.129"), (12, Payload { value: 75.129, comment: None }));
        test_iresult!(parse_line_if(b"  12 74.129"), (12, Payload { value: 74.129, comment: None }));
        test_iresult!(parse_line_if(b"12 73.129  "), (12, Payload { value: 73.129, comment: None }));
        test_iresult!(parse_line_if(b"  12   73.119  "), (12, Payload { value: 73.119, comment: None }));
        test_iresult!(parse_line_if(b"12 78.129# foo\n"), (12, Payload { value: 78.129, comment: Some(b" foo".as_ref()) }));
        test_iresult!(parse_line_if(b"    12 68.129# foo\n"), (12, Payload { value: 68.129, comment: Some(b" foo".as_ref()) }));
        test_iresult!(parse_line_if(b"12   77.127    # foo\n"), (12, Payload { value: 77.127, comment: Some(b" foo".as_ref()) }));
        test_iresult!(parse_line_if(b"  16 76.129    # foo\n"), (16, Payload { value: 76.129, comment: Some(b" foo".as_ref()) }));
        test_iresult!(parse_line_if(b"12   75.129# foo"), (12, Payload { value: 75.129, comment: Some(b" foo".as_ref()) }));
        test_iresult!(parse_line_if(b"  12 74.129# foo"), (12, Payload { value: 74.129, comment: Some(b" foo".as_ref()) }));
        test_iresult!(parse_line_if(b"12   73.129  # foo"), (12, Payload { value: 73.129, comment: Some(b" foo".as_ref()) }));
        test_iresult!(parse_line_if(b"  12 73.119  # foo"), (12, Payload { value: 73.119, comment: Some(b" foo".as_ref()) }));
    }

    #[test]
    fn test_parse_line_ii() {
        test_iresult!(parse_line_ii(b"12 78\n"), (12, Payload { value: 78, comment: None }));
        test_iresult!(parse_line_ii(b"    12   68\n"), (12, Payload { value: 68, comment: None }));
        test_iresult!(parse_line_ii(b"12 77    \n"), (12, Payload { value: 77, comment: None }));
        test_iresult!(parse_line_ii(b"  22   76    \n"), (22, Payload { value: 76, comment: None }));
        test_iresult!(parse_line_ii(b"12   75"), (12, Payload { value: 75, comment: None }));
        test_iresult!(parse_line_ii(b"  12 74"), (12, Payload { value: 74, comment: None }));
        test_iresult!(parse_line_ii(b"12 73  "), (12, Payload { value: 73, comment: None }));
        test_iresult!(parse_line_ii(b"  12   73  "), (12, Payload { value: 73, comment: None }));
        test_iresult!(parse_line_ii(b"12 78# foo\n"), (12, Payload { value: 78, comment: Some(b" foo".as_ref()) }));
        test_iresult!(parse_line_ii(b"    12 68# foo\n"), (12, Payload { value: 68, comment: Some(b" foo".as_ref()) }));
        test_iresult!(parse_line_ii(b"12   77    # foo\n"), (12, Payload { value: 77, comment: Some(b" foo".as_ref()) }));
        test_iresult!(parse_line_ii(b"  16 76    # foo\n"), (16, Payload { value: 76, comment: Some(b" foo".as_ref()) }));
        test_iresult!(parse_line_ii(b"12   75# bar"), (12, Payload { value: 75, comment: Some(b" bar".as_ref()) }));
        test_iresult!(parse_line_ii(b"  12 74# foo"), (12, Payload { value: 74, comment: Some(b" foo".as_ref()) }));
        test_iresult!(parse_line_ii(b"12   73  # comment"), (12, Payload { value: 73, comment: Some(b" comment".as_ref()) }));
        test_iresult!(parse_line_ii(b"  12 73  # foo"), (12, Payload { value: 73, comment: Some(b" foo".as_ref()) }));
    }

    #[test]
    fn test_parse_line_iif() {
        test_iresult!(parse_line_iif(b"12  9 78.129\n"), ((12,9), Payload { value: 78.129, comment: None }));
        test_iresult!(parse_line_iif(b"   4 12   68.129\n"), ((4,12), Payload { value: 68.129, comment: None }));
        test_iresult!(parse_line_iif(b"12 1 77.127    \n"), ((12,1), Payload { value: 77.127, comment: None }));
        test_iresult!(parse_line_iif(b"  22 5  76.129    \n"), ((22,5), Payload { value: 76.129, comment: None }));
        test_iresult!(parse_line_iif(b"12 13  75.129"), ((12,13), Payload { value: 75.129, comment: None }));
        test_iresult!(parse_line_iif(b"  12 8 74.129"), ((12,8), Payload { value: 74.129, comment: None }));
        test_iresult!(parse_line_iif(b"12  3    73.129  "), ((12,3), Payload { value: 73.129, comment: None }));
        test_iresult!(parse_line_iif(b"  12 1  73.119  "), ((12,1), Payload { value: 73.119, comment: None }));
        test_iresult!(parse_line_iif(b"12 2 78.129# foo\n"), ((12,2), Payload { value: 78.129, comment: Some(b" foo".as_ref()) }));
        test_iresult!(parse_line_iif(b"    4   12 68.129# foo\n"), ((4,12), Payload { value: 68.129, comment: Some(b" foo".as_ref()) }));
        test_iresult!(parse_line_iif(b"12 5  77.127    # foo\n"), ((12,5), Payload { value: 77.127, comment: Some(b" foo".as_ref()) }));
        test_iresult!(parse_line_iif(b"  16 6 76.129    # bar\n"), ((16,6), Payload { value: 76.129, comment: Some(b" bar".as_ref()) }));
        test_iresult!(parse_line_iif(b"7 12   75.129# bar"), ((7,12), Payload { value: 75.129, comment: Some(b" bar".as_ref()) }));
        test_iresult!(parse_line_iif(b"  12        8 74.129# baz"), ((12,8), Payload { value: 74.129, comment: Some(b" baz".as_ref()) }));
        test_iresult!(parse_line_iif(b"9 12   73.129  # foo"), ((9,12), Payload { value: 73.129, comment: Some(b" foo".as_ref()) }));
        test_iresult!(parse_line_iif(b"  12     10 73.119  # foo"), ((12,10), Payload { value: 73.119, comment: Some(b" foo".as_ref()) }));
    }

    #[test]
    fn test_parse_line_is() {
        test_iresult!(parse_line_is(b"12 There is a string here\n"), (12, Payload { value: &b"There is a string here"[..], comment: None }));
        test_iresult!(parse_line_is(b"    12   More strings\n"), (12, Payload { value: &b"More strings"[..], comment: None }));
        test_iresult!(parse_line_is(b"12 Version number    \n"), (12, Payload { value: &b"Version number    "[..], comment: None }));
        test_iresult!(parse_line_is(b"  22   3.4.8    \n"), (22, Payload { value: &b"3.4.8    "[..], comment: None }));
        test_iresult!(parse_line_is(b"12   String"), (12, Payload { value: &b"String"[..], comment: None }));
        test_iresult!(parse_line_is(b"  12 String"), (12, Payload { value: &b"String"[..], comment: None }));
        test_iresult!(parse_line_is(b"12 String  "), (12, Payload { value: &b"String  "[..], comment: None }));
        test_iresult!(parse_line_is(b"  12   String  "), (12, Payload { value: &b"String  "  [..], comment: None }));
        test_iresult!(parse_line_is(b"12 String# foo\n"), (12, Payload { value: &b"String"[..], comment: Some(b" foo".as_ref()) }));
        test_iresult!(parse_line_is(b"    12 String# foo\n"), (12, Payload { value: &b"String"[..], comment: Some(b" foo".as_ref()) }));
        test_iresult!(parse_line_is(b"12   String    # foo\n"), (12, Payload { value: &b"String    "[..], comment: Some(b" foo".as_ref()) }));
        test_iresult!(parse_line_is(b"  16 String    # foo\n"), (16, Payload { value: &b"String    "[..], comment: Some(b" foo".as_ref()) }));
        test_iresult!(parse_line_is(b"12   String# bar"), (12, Payload { value: &b"String"[..], comment: Some(b" bar".as_ref()) }));
        test_iresult!(parse_line_is(b"  12 String# foo"), (12, Payload { value: &b"String"[..], comment: Some(b" foo".as_ref()) }));
        test_iresult!(parse_line_is(b"12   Not comment String  # comment"), (12, Payload { value: &b"Not comment String  "[..], comment: Some(b" comment".as_ref()) }));
        test_iresult!(parse_line_is(b"  12 String  # foo"), (12, Payload { value: &b"String  "[..], comment: Some(b" foo".as_ref()) }));
    }

    #[test]
    fn test_parse_line_s() {
        test_iresult!(parse_line_s(b" There is a string here\n"), Payload { value: &b" There is a string here"[..], comment: None });
        test_iresult!(parse_line_s(b"       More strings\n"), Payload { value: &b"       More strings"[..], comment: None });
        test_iresult!(parse_line_s(b" Version number    \n"), Payload { value: &b" Version number    "[..], comment: None });
        test_iresult!(parse_line_s(b"     3.4.8    \n"), Payload { value: &b"     3.4.8    "[..], comment: None });
        test_iresult!(parse_line_s(b"   String"), Payload { value: &b"   String"[..], comment: None });
        test_iresult!(parse_line_s(b"   String"), Payload { value: &b"   String"[..], comment: None });
        test_iresult!(parse_line_s(b" String  "), Payload { value: &b" String  "[..], comment: None });
        test_iresult!(parse_line_s(b"     String  "), Payload { value: &b"     String  "  [..], comment: None });
        test_iresult!(parse_line_s(b" String# foo\n"), Payload { value: &b" String"[..], comment: Some(b" foo".as_ref()) });
        test_iresult!(parse_line_s(b"     String# foo\n"), Payload { value: &b"     String"[..], comment: Some(b" foo".as_ref()) });
        test_iresult!(parse_line_s(b"   String    # foo\n"), Payload { value: &b"   String    "[..], comment: Some(b" foo".as_ref()) });
        test_iresult!(parse_line_s(b"   String    # foo\n"), Payload { value: &b"   String    "[..], comment: Some(b" foo".as_ref()) });
        test_iresult!(parse_line_s(b"tring# bar"), Payload { value: &b"tring"[..], comment: Some(b" bar".as_ref()) });
        test_iresult!(parse_line_s(b"   String# foo"), Payload { value: &b"   String"[..], comment: Some(b" foo".as_ref()) });
        test_iresult!(parse_line_s(b"  Not comment String  # comment"), Payload { value: &b"  Not comment String  "[..], comment: Some(b" comment".as_ref()) });
        test_iresult!(parse_line_s(b"This   String  # foo"), Payload { value: &b"This   String  "[..], comment: Some(b" foo".as_ref()) });
    }

    #[test]
    fn test_block_if() {
        let (name, result) = unwrap_anyblock!(block_if(b"BLOCK TEST", "test"), IF);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_if(b"BLOCK TEST\n\n\n", "test"), IF);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_if(b"BLOCK TEST     ", "test"), IF);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_if(b"BLOCK TEST     \n", "test"), IF);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_if(b"BLOCK TEST     # foo\n", "test"), IF);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_if(b"BLOCK TEST     # foo", "test"), IF);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_if(b"BLOCK TEST     # foo    \n", "test"), IF);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_if(b"BLOCK TEST     # foo      ", "test"), IF);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_if(b"BLOCK TEST     # foo      \n  12 5.4", "test"), IF);
        assert_eq!(name, "test");
        assert_eq!(result.len(), 1);
        assert_eq!(result[&12], Payload { value: 5.4, comment: None });

        let (name, result) = unwrap_anyblock!(block_if(b"BLOCK TEST     # foo      \n # Pre comment \n 12 5.4", "test"), IF);
        assert_eq!(name, "test");
        assert_eq!(result.len(), 1);
        assert_eq!(result[&12], Payload { value: 5.4, comment: None });

        let (name, result) = unwrap_anyblock!(block_if(b"BLOCK TEST     # foo      \n # Pre comment \n 12 5.4\n # Post comment\n 13 8.93", "test"), IF);
        assert_eq!(name, "test");
        assert_eq!(result.len(), 2);
        assert_eq!(result[&12], Payload { value: 5.4, comment: None });
        assert_eq!(result[&13], Payload { value: 8.93, comment: None });

        let (name, result) = unwrap_anyblock!(block_if(b"BLOCK TEST     # foo      \n # Pre comment \n 12 5.4\n # Post comment\n    \n#Another comment\n 13 8.93   # Trailing comment", "test"), IF);
        assert_eq!(name, "test");
        assert_eq!(result.len(), 2);
        assert_eq!(result[&12], Payload { value: 5.4, comment: None });
        assert_eq!(result[&13], Payload { value: 8.93, comment: Some(&b" Trailing comment"[..]) });
    }

    #[test]
    fn test_block_ii() {
        let (name, result) = unwrap_anyblock!(block_ii(b"BLOCK TEST", "test"), II);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_ii(b"BLOCK TEST\n\n\n", "test"), II);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_ii(b"BLOCK TEST     ", "test"), II);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_ii(b"BLOCK TEST     \n", "test"), II);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_ii(b"BLOCK TEST     # foo\n", "test"), II);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_ii(b"BLOCK TEST     # foo", "test"), II);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_ii(b"BLOCK TEST     # foo    \n", "test"), II);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_ii(b"BLOCK TEST     # foo      ", "test"), II);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_ii(b"BLOCK TEST     # foo      \n  12 5", "test"), II);
        assert_eq!(name, "test");
        assert_eq!(result.len(), 1);
        assert_eq!(result[&12], Payload { value: 5, comment: None });

        let (name, result) = unwrap_anyblock!(block_ii(b"BLOCK TEST     # foo      \n # Pre comment \n 12 5", "test"), II);
        assert_eq!(name, "test");
        assert_eq!(result.len(), 1);
        assert_eq!(result[&12], Payload { value: 5, comment: None });

        let (name, result) = unwrap_anyblock!(block_ii(b"BLOCK TEST     # foo      \n # Pre comment \n 12 5\n # Post comment\n 13 8", "test"), II);
        assert_eq!(name, "test");
        assert_eq!(result.len(), 2);
        assert_eq!(result[&12], Payload { value: 5, comment: None });
        assert_eq!(result[&13], Payload { value: 8, comment: None });

        let (name, result) = unwrap_anyblock!(block_ii(b"BLOCK TEST     # foo      \n # Pre comment \n 12 5\n # Post comment\n    \n#Another comment\n 18 8   # Trailing comment", "test"), II);
        assert_eq!(name, "test");
        assert_eq!(result.len(), 2);
        assert_eq!(result[&12], Payload { value: 5, comment: None });
        assert_eq!(result[&18], Payload { value: 8, comment: Some(&b" Trailing comment"[..]) });
    }

    #[test]
    fn test_block_iif() {
        let (name, result) = unwrap_anyblock!(block_iif(b"BLOCK TEST", "test"), IIF);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_iif(b"BLOCK TEST\n\n\n", "test"), IIF);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_iif(b"BLOCK TEST     ", "test"), IIF);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_iif(b"BLOCK TEST     \n", "test"), IIF);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_iif(b"BLOCK TEST     # foo\n", "test"), IIF);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_iif(b"BLOCK TEST     # foo", "test"), IIF);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_iif(b"BLOCK TEST     # foo    \n", "test"), IIF);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_iif(b"BLOCK TEST     # foo      ", "test"), IIF);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_iif(b"BLOCK TEST     # foo      \n  12 5  9.35", "test"), IIF);
        assert_eq!(name, "test");
        assert_eq!(result.len(), 1);
        assert_eq!(result[&(12,5)], Payload { value: 9.35, comment: None });

        let (name, result) = unwrap_anyblock!(block_iif(b"BLOCK TEST     # foo      \n # Pre comment \n 13 6 -5.2", "test"), IIF);
        assert_eq!(name, "test");
        assert_eq!(result.len(), 1);
        assert_eq!(result[&(13,6)], Payload { value: -5.2, comment: None });

        let (name, result) = unwrap_anyblock!(block_iif(b"BLOCK TEST     # foo      \n # Pre comment \n 12 5  1.0e-9\n # Post comment\n 13 8 4.2e3", "test"), IIF);
        assert_eq!(name, "test");
        assert_eq!(result.len(), 2);
        assert_eq!(result[&(12,5)], Payload { value: 1.0e-9, comment: None });
        assert_eq!(result[&(13,8)], Payload { value: 4.2e3, comment: None });

        let (name, result) = unwrap_anyblock!(block_iif(b"BLOCK TEST     # foo      \n # Pre comment \n 13 6   -78.32\n # Post comment\n    \n#Another comment\n 18 8   4.2e4  # Trailing comment", "test"), IIF);
        assert_eq!(name, "test");
        assert_eq!(result.len(), 2);
        assert_eq!(result[&(13,6)], Payload { value: -78.32, comment: None });
        assert_eq!(result[&(18,8)], Payload { value: 4.2e4, comment: Some(&b" Trailing comment"[..]) });
    }

    #[test]
    fn test_block_is() {
        let (name, result) = unwrap_anyblock!(block_is(b"BLOCK TEST", "test"), IS);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_is(b"BLOCK TEST\n\n\n", "test"), IS);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_is(b"BLOCK TEST     ", "test"), IS);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_is(b"BLOCK TEST     \n", "test"), IS);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_is(b"BLOCK TEST     # foo\n", "test"), IS);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_is(b"BLOCK TEST     # foo", "test"), IS);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_is(b"BLOCK TEST     # foo    \n", "test"), IS);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_is(b"BLOCK TEST     # foo      ", "test"), IS);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(block_is(b"BLOCK TEST     # foo      \n  12 Value", "test"), IS);
        assert_eq!(name, "test");
        assert_eq!(result.len(), 1);
        assert_eq!(result[&12], Payload { value: &b"Value"[..], comment: None });

        let (name, result) = unwrap_anyblock!(block_is(b"BLOCK TEST     # foo      \n # Pre comment \n 12 This is the value", "test"), IS);
        assert_eq!(name, "test");
        assert_eq!(result.len(), 1);
        assert_eq!(result[&12], Payload { value: &b"This is the value"[..], comment: None });

        let (name, result) = unwrap_anyblock!(block_is(b"BLOCK TEST     # foo      \n # Pre comment \n 12 version number?\n # Post comment\n 13 error code", "test"), IS);
        assert_eq!(name, "test");
        assert_eq!(result.len(), 2);
        assert_eq!(result[&12], Payload { value: &b"version number?"[..], comment: None });
        assert_eq!(result[&13], Payload { value: &b"error code"[..], comment: None });

        let (name, result) = unwrap_anyblock!(block_is(b"BLOCK TEST     # foo      \n # Pre comment \n 12 version\n # Post comment\n    \n#Another comment\n 18 here stands an error code   # Trailing comment", "test"), IS);
        assert_eq!(name, "test");
        assert_eq!(result.len(), 2);
        assert_eq!(result[&12], Payload { value: &b"version"[..], comment: None });
        assert_eq!(result[&18], Payload { value: &b"here stands an error code   "[..], comment: Some(&b" Trailing comment"[..]) });
    }

    #[test]
    fn test_block_s() {
        let (name, result) = unwrap_anyblock!(unknown_block(b"BLOCK TEST"), Unknown);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(unknown_block(b"BLOCK foo\n\n\n"), Unknown);
        assert_eq!(name, "foo");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(unknown_block(b"BLOCK super     "), Unknown);
        assert_eq!(name, "super");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(unknown_block(b"BLOCK FOO Bar     \n"), Unknown);
        assert_eq!(name, "foo bar");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(unknown_block(b"BLOCK    TEST     # foo\n"), Unknown);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(unknown_block(b"BLOCK blib blub     # foo"), Unknown);
        assert_eq!(name, "blib blub");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(unknown_block(b"BLOCK TEST     # foo    \n"), Unknown);
        assert_eq!(name, "test");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(unknown_block(b"BLOCK Is this a TEST?     # foo      "), Unknown);
        assert_eq!(name, "is this a test?");
        assert!(result.is_empty());

        let (name, result) = unwrap_anyblock!(unknown_block(b"BLOCK TEST     # foo      \n  12 Value"), Unknown);
        assert_eq!(name, "test");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Payload { value: &b"  12 Value"[..], comment: None });

        let (name, result) = unwrap_anyblock!(unknown_block(b"BLOCK TEST     # foo      \n # Pre comment \n 12 This is the value"), Unknown);
        assert_eq!(name, "test");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Payload { value: &b" 12 This is the value"[..], comment: None });

        let (name, result) = unwrap_anyblock!(unknown_block(b"BLOCK WHY?     # foo      \n # Pre comment \n   12 version number?\n # Post comment\n 13 error code"), Unknown);
        assert_eq!(name, "why?");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Payload { value: &b"   12 version number?"[..], comment: None });
        assert_eq!(result[1], Payload { value: &b" 13 error code"[..], comment: None });

        let (name, result) = unwrap_anyblock!(unknown_block(b"BLOCK TEST     # foo      \n # Pre comment \n 12 version\n # Post comment\n    \n#Another comment\nhere stands an error code   # Trailing comment"), Unknown);
        assert_eq!(name, "test");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Payload { value: &b" 12 version"[..], comment: None });
        assert_eq!(result[1], Payload { value: &b"here stands an error code   "[..], comment: Some(&b" Trailing comment"[..]) });
    }

    #[test]
    fn test_block_q_if() {
        let (name, q, result) = unwrap_anyblock_q!(block_q_if(b"BLOCK TEST Q= 5.4", "test"), IFQ);
        assert_eq!(name, "test");
        assert_eq!(q, 5.4);
        assert!(result.is_empty());

        let (name, q, result) = unwrap_anyblock_q!(block_q_if(b"BLOCK TEST Q= -9.4\n\n\n", "test"), IFQ);
        assert_eq!(name, "test");
        assert_eq!(q, -9.4);
        assert!(result.is_empty());

        let (name, q, result) = unwrap_anyblock_q!(block_q_if(b"BLOCK TEST   Q= -321.913   ", "test"), IFQ);
        assert_eq!(name, "test");
        assert_eq!(q, -321.913);
        assert!(result.is_empty());

        let (name, q, result) = unwrap_anyblock_q!(block_q_if(b"BLOCK TEST  Q= 91.20   \n", "test"), IFQ);
        assert_eq!(name, "test");
        assert_eq!(q, 91.20);
        assert!(result.is_empty());

        let (name, q, result) = unwrap_anyblock_q!(block_q_if(b"BLOCK TEST Q= -321.913      # foo\n", "test"), IFQ);
        assert_eq!(name, "test");
        assert_eq!(q, -321.913);
        assert!(result.is_empty());

        let (name, q, result) = unwrap_anyblock_q!(block_q_if(b"BLOCK TEST    Q= -321.913   # foo", "test"), IFQ);
        assert_eq!(name, "test");
        assert_eq!(q, -321.913);
        assert!(result.is_empty());

        let (name, q, result) = unwrap_anyblock_q!(block_q_if(b"BLOCK TEST  Q= -321.913     # foo    \n", "test"), IFQ);
        assert_eq!(name, "test");
        assert_eq!(q, -321.913);
        assert!(result.is_empty());

        let (name, q, result) = unwrap_anyblock_q!(block_q_if(b"BLOCK TEST      Q= -321.913 # foo      ", "test"), IFQ);
        assert_eq!(name, "test");
        assert_eq!(q, -321.913);
        assert!(result.is_empty());

        let (name, q, result) = unwrap_anyblock_q!(block_q_if(b"BLOCK TEST  Q= -321.913     # foo      \n  12 5.4", "test"), IFQ);
        assert_eq!(name, "test");
        assert_eq!(q, -321.913);
        assert_eq!(result.len(), 1);
        assert_eq!(result[&12], Payload { value: 5.4, comment: None });

        let (name, q, result) = unwrap_anyblock_q!(block_q_if(b"BLOCK TEST  Q= -321.913     # foo      \n # Pre comment \n 12 5.4", "test"), IFQ);
        assert_eq!(name, "test");
        assert_eq!(q, -321.913);
        assert_eq!(result.len(), 1);
        assert_eq!(result[&12], Payload { value: 5.4, comment: None });

        let (name, q, result) = unwrap_anyblock_q!(block_q_if(b"BLOCK TEST     Q= -321.913  # foo      \n # Pre comment \n 12 5.4\n # Post comment\n 13 8.93", "test"), IFQ);
        assert_eq!(name, "test");
        assert_eq!(q, -321.913);
        assert_eq!(result.len(), 2);
        assert_eq!(result[&12], Payload { value: 5.4, comment: None });
        assert_eq!(result[&13], Payload { value: 8.93, comment: None });

        let (name, q, result) = unwrap_anyblock_q!(block_q_if(b"BLOCK TEST    Q= -321.913   # foo      \n # Pre comment \n 12 5.4\n # Post comment\n    \n#Another comment\n 13 8.93   # Trailing comment", "test"), IFQ);
        assert_eq!(name, "test");
        assert_eq!(q, -321.913);
        assert_eq!(result.len(), 2);
        assert_eq!(result[&12], Payload { value: 5.4, comment: None });
        assert_eq!(result[&13], Payload { value: 8.93, comment: Some(&b" Trailing comment"[..]) });
    }

    #[test]
    fn test_block_q_iif() {
        let (name, q, result) = unwrap_anyblock_q!(block_q_iif(b"BLOCK TEST    Q=  723.42", "test"), IIFQ);
        assert_eq!(name, "test");
        assert_eq!(q, 723.42);
        assert!(result.is_empty());

        let (name, q, result) = unwrap_anyblock_q!(block_q_iif(b"BLOCK TEST   Q= 723.42\n\n\n", "test"), IIFQ);
        assert_eq!(name, "test");
        assert_eq!(q, 723.42);
        assert!(result.is_empty());

        let (name, q, result) = unwrap_anyblock_q!(block_q_iif(b"BLOCK TEST   Q= 723.42     ", "test"), IIFQ);
        assert_eq!(name, "test");
        assert_eq!(q, 723.42);
        assert!(result.is_empty());

        let (name, q, result) = unwrap_anyblock_q!(block_q_iif(b"BLOCK TEST   Q= 723.42     \n", "test"), IIFQ);
        assert_eq!(name, "test");
        assert_eq!(q, 723.42);
        assert!(result.is_empty());

        let (name, q, result) = unwrap_anyblock_q!(block_q_iif(b"BLOCK TEST   Q= 723.42     # foo\n", "test"), IIFQ);
        assert_eq!(name, "test");
        assert_eq!(q, 723.42);
        assert!(result.is_empty());

        let (name, q, result) = unwrap_anyblock_q!(block_q_iif(b"BLOCK TEST   Q= 723.42     # foo", "test"), IIFQ);
        assert_eq!(name, "test");
        assert_eq!(q, 723.42);
        assert!(result.is_empty());

        let (name, q, result) = unwrap_anyblock_q!(block_q_iif(b"BLOCK TEST   Q= 723.42     # foo    \n", "test"), IIFQ);
        assert_eq!(name, "test");
        assert_eq!(q, 723.42);
        assert!(result.is_empty());

        let (name, q, result) = unwrap_anyblock_q!(block_q_iif(b"BLOCK TEST   Q= 723.42     # foo      ", "test"), IIFQ);
        assert_eq!(name, "test");
        assert_eq!(q, 723.42);
        assert!(result.is_empty());

        let (name, q, result) = unwrap_anyblock_q!(block_q_iif(b"BLOCK TEST   Q= 723.42     # foo      \n  12 5  9.35", "test"), IIFQ);
        assert_eq!(name, "test");
        assert_eq!(q, 723.42);
        assert_eq!(result.len(), 1);
        assert_eq!(result[&(12,5)], Payload { value: 9.35, comment: None });

        let (name, q, result) = unwrap_anyblock_q!(block_q_iif(b"BLOCK TEST   Q= 723.42     # foo      \n # Pre comment \n 13 6 -5.2", "test"), IIFQ);
        assert_eq!(name, "test");
        assert_eq!(q, 723.42);
        assert_eq!(result.len(), 1);
        assert_eq!(result[&(13,6)], Payload { value: -5.2, comment: None });

        let (name, q, result) = unwrap_anyblock_q!(block_q_iif(b"BLOCK TEST   Q= 723.42     # foo      \n # Pre comment \n 12 5  1.0e-9\n # Post comment\n 13 8 4.2e3", "test"), IIFQ);
        assert_eq!(name, "test");
        assert_eq!(q, 723.42);
        assert_eq!(result.len(), 2);
        assert_eq!(result[&(12,5)], Payload { value: 1.0e-9, comment: None });
        assert_eq!(result[&(13,8)], Payload { value: 4.2e3, comment: None });

        let (name, q, result) = unwrap_anyblock_q!(block_q_iif(b"BLOCK TEST   Q= 723.42     # foo      \n # Pre comment \n 13 6   -78.32\n # Post comment\n    \n#Another comment\n 18 8   4.2e4  # Trailing comment", "test"), IIFQ);
        assert_eq!(name, "test");
        assert_eq!(q, 723.42);
        assert_eq!(result.len(), 2);
        assert_eq!(result[&(13,6)], Payload { value: -78.32, comment: None });
        assert_eq!(result[&(18,8)], Payload { value: 4.2e4, comment: Some(&b" Trailing comment"[..]) });
    }

    #[test]
    fn test_parse_anyblock() {
        let (name, result) = unwrap_anyblock!(parse_anyblock(b"BLOCK MODSEL     # foo      \n # Pre comment \n 12 5\n # Post comment\n 13 8"), II);
        assert_eq!(name, "modsel");
        assert_eq!(result.len(), 2);
        assert_eq!(result[&12], Payload { value: 5, comment: None });
        assert_eq!(result[&13], Payload { value: 8, comment: None });

        let (name, result) = unwrap_anyblock!(parse_anyblock(b"BLOCK sMinPUts     # foo      \n # Pre comment \n 12 5.4\n # Post comment\n    \n#Another comment\n 13 8.93   # Trailing comment"), IF);
        assert_eq!(name, "sminputs");
        assert_eq!(result.len(), 2);
        assert_eq!(result[&12], Payload { value: 5.4, comment: None });
        assert_eq!(result[&13], Payload { value: 8.93, comment: Some(&b" Trailing comment"[..]) });

        let (name, result) = unwrap_anyblock!(parse_anyblock(b"BLOCK mInpar     # foo      \n # Pre comment \n 12 5.4\n # Post comment\n    \n#Another comment\n 13 8.93   # Trailing comment"), IF);
        assert_eq!(name, "minpar");
        assert_eq!(result.len(), 2);
        assert_eq!(result[&12], Payload { value: 5.4, comment: None });
        assert_eq!(result[&13], Payload { value: 8.93, comment: Some(&b" Trailing comment"[..]) });

        let (name, result) = unwrap_anyblock!(parse_anyblock(b"BLOCK extPAR     # foo      \n # Pre comment \n 12 5.4\n # Post comment\n    \n#Another comment\n 13 8.93   # Trailing comment"), IF);
        assert_eq!(name, "extpar");
        assert_eq!(result.len(), 2);
        assert_eq!(result[&12], Payload { value: 5.4, comment: None });
        assert_eq!(result[&13], Payload { value: 8.93, comment: Some(&b" Trailing comment"[..]) });

        let (name, result) = unwrap_anyblock!(parse_anyblock(b"BLOCK MASS     # foo      \n # Pre comment \n 12 5.4\n # Post comment\n    \n#Another comment\n 13 8.93   # Trailing comment"), IF);
        assert_eq!(name, "mass");
        assert_eq!(result.len(), 2);
        assert_eq!(result[&12], Payload { value: 5.4, comment: None });
        assert_eq!(result[&13], Payload { value: 8.93, comment: Some(&b" Trailing comment"[..]) });

        let (name, result) = unwrap_anyblock!(parse_anyblock(b"BLOCK NMIX     # foo      \n # Pre comment \n 13 6   -78.32\n # Post comment\n    \n#Another comment\n 18 8   4.2e4  # Trailing comment"), IIF);
        assert_eq!(name, "nmix");
        assert_eq!(result.len(), 2);
        assert_eq!(result[&(13,6)], Payload { value: -78.32, comment: None });
        assert_eq!(result[&(18,8)], Payload { value: 4.2e4, comment: Some(&b" Trailing comment"[..]) });

        let (name, result) = unwrap_anyblock!(parse_anyblock(b"BLOCK uMIX     # foo      \n # Pre comment \n 13 6   -78.32\n # Post comment\n    \n#Another comment\n 18 8   4.2e4  # Trailing comment"), IIF);
        assert_eq!(name, "umix");
        assert_eq!(result.len(), 2);
        assert_eq!(result[&(13,6)], Payload { value: -78.32, comment: None });
        assert_eq!(result[&(18,8)], Payload { value: 4.2e4, comment: Some(&b" Trailing comment"[..]) });

        let (name, result) = unwrap_anyblock!(parse_anyblock(b"BLOCK vmix     # foo      \n # Pre comment \n 13 6   -78.32\n # Post comment\n    \n#Another comment\n 18 8   4.2e4  # Trailing comment"), IIF);
        assert_eq!(name, "vmix");
        assert_eq!(result.len(), 2);
        assert_eq!(result[&(13,6)], Payload { value: -78.32, comment: None });
        assert_eq!(result[&(18,8)], Payload { value: 4.2e4, comment: Some(&b" Trailing comment"[..]) });

        let (name, result) = unwrap_anyblock!(parse_anyblock(b"BLOCK stopmix     # foo      \n # Pre comment \n 13 6   -78.32\n # Post comment\n    \n#Another comment\n 18 8   4.2e4  # Trailing comment"), IIF);
        assert_eq!(name, "stopmix");
        assert_eq!(result.len(), 2);
        assert_eq!(result[&(13,6)], Payload { value: -78.32, comment: None });
        assert_eq!(result[&(18,8)], Payload { value: 4.2e4, comment: Some(&b" Trailing comment"[..]) });

        let (name, result) = unwrap_anyblock!(parse_anyblock(b"BLOCK sbotmix     # foo      \n # Pre comment \n 13 6   -78.32\n # Post comment\n    \n#Another comment\n 18 8   4.2e4  # Trailing comment"), IIF);
        assert_eq!(name, "sbotmix");
        assert_eq!(result.len(), 2);
        assert_eq!(result[&(13,6)], Payload { value: -78.32, comment: None });
        assert_eq!(result[&(18,8)], Payload { value: 4.2e4, comment: Some(&b" Trailing comment"[..]) });

        let (name, result) = unwrap_anyblock!(parse_anyblock(b"BLOCK staumix     # foo      \n # Pre comment \n 13 6   -78.32\n # Post comment\n    \n#Another comment\n 18 8   4.2e4  # Trailing comment"), IIF);
        assert_eq!(name, "staumix");
        assert_eq!(result.len(), 2);
        assert_eq!(result[&(13,6)], Payload { value: -78.32, comment: None });
        assert_eq!(result[&(18,8)], Payload { value: 4.2e4, comment: Some(&b" Trailing comment"[..]) });

        let (name, q, result) = unwrap_anyblock_q!(parse_anyblock(b"BLOCK HMIX    Q= -321.913   # foo      \n # Pre comment \n 12 5.4\n # Post comment\n    \n#Another comment\n 13 8.93   # Trailing comment"), IFQ);
        assert_eq!(name, "hmix");
        assert_eq!(q, -321.913);
        assert_eq!(result.len(), 2);
        assert_eq!(result[&12], Payload { value: 5.4, comment: None });
        assert_eq!(result[&13], Payload { value: 8.93, comment: Some(&b" Trailing comment"[..]) });

        let (name, q, result) = unwrap_anyblock_q!(parse_anyblock(b"BLOCK GAuge    Q= -321.913   # foo      \n # Pre comment \n 12 5.4\n # Post comment\n    \n#Another comment\n 13 8.93   # Trailing comment"), IFQ);
        assert_eq!(name, "gauge");
        assert_eq!(q, -321.913);
        assert_eq!(result.len(), 2);
        assert_eq!(result[&12], Payload { value: 5.4, comment: None });
        assert_eq!(result[&13], Payload { value: 8.93, comment: Some(&b" Trailing comment"[..]) });

        let (name, q, result) = unwrap_anyblock_q!(parse_anyblock(b"BLOCK msoft    Q= -321.913   # foo      \n # Pre comment \n 12 5.4\n # Post comment\n    \n#Another comment\n 13 8.93   # Trailing comment"), IFQ);
        assert_eq!(name, "msoft");
        assert_eq!(q, -321.913);
        assert_eq!(result.len(), 2);
        assert_eq!(result[&12], Payload { value: 5.4, comment: None });
        assert_eq!(result[&13], Payload { value: 8.93, comment: Some(&b" Trailing comment"[..]) });

        let (name, q, result) = unwrap_anyblock_q!(parse_anyblock(b"BLOCK Au   Q= 723.42     # foo      \n # Pre comment \n 13 6   -78.32\n # Post comment\n    \n#Another comment\n 18 8   4.2e4  # Trailing comment"), IIFQ);
        assert_eq!(name, "au");
        assert_eq!(q, 723.42);
        assert_eq!(result.len(), 2);
        assert_eq!(result[&(13,6)], Payload { value: -78.32, comment: None });
        assert_eq!(result[&(18,8)], Payload { value: 4.2e4, comment: Some(&b" Trailing comment"[..]) });

        let (name, q, result) = unwrap_anyblock_q!(parse_anyblock(b"BLOCK AD   Q= 723.42     # foo      \n # Pre comment \n 13 6   -78.32\n # Post comment\n    \n#Another comment\n 18 8   4.2e4  # Trailing comment"), IIFQ);
        assert_eq!(name, "ad");
        assert_eq!(q, 723.42);
        assert_eq!(result.len(), 2);
        assert_eq!(result[&(13,6)], Payload { value: -78.32, comment: None });
        assert_eq!(result[&(18,8)], Payload { value: 4.2e4, comment: Some(&b" Trailing comment"[..]) });

        let (name, q, result) = unwrap_anyblock_q!(parse_anyblock(b"BLOCK aE   Q= 724.42     # foo      \n # Pre comment \n 12 6   -78.32\n # Post comment\n    \n#Another comment\n 18 9   4.3e4  # Trailing comment"), IIFQ);
        assert_eq!(name, "ae");
        assert_eq!(q, 724.42);
        assert_eq!(result.len(), 2);
        assert_eq!(result[&(12,6)], Payload { value: -78.32, comment: None });
        assert_eq!(result[&(18,9)], Payload { value: 4.3e4, comment: Some(&b" Trailing comment"[..]) });

        let (name, q, result) = unwrap_anyblock_q!(parse_anyblock(b"BLOCK yu   Q= 723.42     # foo      \n # Pre comment \n 13 6   -78.32\n # Post comment\n    \n#Another comment\n 18 8   4.2e4  # Trailing comment"), IIFQ);
        assert_eq!(name, "yu");
        assert_eq!(q, 723.42);
        assert_eq!(result.len(), 2);
        assert_eq!(result[&(13,6)], Payload { value: -78.32, comment: None });
        assert_eq!(result[&(18,8)], Payload { value: 4.2e4, comment: Some(&b" Trailing comment"[..]) });

        let (name, q, result) = unwrap_anyblock_q!(parse_anyblock(b"BLOCK yD   Q= 723.42     # foo      \n # Pre comment \n 13 6   -78.32\n # Post comment\n    \n#Another comment\n 18 8   4.2e4  # Trailing comment"), IIFQ);
        assert_eq!(name, "yd");
        assert_eq!(q, 723.42);
        assert_eq!(result.len(), 2);
        assert_eq!(result[&(13,6)], Payload { value: -78.32, comment: None });
        assert_eq!(result[&(18,8)], Payload { value: 4.2e4, comment: Some(&b" Trailing comment"[..]) });

        let (name, q, result) = unwrap_anyblock_q!(parse_anyblock(b"BLOCK YE   Q= 723.42     # foo      \n # Pre comment \n 13 6   -78.32\n # Post comment\n    \n#Another comment\n 18 8   4.2e4  # Trailing comment"), IIFQ);
        assert_eq!(name, "ye");
        assert_eq!(q, 723.42);
        assert_eq!(result.len(), 2);
        assert_eq!(result[&(13,6)], Payload { value: -78.32, comment: None });
        assert_eq!(result[&(18,8)], Payload { value: 4.2e4, comment: Some(&b" Trailing comment"[..]) });

        let (name, result) = unwrap_anyblock!(parse_anyblock(b"BLOCK spinfo     # foo      \n # Pre comment \n 12 version\n # Post comment\n    \n#Another comment\n 18 here stands an error code   # Trailing comment"), IS);
        assert_eq!(name, "spinfo");
        assert_eq!(result.len(), 2);
        assert_eq!(result[&12], Payload { value: &b"version"[..], comment: None });
        assert_eq!(result[&18], Payload { value: &b"here stands an error code   "[..], comment: Some(&b" Trailing comment"[..]) });

        let (name, result) = unwrap_anyblock!(parse_anyblock(b"BLOCK UNKNOWN     # foo      \n # Pre comment \n 13 6   -78.32\n # Post comment\n    \n#Another comment\n 18 8   4.2e4  # Trailing comment"), Unknown);
        assert_eq!(name, "unknown");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Payload { value: b" 13 6   -78.32".as_ref(), comment: None });
        assert_eq!(result[1], Payload { value: b" 18 8   4.2e4  ".as_ref(), comment: Some(&b" Trailing comment"[..]) });
    }

    #[test]
    fn test_example1() {
        let input = b"\
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
     5   -100.0     # A0";
        parse_slha(input).unwrap();
    }
}
