use std::str::FromStr;

use anyhow::{anyhow, Context, Error};

use crate::bgx_response::ResponseCodes;

#[derive(Debug, PartialEq)]
pub(crate) struct ResponseHeader {
    pub response_code: ResponseCodes,
    pub length: u16,
}

impl FromStr for ResponseHeader {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // typical header -> R000117\r\n but R and newline should already been removed by the parser

        if s.len() != 6 {
            return Err(anyhow!("Header code lenght is != 6"));
        }

        Ok(Self {
            response_code: s
                .get(..1)
                .context("Couldn't get code number in header")?
                .parse::<u8>()?
                .into(),
            length: s
                .get(1..6)
                .context("Couldn't get length numbers in header")?
                .parse::<u16>()?,
        })
    }
}

#[test]
fn test_response_header_1() {
    const HEADER: &str = "000009";

    let h = ResponseHeader::from_str(HEADER).unwrap();
    let h2 = ResponseHeader {
        response_code: ResponseCodes::Success,
        length: 9,
    };

    assert_eq!(h, h2);
}

#[test]
#[should_panic]
fn test_response_header_2() {
    const HEADER: &str = "000010";

    let h = ResponseHeader::from_str(HEADER).unwrap();
    let h2 = ResponseHeader {
        response_code: ResponseCodes::Success,
        length: 9,
    };

    assert_eq!(h, h2);
}

#[test]
#[should_panic]
fn test_response_header_3() {
    const HEADER: &str = "00009";

    let _ = ResponseHeader::from_str(HEADER).unwrap();
}

#[test]
#[should_panic]
fn test_response_header_4() {
    const HEADER: &str = "RR0009";

    let _ = ResponseHeader::from_str(HEADER).unwrap();
}

#[test]
#[should_panic]
fn test_response_header_5() {
    const HEADER: &str = "R10009";

    let _ = ResponseHeader::from_str(HEADER).unwrap();
}

#[test]
#[should_panic]
fn test_response_header_6() {
    const HEADER: &str = "2120009";

    let _ = ResponseHeader::from_str(HEADER).unwrap();
}
