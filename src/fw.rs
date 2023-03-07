use winnow::{
    bytes::{take_until0, take_until1},
    sequence::preceded,
    IResult,
};

pub fn parse_fw_ver(s: &str) -> IResult<&str, &str> {
    // WARN: Do not match on BGX13P. instead of BGX13 here as this reported name is not consistent over older versions
    preceded(take_until0("BGX13"), take_until1("\r\n"))(s)
}

#[test]
fn parse_firmware_version_1() {
    let input1 = "XXXXXXBGX13P.1.2.2738.2-1524-2738\r\n";
    let input2 = "BGX13P.1.2.2738.2-1524-2738\r\n";
    let input3 = "XXXXXXBGX13P.1.2.2738.2-1524-2738\r\nXXXX";

    const TARGET: &str = "BGX13P.1.2.2738.2-1524-2738";

    assert_eq!(parse_fw_ver(input1).unwrap().1, TARGET);
    assert_eq!(parse_fw_ver(input2).unwrap().1, TARGET);
    assert_eq!(parse_fw_ver(input3).unwrap().1, TARGET);
}

#[test]
#[should_panic]
fn parse_firmware_version_2() {
    let input1 = "XXXXXXBGX13P.1.2.2738.2-1524-2738";
    let input2 = "BX13P.1.2.2738.2-1524-2738\r\n";
    let input3 = "XXXXXXBGX13P.1.2.2738.2-1524-2738\rX\nXXXX";

    const TARGET: &str = "BGX13P.1.2.2738.2-1524-2738";

    assert_eq!(parse_fw_ver(input1).unwrap().1, TARGET);
    assert_eq!(parse_fw_ver(input2).unwrap().1, TARGET);
    assert_eq!(parse_fw_ver(input3).unwrap().1, TARGET);
}
