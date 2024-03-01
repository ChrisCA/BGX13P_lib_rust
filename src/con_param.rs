use anyhow::{anyhow, Error, Result};
use log::debug;
use winnow::{
    ascii::{hex_digit1, multispace1},
    combinator::preceded,
    PResult, Parser,
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

        let mac = parse_con_param(&mut value.as_str()).map_err(|e| anyhow!(e))?;

        Ok(ConInfo(mac))
    }
}

/// takes the con_param answer (wo header) and parses the MAC from it
fn parse_con_param(input: &mut &str) -> PResult<Mac> {
    preceded(
        (
            "!",
            multispace1,
            "Param",
            multispace1,
            "Value",
            multispace1,
            "#",
            multispace1,
            "Addr",
            multispace1,
        ),
        hex_digit1,
    )
    .parse_to()
    .parse_next(input)
}

#[test]
fn test_parse_con_info() {
    let mut CON_INFO: &str = "!  Param Value\r\n
    #  Addr  D0CF5E828DF6\r\n
    #  Itvl  12\r\n
    #  Mtu   250\r\n
    #  Phy   1m\r\n
    #  Tout  400\r\n
    #  Err   0000\r\n";

    let ex_res = ConInfo("D0CF5E828DF6".parse().unwrap());

    let test_res = ConInfo(parse_con_param(&mut CON_INFO).unwrap());

    assert_eq!(ex_res, test_res)
}
