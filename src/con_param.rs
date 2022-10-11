use std::error::Error;

use anyhow::Result;
use log::debug;
use nom::{
    bytes::complete::tag,
    character::complete::{hex_digit1, multispace1},
    error::VerboseError,
    sequence::{preceded, tuple},
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
    type Error = Box<dyn Error>;

    fn try_from(value: BgxResponse) -> Result<Self, Self::Error> {
        let value = match value {
            BgxResponse::DataWithHeader(_, (_, s, _)) => s,
            BgxResponse::DataWithoutHeader(d) => {
                return Err(
                    format!("Data without header cannot be a con param result: {:?}", d).into(),
                )
            }
        };

        debug!("con param results:\n{}", &value);

        let mac = parse_con_param(&value)?;

        Ok(ConInfo(mac))
    }
}

/// takes the con_param answer (wo header) and parses the MAC from it
fn parse_con_param(s: &str) -> Result<Mac> {
    preceded(
        tuple((
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
        )),
        hex_digit1,
    )(s)
    .map_err(|e: nom::Err<VerboseError<_>>| anyhow::anyhow!(e.to_string()))?
    .1
    .parse()
    .map_err(|e: Box<dyn Error>| anyhow::anyhow!(e.to_string()))
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

    let test_res = ConInfo(parse_con_param(CON_INFO).unwrap());

    assert_eq!(ex_res, test_res)
}
