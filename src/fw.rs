use anyhow::Result;
use nom::{
    bytes::complete::{take_until, take_until1},
    error::VerboseError,
    sequence::preceded,
};

pub fn parse_fw_ver(s: &str) -> Result<&str> {
    // WARN: Do not match on BGX13P. instead of BGX13 here as this reported name is not consistent over older versions

    Ok(preceded(take_until("BGX13"), take_until1("\r\n"))(s)
        .map_err(|e: nom::Err<VerboseError<_>>| anyhow::anyhow!(e.to_string()))?
        .1)
}

#[test]
fn parse_firmware_version_1() {
    let input1 = "XXXXXXBGX13P.1.2.2738.2-1524-2738\r\n";
    let input2 = "BGX13P.1.2.2738.2-1524-2738\r\n";
    let input3 = "XXXXXXBGX13P.1.2.2738.2-1524-2738\r\nXXXX";

    assert_eq!(parse_fw_ver(input1).unwrap(), "BGX13P.1.2.2738.2-1524-2738");
    assert_eq!(parse_fw_ver(input2).unwrap(), "BGX13P.1.2.2738.2-1524-2738");
    assert_eq!(parse_fw_ver(input3).unwrap(), "BGX13P.1.2.2738.2-1524-2738");
}

#[test]
#[should_panic]
fn parse_firmware_version_2() {
    let input1 = "XXXXXXBGX13P.1.2.2738.2-1524-2738";
    let input2 = "BX13P.1.2.2738.2-1524-2738\r\n";
    let input3 = "XXXXXXBGX13P.1.2.2738.2-1524-2738\rX\nXXXX";

    assert_eq!(parse_fw_ver(input1).unwrap(), "BGX13P.1.2.2738.2-1524-2738");
    assert_eq!(parse_fw_ver(input2).unwrap(), "BGX13P.1.2.2738.2-1524-2738");
    assert_eq!(parse_fw_ver(input3).unwrap(), "BGX13P.1.2.2738.2-1524-2738");
}
