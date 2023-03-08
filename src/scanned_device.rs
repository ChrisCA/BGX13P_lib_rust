use std::str::FromStr;

use anyhow::{Context, Error};
use log::trace;
use tap::Tap;

use crate::mac::Mac;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ScannedDevice {
    pub mac: Mac,
    pub friendly_name: String,
    pub rssi: i8,
}

impl FromStr for ScannedDevice {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        trace!("Parse ScannedDevice from: {}", s);

        let mut spl = s
            .split_whitespace()
            .tap(|spl| trace!("Split on whitespace: {:?}", spl));

        let rssi = spl
            .nth(2)
            .context("Couldn't get rssi on 2")?
            .tap(|r| trace!("Value to be parsed to i8 RSSI: {}", r))
            .parse()?;
        let mac = spl
            .next()
            .context("Couldn't get mac on 3")?
            .tap(|r| trace!("Mac to parse: {}", r))
            .parse()?;
        let friendly_name = spl
            .next()
            .context("Couldn't get friendly_name on 4")?
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
        mac: "d0cf5e828506".parse().unwrap(),
        friendly_name: "LOR-8090".to_string(),
        rssi: -47,
    };

    assert_eq!(res1, res2);
}
