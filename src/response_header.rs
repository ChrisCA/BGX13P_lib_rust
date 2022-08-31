use std::error::Error;

use crate::response::ResponseCodes;

#[derive(Debug, PartialEq, Eq)]
pub struct ResponseHeader {
    pub response_code: ResponseCodes,
    pub length: u16,
}

impl TryFrom<&[u8]> for ResponseHeader {
    type Error = Box<dyn Error>;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        // typical header -> R000117\r\n but R and newline should already been removed by the parser

        if value.len() != 6 {
            return Err("Header code lenght is != 6".into());
        }

        let value = std::str::from_utf8(value)?;

        Ok(Self {
            response_code: value
                .get(..1)
                .ok_or("Couldn't get code number in header")?
                .parse::<u8>()
                .map_err(|e| format!("{e}\tCouldn't parse {:?} to response code", value.get(..1)))?
                .try_into()?,
            length: value
                .get(1..6)
                .ok_or("Couldn't get length numbers in header")?
                .parse::<u16>()?,
        })
    }
}

#[test]
fn test_response_header_1() {
    const HEADER: &[u8] = b"000009";

    let h = ResponseHeader::try_from(HEADER).unwrap();
    let h2 = ResponseHeader {
        response_code: ResponseCodes::Success,
        length: 9,
    };

    assert_eq!(h, h2);
}

#[test]
#[should_panic]
fn test_response_header_2() {
    const HEADER: &[u8] = b"000010";

    let h = ResponseHeader::try_from(HEADER).unwrap();
    let h2 = ResponseHeader {
        response_code: ResponseCodes::Success,
        length: 9,
    };

    assert_eq!(h, h2);
}

#[test]
#[should_panic]
fn test_response_header_3() {
    const HEADER: &[u8] = b"00009";

    let _ = ResponseHeader::try_from(HEADER).unwrap();
}

#[test]
#[should_panic]
fn test_response_header_4() {
    const HEADER: &[u8] = b"RR0009";

    let _ = ResponseHeader::try_from(HEADER).unwrap();
}

#[test]
#[should_panic]
fn test_response_header_5() {
    const HEADER: &[u8] = b"R10009";

    let _ = ResponseHeader::try_from(HEADER).unwrap();
}

#[test]
#[should_panic]
fn test_response_header_6() {
    const HEADER: &[u8] = b"2120009";

    let _ = ResponseHeader::try_from(HEADER).unwrap();
}
