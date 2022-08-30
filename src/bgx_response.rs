use log::{debug, trace};
use nom::{
    bytes::complete::{take, take_till},
    character::complete::{char, crlf, digit1},
    error::VerboseError,
    sequence::delimited,
};
use thiserror::Error;

use crate::response_header::ResponseHeader;

#[derive(Debug, PartialEq, Eq, Error)]
pub(crate) enum ResponseCodes {
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

#[derive(Debug, PartialEq)]
pub(crate) enum BgxResponse {
    DataWithHeader(ResponseHeader, (Vec<u8>, String, Vec<u8>)),
    DataWithoutHeader(Vec<u8>),
}

impl TryFrom<&[u8]> for BgxResponse {
    type Error = Box<dyn std::error::Error>;

    /// takes input, returns optional content before, the actual content and the optional content after
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        /*
        SAMPLE:
        R000029\r\n
        BGX13P.1.2.2738.2-1524-2738\r\n
        */
        debug!("BGX answered: {:?}", value);

        // split everything off before the 'R'
        let (after_header, before_header) =
            take_till(|c| c == b'R')(value).map_err(|e: nom::Err<VerboseError<_>>| {
                format!("Didn't get any data when reading from BGX due to: {}", e)
            })?;

        // early return if no 'R' is found
        if after_header.is_empty() {
            return Ok(BgxResponse::DataWithoutHeader(before_header.to_vec()));
        }

        // get out the relevant numbers from the header
        let (module_message, header) = delimited(char('R'), digit1, crlf)(after_header)
            .map_err(|e: nom::Err<VerboseError<_>>| format!("{}", e))?;

        // parse header
        let header = ResponseHeader::try_from(header)?;
        trace!("Parsed header: {:?}", header);

        // split of the part of the module answer which has been communicated via the header
        let (after_message, module_message) = take(header.length)(module_message)
            .map_err(|e: nom::Err<VerboseError<_>>| format!("{}", e))?;

        let module_message = std::str::from_utf8(module_message)?;

        Ok(BgxResponse::DataWithHeader(
            header,
            (
                before_header.to_vec(),
                module_message.to_string(),
                after_message.to_vec(),
            ),
        ))
    }
}

#[test]
fn module_response_test_1() {
    const input1: &[u8] = b"R000029\r\nBGX13P.1.2.2738.2-1524-2738\r\n";

    assert_eq!(
        BgxResponse::DataWithHeader(
            ResponseHeader {
                response_code: ResponseCodes::Success,
                length: 29
            },
            (
                Vec::new(),
                "BGX13P.1.2.2738.2-1524-2738\r\n".to_string(),
                Vec::new()
            )
        ),
        BgxResponse::try_from(input1).unwrap()
    )
}
