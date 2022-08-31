#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![forbid(clippy::indexing_slicing)]

use log::{debug, trace, warn};
use nom::{
    bytes::complete::{take_until, take_until1},
    error::VerboseError,
};

use serialport::{SerialPortInfo, SerialPortType};
use std::error::Error;

pub mod bgx;
mod command;
mod response;
mod response_header;
mod scan;
mod scanned_device;

fn parse_fw_ver(s: &str) -> Result<(&str, &str, &str), Box<dyn Error>> {
    // WARN: Do not match on BGX13P. instead of BGX13 here as this reported name is not consistent over older versions

    let first = take_until("BGX13")(s).map_err(|e: nom::Err<VerboseError<_>>| format!("{}", e))?;
    let second =
        take_until1("\r\n")(first.0).map_err(|e: nom::Err<VerboseError<_>>| format!("{}", e))?;

    Ok((first.1, second.1, second.0))
}

#[test]
fn parse_firmware_version_1() {
    let input1 = "XXXXXXBGX13P.1.2.2738.2-1524-2738\r\n";
    let input2 = "BGX13P.1.2.2738.2-1524-2738\r\n";
    let input3 = "XXXXXXBGX13P.1.2.2738.2-1524-2738\r\nXXXX";

    assert_eq!(
        parse_fw_ver(input1).unwrap(),
        ("XXXXXX", "BGX13P.1.2.2738.2-1524-2738", "\r\n")
    );
    assert_eq!(
        parse_fw_ver(input2).unwrap(),
        ("", "BGX13P.1.2.2738.2-1524-2738", "\r\n")
    );
    assert_eq!(
        parse_fw_ver(input3).unwrap(),
        ("XXXXXX", "BGX13P.1.2.2738.2-1524-2738", "\r\nXXXX")
    );
}

#[test]
#[should_panic]
fn parse_firmware_version_2() {
    let input1 = "XXXXXXBGX13P.1.2.2738.2-1524-2738";
    let input2 = "BX13P.1.2.2738.2-1524-2738\r\n";
    let input3 = "XXXXXXBGX13P.1.2.2738.2-1524-2738\rX\nXXXX";

    assert_eq!(
        parse_fw_ver(input1).unwrap(),
        ("XXXXXX", "BGX13P.1.2.2738.2-1524-2738", "\r\n")
    );
    assert_eq!(
        parse_fw_ver(input2).unwrap(),
        ("", "BGX13P.1.2.2738.2-1524-2738", "\r\n")
    );
    assert_eq!(
        parse_fw_ver(input3).unwrap(),
        ("XXXXXX", "BGX13P.1.2.2738.2-1524-2738", "\r\nXXXX")
    );
}

/// searches and returns serial port devices connected via USB
fn find_module() -> Result<Vec<SerialPortInfo>, Box<dyn Error>> {
    let ports = serialport::available_ports()?;
    trace!("Detected the following ports: {:?}", &ports);

    let ports = ports
        .into_iter()
        .filter(|p| {
            if let SerialPortType::UsbPort(n) = &p.port_type {
                debug!("Found USB port: {:?}", &n);

                if let Some(m) = &n.manufacturer {
                    if m.contains("Silicon Labs") || m.contains("Cygnal") || m.contains("CP21") {
                        return true;
                    } else {
                        warn!(
                            "Found UsbPort but manufacturer string {} didn't match for BGX",
                            m
                        );
                    }
                }
            }

            false
        })
        .collect::<Vec<_>>();

    Ok(ports)
}
