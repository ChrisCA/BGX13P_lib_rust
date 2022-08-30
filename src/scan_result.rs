use std::{error::Error, str::FromStr};

#[derive(Debug, PartialEq, Eq)]
pub struct ScanResult {
    pub mac: String,
    pub friendly_name: String,
    pub rssi: i8,
}

impl FromStr for ScanResult {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut spl = s.split_whitespace();

        let rssi: i8 = spl.nth(2).ok_or("Couldn't get rssi on 2")?.parse()?;
        let mac = spl.next().ok_or("Couldn't get mac on 3")?.replace(':', "");
        let friendly_name = spl
            .next()
            .ok_or("Couldn't get friendly_name on 4")?
            .to_string();

        Ok(Self {
            mac,
            friendly_name,
            rssi,
        })
    }
}

#[test]
fn test_scan_result_1() {
    const SCAN_RESULT: &str = "R000117\r\n!  # RSSI BD_ADDR           Device Name\r\n#  1  -47 d0:cf:5e:82:85:06 LOR-8090\r\n#  2  -52 00:0d:6f:a7:a1:54 LOR-8090\r\n";
    let lines = SCAN_RESULT.lines().skip(2);

    let res1 = lines
        .map(|f| ScanResult::from_str(f).unwrap())
        .collect::<Vec<_>>();
    let res2 = vec![
        ScanResult {
            mac: "d0cf5e828506".to_string(),
            friendly_name: "LOR-8090".to_string(),
            rssi: -47,
        },
        ScanResult {
            mac: "000d6fa7a154".to_string(),
            friendly_name: "LOR-8090".to_string(),
            rssi: -52,
        },
    ];

    assert_eq!(res1, res2);
}

#[test]
fn test_scan_result_2() {
    const SCAN_RESULT: &str = "#  1  -47 d0:cf:5e:82:85:06 LOR-8090";

    let res1 = ScanResult::from_str(SCAN_RESULT).unwrap();
    let res2 = ScanResult {
        mac: "d0cf5e828506".to_string(),
        friendly_name: "LOR-8090".to_string(),
        rssi: -47,
    };

    assert_eq!(res1, res2);
}
