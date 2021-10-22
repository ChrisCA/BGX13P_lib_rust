#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![forbid(clippy::indexing_slicing)]

use anyhow::{anyhow, Context, Result};
use combine::{
    any, between,
    parser::{
        char::string,
        range::{take, take_until_range},
        repeat::repeat_skip_until,
        Parser,
    },
    token,
};
use command::Command;
use log::{debug, info, trace, warn};
use scan_result::ScanResult;
use serialport::{
    ClearBuffer::All, DataBits, FlowControl, Parity, SerialPort, SerialPortInfo, SerialPortType,
    StopBits,
};
use std::{
    io::{Read, Write},
    str::FromStr,
    thread::sleep,
    time::Duration,
};

use crate::{
    bgx_response::{ModuleResponse, ResponseCodes},
    response_header::ResponseHeader,
};

mod bgx_response;
mod command;
mod response_header;
mod scan_result;
mod state_machine;

pub struct Bgx13p {
    port: Box<dyn SerialPort>,
    default_settings_applied: bool,
}

impl Bgx13p {
    pub fn new() -> Result<Self> {
        let p = find_module()?;

        for e in &p {
            info!("Found port: {}", e.port_name);
        }

        let chosen_port = p.first().context("Couldn't get any first port");
        let port_name = match chosen_port {
            Ok(p) => &p.port_name,
            Err(e) => {
                warn!(
                    "Couldn't determine USB port due to: {} => try using default /dev/ttyUSB0",
                    e
                );
                "/dev/ttyUSB0"
            }
        };

        let op = serialport::new(port_name, 115200)
            .data_bits(DataBits::Eight)
            .flow_control(FlowControl::None)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .timeout(Command::TIMEOUT_COMMON)
            .open()?;

        Ok(Self {
            port: op,
            default_settings_applied: Default::default(),
        })
    }

    /**
        Try to reach a well known state in which settings for further usage are set.
        This will also bring the module into the Command Mode and check for a compatible FW version.
    */
    pub fn reach_well_known_state(&mut self) -> Result<()> {
        // early return if we are already in a well known state
        if self.default_settings_applied {
            return Ok(());
        }

        // first try to request the FW version within a certain timeout
        self.write_line(Command::GetVersion)?;

        let answer: Vec<u8> = self
            .port
            .as_mut()
            .bytes()
            .take_while(|f| f.is_ok())
            .map(|f| f.expect("Couldn't get byte"))
            .collect();

        // if this is empty we assume that the device is in stream mode
        // TODO: catch the case that the default USB0 is any other device which does not answer
        if answer.is_empty() {
            self.send_leave_stream_mode_signal()?;
            self.reach_well_known_state()?;

            return Ok(());
        }
        let answer = std::str::from_utf8(&answer)?;
        trace!("FW version feedback: {}", answer);

        // parse FW version and check if compatible
        // atm only BGX13P.1.2.2738.2-1524-2738 is considered
        let _until_bgx = parse_fw_ver(answer).context("FW version couldn't be parsed")?;
        info!("Found FW: {}", _until_bgx);
        let other_fw = _until_bgx != "BGX13P.1.2.2738.2-1524-2738";

        self.apply_default_settings(other_fw)?;

        // verify success
        self.write_line(b"")?;
        let answer = self.read_answer(None)?;

        if let ModuleResponse::DataWithHeader(n, _) = answer {
            if ResponseCodes::Success == n.response_code {
                self.default_settings_applied = true;
                info!("Reached well known state");

                return Ok(());
            }
        }

        Err(anyhow!("Couldn't reach a well known state"))
    }

    /// scans for nearby BGX modules
    pub fn scan(&mut self) -> Result<Vec<ScanResult>> {
        self.skip_stream_mode()?;

        self.write_line(Command::SCAN)?;
        self.read_answer(None)?;
        debug!("start scan");
        sleep(Duration::from_secs(10));
        self.write_line(Command::SCAN_RESULTS)?;
        let ans = self.read_answer(Some(Duration::from_millis(20)))?;
        debug!("stop scan");

        match ans {
            ModuleResponse::DataWithHeader(_, ans) => {
                let ans = String::from_utf8(ans)?;
                return Ok(ans
                    .lines()
                    .filter_map(|f| ScanResult::from_str(f).ok())
                    .collect::<Vec<_>>());
            }
            ModuleResponse::DataWithoutHeader(_) => Err(anyhow!(
                "Got data without header when expecting scan answer"
            )),
        }
    }

    /// writes a command to the module which ends with \r\n and errors on timeout
    fn write_line(&mut self, cmd: &[u8]) -> Result<()> {
        let mut command = cmd.to_vec();
        command.extend(Command::LINEBREAK);

        self.port.write_all(&command)?;

        Ok(())
    }

