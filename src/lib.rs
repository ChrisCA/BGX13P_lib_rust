#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![forbid(clippy::indexing_slicing)]
use anyhow::{anyhow, Context, Error, Result};
use combine::{
    between,
    parser::{
        char::string,
        range::{take, take_until_range},
        Parser,
    },
    token,
};
use log::{debug, info, trace, warn};
use serialport::{
    ClearBuffer::All, DataBits, FlowControl, Parity, SerialPort, SerialPortInfo, SerialPortType,
    StopBits,
};
use std::{
    fmt::Display,
    io::{Read, Write},
    str::FromStr,
    thread::sleep,
    time::Duration,
};
use thiserror::Error;

pub struct Bgx13p {
    port: Box<dyn SerialPort>,
    default_settings_applied: bool,
}

#[derive(Debug, PartialEq)]
pub struct ScanResult {
    pub mac: String,
    pub friendly_name: String,
    pub rssi: i8,
}

impl FromStr for ScanResult {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut spl = s.split_whitespace();

        let rssi: i8 = spl.nth(2).context("Couldn't get rssi on 2")?.parse()?;
        let mac = spl
            .next()
            .context("Couldn't get mac on 3")?
            .replace(':', "");
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

#[derive(Debug, PartialEq, Error)]
enum ResponseCodes {
    Success,
    CommandFailed,
    ParseError,
    UnknownCommand,
    TooFewArguments,
    TooManyArguments,
    UnknownVariableOrOption,
    InvalidArgument,
    Timeout,
    SecurityMismatch,
}

impl Display for ResponseCodes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", &self))
    }
}

#[derive(Debug, PartialEq)]
enum ModuleResponse {
    DataWithHeader(ResponseHeader, Vec<u8>),
    DataWithoutHeader(Vec<u8>),
}

impl From<u8> for ResponseCodes {
    fn from(value: u8) -> Self {
        match value {
            0 => ResponseCodes::Success,
            1 => ResponseCodes::CommandFailed,
            2 => ResponseCodes::ParseError,
            3 => ResponseCodes::UnknownCommand,
            4 => ResponseCodes::TooFewArguments,
            5 => ResponseCodes::TooManyArguments,
            6 => ResponseCodes::UnknownVariableOrOption,
            7 => ResponseCodes::InvalidArgument,
            8 => ResponseCodes::Timeout,
            9 => ResponseCodes::SecurityMismatch,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq)]
struct ResponseHeader {
    pub response_code: ResponseCodes,
    pub length: u16,
}

impl FromStr for ResponseHeader {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // typical header -> R000117\r\n but R and newline should already been removed by the parser

        if s.len() != 6 {
            return Err(anyhow!("Header code lenght is != 6"));
        }

        Ok(Self {
            response_code: s
                .get(..1)
                .context("Couldn't get code number in header")?
                .parse::<u8>()?
                .into(),
            length: s
                .get(1..6)
                .context("Couldn't get length numbers in header")?
                .parse::<u16>()?,
        })
    }
}

#[test]
fn test_response_header_1() {
    const HEADER: &str = "000009";

    let h = ResponseHeader::from_str(HEADER).unwrap();
    let h2 = ResponseHeader {
        response_code: ResponseCodes::Success,
        length: 9,
    };

    assert_eq!(h, h2);
}

#[test]
#[should_panic]
fn test_response_header_2() {
    const HEADER: &str = "000010";

    let h = ResponseHeader::from_str(HEADER).unwrap();
    let h2 = ResponseHeader {
        response_code: ResponseCodes::Success,
        length: 9,
    };

    assert_eq!(h, h2);
}

#[test]
#[should_panic]
fn test_response_header_3() {
    const HEADER: &str = "00009";

    let _ = ResponseHeader::from_str(HEADER).unwrap();
}

#[test]
#[should_panic]
fn test_response_header_4() {
    const HEADER: &str = "RR0009";

    let _ = ResponseHeader::from_str(HEADER).unwrap();
}

#[test]
#[should_panic]
fn test_response_header_5() {
    const HEADER: &str = "R10009";

    let _ = ResponseHeader::from_str(HEADER).unwrap();
}

