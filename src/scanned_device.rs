use std::{error::Error, str::FromStr};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ScannedDevice {
    pub mac: String,
    pub friendly_name: String,
    pub rssi: i8,
}

impl FromStr for ScannedDevice {
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
fn scanned_device_1() {
    const SCAN_RESULT: &str = "#  1  -47 d0:cf:5e:82:85:06 LOR-8090";

    let res1 = ScannedDevice::from_str(SCAN_RESULT).unwrap();
    let res2 = ScannedDevice {
        mac: "d0cf5e828506".to_string(),
        friendly_name: "LOR-8090".to_string(),
        rssi: -47,
    };

    assert_eq!(res1, res2);
}