    /// reads all available bytes from the module
    pub fn read(&mut self, custom_timeout: Option<Duration>) -> Result<Vec<u8>> {
        match self.read_answer(custom_timeout)?{
            ModuleResponse::DataWithHeader(h, _) => return  Err(anyhow!("Got data with header {:?} but expected passthrough payload from BGX module. This shouldn't happen normally.",h)),
            ModuleResponse::DataWithoutHeader(r) => Ok(r),
        }
    }

    // writes all byte to modules
    pub fn write(&mut self, payload: &[u8], custom_timeout: Option<Duration>) -> Result<()> {
        if let Some(custom_timeout) = custom_timeout {
            self.port.set_timeout(custom_timeout)?;
        }

        self.port.write_all(payload)?;

        if custom_timeout.is_some() {
            self.port.set_timeout(Command::TIMEOUT_COMMON)?;
        }

        Ok(())
    }

    fn read_answer(&mut self, custom_timeout: Option<Duration>) -> Result<ModuleResponse> {
        if let Some(custom_timeout) = custom_timeout {
            self.port.set_timeout(custom_timeout)?;
        } else {
            self.port.set_timeout(Command::TIMEOUT_COMMON)?;
        }

        let bytes: Vec<u8> = self
            .port
            .as_mut()
            .bytes()
            .take_while(|f| f.is_ok())
            .map(|f| f.expect("Couldn't get byte"))
            .collect();

        if custom_timeout.is_some() {
            self.port.set_timeout(Command::TIMEOUT_COMMON)?;
        }
        self.port.clear(All)?;

        if !bytes.is_empty() {
            if let Ok(bs) = std::str::from_utf8(&bytes) {
                debug!("BGX answered: {:?}", &bs);

                let header_str = between(token('R'), string("\r\n"), take::<&str>(6)).parse(bs)?;

                // let header_str = header_str?;
                trace!("Should be string for header: {:?}", header_str.0);
                let h = ResponseHeader::from_str(header_str.0)?;
                trace!("Parsed header: {:?}", h);

                /*
                SAMPLE:
                R000029\r\n
                BGX13P.1.2.2738.2-1524-2738\r\n
                */
                let answer = bytes
                    .get(9..h.length as usize + 9)
                    .context(format!(
                        "Couldn't get {} bytes as it should be possible declared in header",
                        h.length
                    ))?
                    .to_vec();
                Ok(ModuleResponse::DataWithHeader(h, answer))
            } else {
                Ok(ModuleResponse::DataWithoutHeader(bytes))
            }
        } else {
            Err(anyhow!("Didn't get any data when reading from BGX"))
        }

        // do not return an error because of response code here as this is a module error but not an error in the read-answer-process
        // match h.response_code {
        //     ResponseCodes::Success => (),
        //     _ => return Err(anyhow!(h.response_code)),
        // }
    }

    /// resets the module to factory default and applies default settings
    fn apply_default_settings(&mut self, expect_old_fw: bool) -> Result<()> {
        self.port.clear(All)?;
        self.port.set_timeout(Duration::from_millis(500))?;

        let cmds: Vec<&[u8]> = if expect_old_fw {
            vec![
                Command::SetModuleToMachineMode,
                Command::SystemRemoteCommandingFalse,
                Command::AdvertiseHighDuration,
                Command::BLEEncryptionPairingAny,
                Command::BLEPHYPreference1M,
                Command::SetDeviceName,
                Command::ClearAllBondings,
                Command::Save,
            ]
        } else {
            vec![
                Command::SetModuleToMachineMode,
                Command::SystemRemoteCommandingFalse,
                Command::AdvertiseHighDuration,
                Command::BLEEncryptionPairingAny,
                Command::BLEPHYMultiplexFalse,
                Command::BLEPHYPreference1M,
                Command::SetDeviceName,
                Command::ClearAllBondings,
                Command::Save,
            ]
        };

        for cmd in cmds {
            self.write_line(cmd)?;
            sleep(Duration::from_millis(200)); // here we do not use a read answer as it use rad until timeout and we do not know whether the header is already activated
            info!("Successfully applied setting");
        }

        let bytes: Vec<u8> = self
            .port
            .as_mut()
            .bytes()
            .take_while(|f| f.is_ok())
            .map(|f| f.expect("Couldn't get byte"))
            .collect();

        let answer = std::str::from_utf8(&bytes)?;
        trace!("Applied settings read: {}", answer);

        let count_success = answer.lines().filter(|l| l == &"Success").count();
        let expected_success = if expect_old_fw { 8 } else { 9 };

        if count_success != expected_success {
            return Err(anyhow!(
                "Only got {} times success instead of expected {} times",
                count_success,
                expected_success
            ));
        }

        self.port.clear(All)?;

        Ok(())
    }

