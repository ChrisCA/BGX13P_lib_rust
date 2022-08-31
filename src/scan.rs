use std::error::Error;

use log::debug;

use crate::{response::BgxResponse, scanned_device::ScannedDevice};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ScanResult(pub Vec<ScannedDevice>);

impl TryFrom<BgxResponse> for ScanResult {
    type Error = Box<dyn Error>;

    fn try_from(value: BgxResponse) -> Result<Self, Self::Error> {
        let value = match value {
            BgxResponse::DataWithHeader(_, (_, s, _)) => s,
            BgxResponse::DataWithoutHeader(d) => {
                return Err(format!("Data without header cannot be a scan result: {:?}", d).into())
            }
        };

        debug!("Scan results:\n{}", &value);

        let lines = value.lines().skip(1);

        Ok(ScanResult(
            lines.map(|f| f.parse().unwrap()).collect::<Vec<_>>(),
        ))
    }
}

#[test]
fn scan_result_1() {
    use crate::response::ResponseCodes;
    use crate::response_header::ResponseHeader;

    let resp: BgxResponse = BgxResponse::DataWithHeader(ResponseHeader{response_code:ResponseCodes::Success,length:123}, (Vec::new(),"!  # RSSI BD_ADDR           Device Name\r\n#  1  -47 d0:cf:5e:82:85:06 LOR-8090\r\n#  2  -52 00:0d:6f:a7:a1:54 LOR-8090\r\n".to_string(),Vec::new())) ;
    let res_test: ScanResult = resp.try_into().unwrap();

    let res_made = ScanResult(vec![
        ScannedDevice {
            mac: "d0cf5e828506".parse().unwrap(),
            friendly_name: "LOR-8090".to_string(),
            rssi: -47,
        },
        ScannedDevice {
            mac: "000d6fa7a154".parse().unwrap(),
            friendly_name: "LOR-8090".to_string(),
            rssi: -52,
        },
    ]);

    assert_eq!(res_test, res_made);
}

// #[test]
// fn scan_result_2() {
//     const input: &[u8] = b"R000269
//     !  # RSSI BD_ADDR           Device Name
//     #  1  -72 ec:1b:bd:1b:12:a1 LOR-1490
//     #  2  -84 60:a4:23:c5:91:b7 LOR-8090
//     #  3  -81 60:a4:23:c4:37:eb LOR-8090
//     #  4  -81 ec:1b:bd:1b:12:e0 LOR-1490
//     #  5  -84 84:71:27:9d:f8:f2 LOR-1490
//     #  6  -79 60:a4:23:c5:90:ab LOR-1450";

//     let lines = std::str::from_utf8(input).unwrap().lines().skip(2);

//     let res1 = lines
//         .map(|f| ScannedDevice::from_str(f).unwrap())
//         .collect::<Vec<_>>();
//     let res2 = vec![
//         ScannedDevice {
//             mac: "d0cf5e828506".to_string(),
//             friendly_name: "LOR-8090".to_string(),
//             rssi: -47,
//         },
//         ScannedDevice {
//             mac: "000d6fa7a154".to_string(),
//             friendly_name: "LOR-8090".to_string(),
//             rssi: -52,
//         },
//     ];

//     assert_eq!(res1, res2);
// }