#[test]
#[should_panic]
fn test_response_header_6() {
    const HEADER: &str = "2120009";

    let _ = ResponseHeader::from_str(HEADER).unwrap();
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

    /// tries to set the module in a well known state in which set settings and mode are defined
    pub fn reach_well_known_state(&mut self) -> Result<()> {
        // early return if we are already in a well known state
        if self.default_settings_applied {
            return Ok(());
        }

        self.skip_stream_mode()?;
        self.apply_default_settings()?;

        // verify success
        self.write_line(Command::GetVersion)?; // TODO: maybe do this later just by sending a line break and check for a ready
        let answer = self.read_answer(None)?;

        if let ModuleResponse::DataWithHeader(n, _) = answer {
            if ResponseCodes::Success == n.response_code {
                self.default_settings_applied = true;
                info!("Reached well known state");

                return Ok(());
            }
        }

        Err(anyhow!("Couldn't reach a well known case"))
    }

    /// scans for nearby BGX modules
    pub fn scan(&mut self) -> Result<Vec<ScanResult>> {
        self.skip_stream_mode()?;

        self.write_line(Command::SCAN)?;
        self.read_answer(None)?;
        dbg!("start scan");
        sleep(Duration::from_secs(10));
        self.write_line(Command::SCAN_RESULTS)?;
        let ans = self.read_answer(None)?;
        dbg!("stop scan");

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

        self.port.set_timeout(Command::TIMEOUT_COMMON)?;
        self.port.clear(All)?;

        if !bytes.is_empty() {
            if let Ok(bs) = String::from_utf8(bytes.clone()) {
                debug!("BGX answered: {:?}", &bs);

                let header_str = between(token('R'), string("\r\n"), take::<&str>(6)).parse(&bs)?;
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
    fn apply_default_settings(&mut self) -> Result<()> {
        self.port.clear(All)?;
        self.port.set_timeout(Duration::from_millis(500))?;

        let cmds: [&[u8]; 9] = [
            Command::SetModuleToMachineMode,
            Command::SystemRemoteCommandingFalse,
            Command::AdvertiseHighDuration,
            Command::BLEEncryptionPairingAny,
            Command::BLEPHYMultiplexFalse,
            Command::BLEPHYPreference1M,
            Command::SetDeviceName,
            Command::ClearAllBondings,
            Command::Save,
        ];

        for cmd in cmds.iter() {
            self.write_line(cmd)?;
            sleep(Duration::from_millis(50)); // here we do not use a read answer as it use rad until timeout and we do not know whether the header is already activated
            info!("Successfully applied setting");
        }

        sleep(Duration::from_millis(200));
        self.port.clear(All)?;

        Ok(())
    }

    /// makes sure the module is not in stream mode anymore, should be ran before any other control commands should be send to the module
    fn skip_stream_mode(&mut self) -> Result<()> {
        debug!("Check if in stream mode...");
        self.port.clear(All)?;
        self.port.set_timeout(Command::TIMEOUT_COMMON)?;

        self.write_line(b"")?;

        let read_from_port = String::from_utf8(
            self.port
                .as_mut()
                .bytes()
                .take_while(|f| f.is_ok())
                .map(|f| f.expect("Couldn't get byte"))
                .collect::<Vec<_>>(),
        )?;

        trace!("Read from port test: {}", &read_from_port);

        if let Ok(until) = take_until_range("Ready\r\n").parse(read_from_port.as_str()) {
            if !until.0.is_empty() {
                trace!(
                    "Found leftover {:?} before ready in getting at stream mode skipping",
                    until.0
                );
            }

            if let Some(after_ready) = until.1.get(7..) {
                if !after_ready.is_empty() {
                    trace!(
                        "Found leftover {:?} after ready in getting at stream mode skipping",
                        after_ready
                    );
                }
            }

            // do not use common read answer method here as we can not always relying on getting a header due to a module being configured properly
            self.port.clear(All)?;
            debug!("Didn't need to skip stream mode");
            return Ok(());
        } else {
            debug!("Probably in stream mode, try to leave...");
            sleep(Duration::from_millis(600));
            self.port.write_all(Command::BreakSequence)?;
            sleep(Duration::from_millis(600)); // min. 500 ms silence on UART for breakout sequence
            self.port.clear(All)?;

            self.skip_stream_mode()?;
        }

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

/// searches and returns serial port devices connected via USB
fn find_module() -> Result<Vec<SerialPortInfo>> {
    let ports = serialport::available_ports()?;
    trace!("Detected the following ports: {:?}", &ports);

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

struct Command;

impl Command {
    pub const GetVersion: &'static [u8; 3] = b"ver";
    pub fn Connect(mac: &str) -> Vec<u8> {
        format!("con {} 1", mac).as_bytes().to_vec()
    }
    pub const Disconnect: &'static [u8; 3] = b"dct";
    pub const Save: &'static [u8; 4] = b"save";
    pub const AdvertiseHighDuration: &'static [u8; 14] = b"set bl v h d 0";
    pub const BLEPHYMultiplexFalse: &'static [u8; 12] = b"set bl p m 0";
    pub const BLEPHYPreference1M: &'static [u8; 13] = b"set bl p p 1m";
    pub const BLEEncryptionPairingAny: &'static [u8; 14] = b"set bl e p any";
    pub const SystemRemoteCommandingFalse: &'static [u8; 12] = b"set sy r e 0";
    pub const BreakSequence: &'static [u8; 3] = b"$$$";
    pub const SetDeviceName: &'static [u8; 21] = b"set sy d n JugglerBGX";
    pub const SetModuleToMachineMode: &'static [u8; 18] = b"set sy c m machine";
    pub const ClearAllBondings: &'static [u8; 4] = b"clrb";
    pub const LINEBREAK: &'static [u8; 2] = b"\r\n";
    /*
    R000009\r\n
    Success\r\n
    */
    pub const SCAN: &'static [u8; 4] = b"scan";
    /*
    R000117\r\n
    !  # RSSI BD_ADDR           Device Name\r\n
    #  1  -47 d0:cf:5e:82:85:06 LOR-8090\r\n
    #  2  -52 00:0d:6f:a7:a1:54 LOR-8090\r\n
    */
    pub const SCAN_RESULTS: &'static [u8; 12] = b"scan results";
    pub const TIMEOUT_COMMON: Duration = Duration::from_millis(50);
    pub const TIMEOUT_CONNECT: Duration = Duration::from_millis(1100);
    pub const TIMEOUT_DISCONNECT: Duration = Duration::from_millis(100);
}
