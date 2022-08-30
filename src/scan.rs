use std::{error::Error, str::FromStr};

use crate::scanned_device::ScannedDevice;

#[derive(Debug, PartialEq, Eq, Clone)]
struct ScanResult(Vec<ScannedDevice>);

impl FromStr for ScanResult {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

#[test]
fn scan_result_1() {
    const SCAN_RESULT: &str = "R000117\r\n!  # RSSI BD_ADDR           Device Name\r\n#  1  -47 d0:cf:5e:82:85:06 LOR-8090\r\n#  2  -52 00:0d:6f:a7:a1:54 LOR-8090\r\n";
    let lines = SCAN_RESULT.lines().skip(2);

    let res1 = lines
        .map(|f| ScannedDevice::from_str(f).unwrap())
        .collect::<Vec<_>>();
    let res2 = vec![
        ScannedDevice {
            mac: "d0cf5e828506".to_string(),
            friendly_name: "LOR-8090".to_string(),
            rssi: -47,
        },
        ScannedDevice {
            mac: "000d6fa7a154".to_string(),
            friendly_name: "LOR-8090".to_string(),
            rssi: -52,
        },
    ];

    assert_eq!(res1, res2);
}
