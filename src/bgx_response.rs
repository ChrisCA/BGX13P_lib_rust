use strum::Display;
use thiserror::Error;

use crate::response_header::ResponseHeader;

#[derive(Debug, PartialEq, Error, Display)]
pub(crate) enum ResponseCodes {
    Success,
    CommandFailed,
    ParseError,
    UnknownCommand,
    TooFewArguments,
    TooManyArguments,
    UnknownVariableOrOption,
    InvalidArgument,
    Timeout,
    SecurityMismatch,
}

impl From<u8> for ResponseCodes {
    fn from(value: u8) -> Self {
        match value {
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
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum ModuleResponse {
    DataWithHeader(ResponseHeader, String),
    DataWithoutHeader(Vec<u8>),
}
