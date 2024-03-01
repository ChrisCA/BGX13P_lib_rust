use std::str::{self, FromStr};

use winnow::{
    ascii::{crlf, digit1},
    combinator::delimited,
    token::take,
    PResult, Parser,
};

use crate::response::ResponseCodes;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ResponseHeader {
    pub response_code: ResponseCodes,
    pub data_length: u32,
}

// typical header -> R000117\r\n but R and newline should already been removed by the parser
pub fn parse_header(input: &mut &[u8]) -> PResult<ResponseHeader> {
    delimited(
        "R",
        (
            take(1u8)
                .and_then(digit1)
                .try_map(str::from_utf8)
                .try_map(u8::from_str)
                .try_map(ResponseCodes::try_from),
            take(5u8)
                .and_then(digit1)
                .try_map(str::from_utf8)
                .try_map(u32::from_str),
        ),
        crlf,
    )
    .map(|p| ResponseHeader {
        response_code: p.0,
        data_length: p.1,
    })
    .parse_next(input)
}

#[test]
fn test_response_header_1() {
    let mut HEADER: &[u8] = b"R000009\r\n";

    let h = parse_header(&mut HEADER).unwrap();
    let h2 = ResponseHeader {
        response_code: ResponseCodes::Success,
        data_length: 9,
    };

    assert_eq!(h, h2);
}

#[test]
#[should_panic]
fn test_response_header_2() {
    let mut HEADER: &[u8] = b"R000010\r\n";

    let h = parse_header(&mut HEADER).unwrap();
    let h2 = ResponseHeader {
        response_code: ResponseCodes::Success,
        data_length: 9,
    };

    assert_eq!(h, h2);
}

#[test]
#[should_panic]
fn test_response_header_3() {
    let mut HEADER: &[u8] = b"00009\r\n";

    let _ = parse_header(&mut HEADER).unwrap();
}

#[test]
#[should_panic]
fn test_response_header_4() {
    let mut HEADER: &[u8] = b"RR0009\r\n";

    let _ = parse_header(&mut HEADER).unwrap();
}

#[test]
#[should_panic]
fn test_response_header_5() {
    let mut HEADER: &[u8] = b"R10009\r\n";

    let _ = parse_header(&mut HEADER).unwrap();
}

#[test]
#[should_panic]
fn test_response_header_6() {
    let mut HEADER: &[u8] = b"2120009\r\n";

    let _ = parse_header(&mut HEADER).unwrap();
}

#[test]
#[should_panic]
fn test_response_header_7() {
    let mut HEADER: &[u8] = b"R000009";

    let _ = parse_header(&mut HEADER).unwrap();
}
