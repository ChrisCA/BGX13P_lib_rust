use std::str::FromStr;

use anyhow::{anyhow, Error, Result};
use log::debug;
use winnow::{
    bytes::tag,
    character::{hex_digit1, multispace1},
    sequence::preceded,
    FinishIResult, IResult, Parser,
};

use crate::{mac::Mac, response::BgxResponse};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ConInfo(Mac);
// only mac is used as other information is not relevant
/*
!  Param Value\r\n
#  Addr  D0CF5E828DF6\r\n
#  Itvl  12\r\n
#  Mtu   250\r\n
#  Phy   1m\r\n
#  Tout  400\r\n
#  Err   0000\r\n
*/

impl TryFrom<BgxResponse> for ConInfo {
    type Error = Error;

    fn try_from(value: BgxResponse) -> Result<Self, Self::Error> {
        let value = match value {
            BgxResponse::DataWithHeader(_, s) => s,
            BgxResponse::DataWithoutHeader(d) => {
                return Err(anyhow!(
                    "Data without header cannot be a con param result: {:?}",
                    d
                ))
            }
        };

        debug!("con param results:\n{}", &value);

        let (_, mac) = parse_con_param(&value)
            .finish_err()
            .map_err(|e| e.into_owned())?;

        Ok(ConInfo(mac))
    }
}

/// takes the con_param answer (wo header) and parses the MAC from it
fn parse_con_param(s: &str) -> IResult<&str, Mac> {
    preceded(
        (
            tag("!"),
            multispace1,
            tag("Param"),
            multispace1,
            tag("Value"),
            multispace1,
            tag("#"),
            multispace1,
            tag("Addr"),
            multispace1,
        ),
        hex_digit1,
    )
    .map_res(Mac::from_str)
    .parse_next(s)
}

#[test]
fn test_parse_con_info() {
    const CON_INFO: &str = "!  Param Value\r\n
    #  Addr  D0CF5E828DF6\r\n
    #  Itvl  12\r\n
    #  Mtu   250\r\n
    #  Phy   1m\r\n
    #  Tout  400\r\n
    #  Err   0000\r\n";

    let ex_res = ConInfo("D0CF5E828DF6".parse().unwrap());

    let test_res = ConInfo(parse_con_param(CON_INFO).unwrap().1);

    assert_eq!(ex_res, test_res)
}
