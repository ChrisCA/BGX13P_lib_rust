use winnow::{combinator::preceded, token::take_until, PResult, Parser};

pub fn parse_fw_ver<'a>(input: &mut &'a str) -> PResult<&'a str> {
    // WARN: Do not match on BGX13P. instead of BGX13 here as this reported name is not consistent over older versions
    preceded(take_until(0.., "BGX13"), take_until(1.., "\r\n")).parse_next(input)
}

#[test]
fn parse_firmware_version_1() {
    let input1 = &mut "XXXXXXBGX13P.1.2.2738.2-1524-2738\r\n";

    const TARGET: &str = "BGX13P.1.2.2738.2-1524-2738";

    assert_eq!(parse_fw_ver(input1).unwrap(), TARGET);
}

#[test]
fn parse_firmware_version_2() {
    let mut input2 = "BGX13P.1.2.2738.2-1524-2738\r\n";

    const TARGET: &str = "BGX13P.1.2.2738.2-1524-2738";

    assert_eq!(parse_fw_ver(&mut input2).unwrap(), TARGET);
}

#[test]
fn parse_firmware_version_3() {
    let mut input3 = "XXXXXXBGX13P.1.2.2738.2-1524-2738\r\nXXXX";

    const TARGET: &str = "BGX13P.1.2.2738.2-1524-2738";

    assert_eq!(parse_fw_ver(&mut input3).unwrap(), TARGET);
}

#[test]
fn parse_firmware_version_4() {
    let mut input4 = "XXXX\r\nXXBGX13P.1.2.2738.2-1524-2738\r\nXXXX";

    const TARGET: &str = "BGX13P.1.2.2738.2-1524-2738";

    assert_eq!(parse_fw_ver(&mut input4).unwrap(), TARGET);
}

#[test]
#[should_panic]
fn parse_firmware_version_5() {
    let mut input1 = "XXXXXXBGX13P.1.2.2738.2-1524-2738";
    let mut input2 = "BX13P.1.2.2738.2-1524-2738\r\n";
    let mut input3 = "XXXXXXBGX13P.1.2.2738.2-1524-2738\rX\nXXXX";

    const TARGET: &str = "BGX13P.1.2.2738.2-1524-2738";

    assert_eq!(parse_fw_ver(&mut input1).unwrap(), TARGET);
    assert_eq!(parse_fw_ver(&mut input2).unwrap(), TARGET);
    assert_eq!(parse_fw_ver(&mut input3).unwrap(), TARGET);
}
