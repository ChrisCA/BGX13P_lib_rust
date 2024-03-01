use log::{debug, warn};
use thiserror::Error;
use winnow::{
    combinator::{repeat_till, rest},
    token::{any, take},
    PResult, Parser,
};

use crate::response_header::{parse_header, ResponseHeader};

#[derive(Debug, PartialEq, Eq, Error, Clone, Copy)]
pub enum ResponseCodes {
    #[error("Success")]
    Success,
    #[error("CommandFailed")]
    CommandFailed,
    #[error("ParseError")]
    ParseError,
    #[error("UnknownCommand")]
    UnknownCommand,
    #[error("TooFewArguments")]
    TooFewArguments,
    #[error("TooManyArguments")]
    TooManyArguments,
    #[error("UnknownVariableOrOption")]
    UnknownVariableOrOption,
    #[error("InvalidArgument")]
    InvalidArgument,
    #[error("Timeout")]
    Timeout,
    #[error("SecurityMismatch")]
    SecurityMismatch,
}

#[derive(Debug, PartialEq, Eq, Error)]
pub enum Errors {
    #[error("Only response code from 0 to 9 are expected, got: {0}")]
    InvalidResponseCode(u8),
}

impl TryFrom<u8> for ResponseCodes {
    type Error = Errors;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => ResponseCodes::Success,
            1 => ResponseCodes::CommandFailed,
            2 => ResponseCodes::ParseError,
            3 => ResponseCodes::UnknownCommand,
            4 => ResponseCodes::TooFewArguments,
            5 => ResponseCodes::TooManyArguments,
            6 => ResponseCodes::UnknownVariableOrOption,
            7 => ResponseCodes::InvalidArgument,
            8 => ResponseCodes::Timeout,
            9 => ResponseCodes::SecurityMismatch,
            _ => return Err(Errors::InvalidResponseCode(value)),
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BgxResponse {
    DataWithHeader(ResponseHeader, String),
    DataWithoutHeader(Vec<u8>),
}

// parses any BGX response
pub fn parse_response(input: &mut &[u8]) -> PResult<BgxResponse> {
    /*
    SAMPLE:
    R000029\r\n
    BGX13P.1.2.2738.2-1524-2738\r\n
    */
    debug!("BGX answered: {:?}", input);

    // return response if no header has been found, otherwise show if there has been something before and get header
    let (input, header) = match repeat_till(0.., any, parse_header).parse_next(input) {
        Ok((before, header)) => {
            let before: Vec<u8> = before; // just for defining the type
            if !before.is_empty() {
                warn!("Data before header: {:?}", before);
            }
            (input, header)
        }
        Err(_) => {
            return Ok(BgxResponse::DataWithoutHeader(
                rest.map(Vec::from).parse_next(input)?,
            ))
        }
    };

    debug!("Header: {:?}", &header);
    let answer = take(header.data_length).parse_next(input)?;

    let answer = match String::from_utf8(answer.to_vec()) {
        Ok(answer) => answer,
        Err(_) => format!("{answer:?}"),
    };

    Ok(BgxResponse::DataWithHeader(header, answer))
}

#[test]
fn module_response_test_1() {
    let mut input1: &[u8] = b"R000029\r\nBGX13P.1.2.2738.2-1524-2738\r\n";

    assert_eq!(
        BgxResponse::DataWithHeader(
            ResponseHeader {
                response_code: ResponseCodes::Success,
                data_length: 29
            },
            "BGX13P.1.2.2738.2-1524-2738\r\n".to_string(),
        ),
        parse_response(&mut input1).unwrap()
    )
}

#[test]
fn module_response_test_2() {
    let mut input: &[u8] = &[
        82, 48, 48, 48, 50, 51, 49, 13, 10, 33, 32, 32, 35, 32, 82, 83, 83, 73, 32, 66, 68, 95, 65,
        68, 68, 82, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 68, 101, 118, 105, 99, 101, 32, 78,
        97, 109, 101, 13, 10, 35, 32, 32, 49, 32, 32, 45, 55, 49, 32, 101, 99, 58, 49, 98, 58, 98,
        100, 58, 49, 98, 58, 49, 50, 58, 97, 49, 32, 76, 79, 82, 45, 49, 52, 57, 48, 13, 10, 35,
        32, 32, 50, 32, 32, 45, 55, 54, 32, 56, 52, 58, 55, 49, 58, 50, 55, 58, 57, 100, 58, 102,
        56, 58, 102, 50, 32, 76, 79, 82, 45, 49, 52, 57, 48, 13, 10, 35, 32, 32, 51, 32, 32, 45,
        55, 52, 32, 54, 48, 58, 97, 52, 58, 50, 51, 58, 99, 53, 58, 57, 48, 58, 97, 98, 32, 76, 79,
        82, 45, 49, 52, 53, 48, 13, 10, 35, 32, 32, 52, 32, 32, 45, 56, 48, 32, 101, 99, 58, 49,
        98, 58, 98, 100, 58, 49, 98, 58, 49, 50, 58, 101, 48, 32, 76, 79, 82, 45, 49, 52, 57, 48,
        13, 10, 35, 32, 32, 53, 32, 32, 45, 56, 53, 32, 54, 48, 58, 97, 52, 58, 50, 51, 58, 99, 53,
        58, 57, 49, 58, 98, 55, 32, 76, 79, 82, 45, 56, 48, 57, 48, 13, 10,
    ];

    let input_wo_header: &[u8] = &[
        33, 32, 32, 35, 32, 82, 83, 83, 73, 32, 66, 68, 95, 65, 68, 68, 82, 32, 32, 32, 32, 32, 32,
        32, 32, 32, 32, 32, 68, 101, 118, 105, 99, 101, 32, 78, 97, 109, 101, 13, 10, 35, 32, 32,
        49, 32, 32, 45, 55, 49, 32, 101, 99, 58, 49, 98, 58, 98, 100, 58, 49, 98, 58, 49, 50, 58,
        97, 49, 32, 76, 79, 82, 45, 49, 52, 57, 48, 13, 10, 35, 32, 32, 50, 32, 32, 45, 55, 54, 32,
        56, 52, 58, 55, 49, 58, 50, 55, 58, 57, 100, 58, 102, 56, 58, 102, 50, 32, 76, 79, 82, 45,
        49, 52, 57, 48, 13, 10, 35, 32, 32, 51, 32, 32, 45, 55, 52, 32, 54, 48, 58, 97, 52, 58, 50,
        51, 58, 99, 53, 58, 57, 48, 58, 97, 98, 32, 76, 79, 82, 45, 49, 52, 53, 48, 13, 10, 35, 32,
        32, 52, 32, 32, 45, 56, 48, 32, 101, 99, 58, 49, 98, 58, 98, 100, 58, 49, 98, 58, 49, 50,
        58, 101, 48, 32, 76, 79, 82, 45, 49, 52, 57, 48, 13, 10, 35, 32, 32, 53, 32, 32, 45, 56,
        53, 32, 54, 48, 58, 97, 52, 58, 50, 51, 58, 99, 53, 58, 57, 49, 58, 98, 55, 32, 76, 79, 82,
        45, 56, 48, 57, 48, 13, 10,
    ];

    assert_eq!(
        BgxResponse::DataWithHeader(
            ResponseHeader {
                response_code: ResponseCodes::Success,
                data_length: 231
            },
            String::from_utf8(input_wo_header.to_vec()).unwrap(),
        ),
        parse_response(&mut input).unwrap()
    )
}
