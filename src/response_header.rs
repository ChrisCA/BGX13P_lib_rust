use winnow::{
    bytes::{tag, take},
    character::{crlf, digit1},
    sequence::delimited,
    IResult, Parser,
};

use std::str::{self, FromStr};

use crate::response::ResponseCodes;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ResponseHeader {
    pub response_code: ResponseCodes,
    pub data_length: u32,
}

// typical header -> R000117\r\n but R and newline should already been removed by the parser
pub fn parse_header(input: &[u8]) -> IResult<&[u8], ResponseHeader> {
    let (input, (response_code, data_length)) = delimited(
        tag("R"),
        (
            take(1u8)
                .and_then(digit1)
                .map_res(str::from_utf8)
                .map_res(u8::from_str)
                .map_res(ResponseCodes::try_from),
            take(5u8)
                .and_then(digit1)
                .map_res(str::from_utf8)
                .map_res(u32::from_str),
        ),
        crlf,
    )(input)?;

    Ok((
        input,
        ResponseHeader {
            response_code,
            data_length,
        },
    ))
}

#[test]
fn test_response_header_1() {
    const HEADER: &[u8] = b"R000009\r\n";

    let (_, h) = parse_header(HEADER).unwrap();
    let h2 = ResponseHeader {
        response_code: ResponseCodes::Success,
        data_length: 9,
    };

    assert_eq!(h, h2);
}

#[test]
#[should_panic]
fn test_response_header_2() {
    const HEADER: &[u8] = b"R000010\r\n";

    let (_, h) = parse_header(HEADER).unwrap();
    let h2 = ResponseHeader {
        response_code: ResponseCodes::Success,
        data_length: 9,
    };

    assert_eq!(h, h2);
}

#[test]
#[should_panic]
fn test_response_header_3() {
    const HEADER: &[u8] = b"00009\r\n";

    let _ = parse_header(HEADER).unwrap();
}

#[test]
#[should_panic]
fn test_response_header_4() {
    const HEADER: &[u8] = b"RR0009\r\n";

    let _ = parse_header(HEADER).unwrap();
}

#[test]
#[should_panic]
fn test_response_header_5() {
    const HEADER: &[u8] = b"R10009\r\n";

    let _ = parse_header(HEADER).unwrap();
}

#[test]
#[should_panic]
fn test_response_header_6() {
    const HEADER: &[u8] = b"2120009\r\n";

    let _ = parse_header(HEADER).unwrap();
}

#[test]
#[should_panic]
fn test_response_header_7() {
    const HEADER: &[u8] = b"R000009";

    let _ = parse_header(HEADER).unwrap();
}
