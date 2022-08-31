#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![forbid(clippy::indexing_slicing)]

use log::{debug, trace, warn};

use serialport::{SerialPortInfo, SerialPortType};
use std::error::Error;

pub mod bgx;
mod command;
mod fw;
mod response;
mod response_header;
mod scan;
mod scanned_device;

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