    /// makes sure the module is not in stream mode anymore, should be ran before any other control commands should be send to the module
    fn skip_stream_mode(&mut self) -> Result<()> {
        debug!("Check if in stream mode...");
        self.port.clear(All)?;
        self.port.set_timeout(Command::TIMEOUT_COMMON)?;

        // here we write two times and then read
        // because we might have left over $$$ from an earlier command which hasn't been used as the device has not been in stream mode
        self.write_line(b"")?;
        self.write_line(b"")?;

        let read_from_port = self
            .port
            .as_mut()
            .bytes()
            .take_while(|f| f.is_ok())
            .map(|f| f.expect("Couldn't get byte"))
            .collect::<Vec<_>>();
        let answer = std::str::from_utf8(&read_from_port)?;

        trace!("Read from port test: {:?}", answer);

        if !answer.is_empty() {
            trace!("Got one or more Ready --> not in stream mode");

            // do not use common read answer method here as we can not always relying on getting a header due to a module being configured properly
            self.port.clear(All)?;

            return Ok(());
        } else {
            debug!("Probably in stream mode, try to leave...");
            sleep(Duration::from_millis(510));
            self.port.write_all(Command::BreakSequence)?;
            sleep(Duration::from_millis(510)); // min. 500 ms silence on UART for breakout sequence
            self.port.clear(All)?;

            debug!("Recheck if in stream mode...");
            self.skip_stream_mode()?;
        }

        Ok(())
    }

    fn send_leave_stream_mode_signal(&mut self) -> Result<()> {
        sleep(Duration::from_millis(510));
        self.port.write_all(Command::BreakSequence)?;
        sleep(Duration::from_millis(510)); // min. 500 ms silence on UART for breakout sequence
        self.port.clear(All)?;

        Ok(())
    }

    /// connects to a device with a given mac,
    /// skips if already connected to the device and disconnects before connecting to a new device
    pub fn connect(&mut self, mac: &str) -> Result<()> {
        self.skip_stream_mode()?;

        self.write_line(&Command::Connect(mac))?;
        let ans = self.read_answer(Some(Command::TIMEOUT_CONNECT))?;

        match ans {
            ModuleResponse::DataWithHeader(h, _) => match h.response_code {
                ResponseCodes::CommandFailed => {
                    self.disconnect()?;
                    Err(anyhow!("Command failed as devices where still connected but now has been disconnected"))
                }
                ResponseCodes::SecurityMismatch => {
                    self.write_line(Command::ClearAllBondings)?;
                    if let Ok(ModuleResponse::DataWithHeader(h, _)) = self.read_answer(None) {
                        if h.response_code == ResponseCodes::Success {
                            return Err(anyhow!(
                                "Security mismatch but performed clear bonding on device"
                            ));
                        }
                    }

                    Err(anyhow!(
                        "Security mismatch and performing clear bonding didn't worked out"
                    ))
                }
                ResponseCodes::Success => Ok(()),
                _ => Err(anyhow!(
                    "Error when handling connection but no plan how to handle it."
                )),
            },
            ModuleResponse::DataWithoutHeader(_) => Err(anyhow!(
                "Got data without header when being in connection process"
            )),
        }
    }

    pub fn disconnect(&mut self) -> Result<()> {
        self.skip_stream_mode()?;

        self.write_line(Command::Disconnect)?;
        self.read_answer(Some(Command::TIMEOUT_DISCONNECT))?;

        Ok(())
    }
}

fn parse_fw_ver(s: &str) -> Result<&str> {
    // do not match on BGX13P. instead of BGX13 here as this reported name is not consistent over older versions
    let first = repeat_skip_until(any(), string("BGX13")).parse(s)?.1;
    let mid = take_until_range("\r\n").parse(first)?.0;

    Ok(mid)
}

#[test]
fn parse_firmware_version_1() {
    let input1 = "XXXXXXBGX13P.1.2.2738.2-1524-2738\r\n";
    let input2 = "BGX13P.1.2.2738.2-1524-2738\r\n";

    assert_eq!(parse_fw_ver(input1).unwrap(), "BGX13P.1.2.2738.2-1524-2738");
    assert_eq!(parse_fw_ver(input2).unwrap(), "BGX13P.1.2.2738.2-1524-2738");
}

/// searches and returns serial port devices connected via USB
fn find_module() -> Result<Vec<SerialPortInfo>> {
    let ports = serialport::available_ports()?;
    trace!("Detected the following ports: {:?}", &ports);

    #[allow(clippy::unnecessary_filter_map)]
    let ports = ports
        .into_iter()
        .filter_map(|p| match &p.port_type {
            SerialPortType::UsbPort(n) => {
                debug!("Found USB port: {:?}", &n);

                if let Some(m) = &n.manufacturer {
                    if m.contains("Silicon Labs") || m.contains("Cygnal") || m.contains("CP21") {
                        Some(p)
                    } else {
                        warn!(
                            "Found UsbPort but manufacturer string {} didn't match for BGX",
                            m
                        );
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    Ok(ports)
}
