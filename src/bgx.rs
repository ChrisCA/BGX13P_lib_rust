use anyhow::{anyhow, Result};
use log::{debug, info, trace, warn};
use serialport::{
    DataBits, FlowControl, Parity, SerialPort, SerialPortInfo, SerialPortType, StopBits,
};
use std::{
    error::Error,
    io::{Read, Write},
    thread::sleep,
    time::Duration,
};
use winnow::FinishIResult;

use crate::{
    command::Command,
    fw::parse_fw_ver,
    mac::Mac,
    response::{parse_response, BgxResponse, ResponseCodes},
    scan::ScanResult,
};

/// searches and returns serial port devices connected via USB
pub fn detect_modules() -> Result<Vec<Bgx13p>> {
    let ports = serialport::available_ports()?;
    trace!("Detected the following ports: {:#?}", &ports);

    let ports = ports
        .into_iter()
        .filter(|p| {
            if let SerialPortType::UsbPort(n) = &p.port_type {
                debug!("Found USB port: {:#?}", &n);

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
        .filter_map(|p| match Bgx13p::new(p) {
            Ok(m) => Some(m),
            Err(e) => {
                info!("USB device not used as BGX due to: {}", e);
                None
            }
        })
        .collect::<Vec<_>>();

    Ok(ports)
}

pub struct Bgx13p {
    port: Box<dyn SerialPort>,
    default_settings_applied: bool,
}

impl Bgx13p {
    fn new(pi: SerialPortInfo) -> Result<Self> {
        let op = serialport::new(pi.port_name, 115200)
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
    // this function is written in a way that it doesn't validate the protocol headers
    // as the module might not yet be configured to utilize these headers
    pub fn reach_well_known_state(&mut self) -> Result<()> {
        // early return if we are already in a well known state
        if self.default_settings_applied {
            return Ok(());
        }

        self.switch_to_command_mode()?;

        // first try to request the FW version within a certain timeout
        self.write_line(Command::GetVersion, None)?;

        let answer: Vec<u8> = self
            .port
            .as_mut()
            .bytes()
            .take_while(|f| f.is_ok())
            .collect::<Result<Vec<_>, std::io::Error>>()?;

        let answer = std::str::from_utf8(&answer)?;
        trace!("FW version feedback: {}", answer);

        // parse FW version and check if compatible
        // atm only BGX13P.1.2.2738 with multiple endings as ".2-1524-2738" is considered
        let (_, fw_version) = parse_fw_ver(answer)
            .finish_err()
            .map_err(|e| e.into_owned())?;
        info!("Found FW string: {fw_version}");
        let other_fw = !fw_version.contains("BGX13P.1.2.2738");

        self.apply_default_settings(other_fw)?;

        // verify success
        self.write_line(b"", None)?;
        let answer = self.read_answer(None)?;

        if let BgxResponse::DataWithHeader(n, _) = answer {
            if ResponseCodes::Success == n.response_code {
                self.default_settings_applied = true;
                info!("Reached well known state");

                return Ok(());
            }
        }

        Err(anyhow::anyhow!("Couldn't reach a well known state"))
    }

    /// Scans for nearby BGX modules.
    /// Module must not be connect or scan will fail.
    pub fn scan(&mut self) -> Result<ScanResult> {
        self.switch_to_command_mode()?;

        self.disconnect()?;

        self.write_line(Command::SCAN, None)?;
        self.read_answer(None)?;
        debug!("start scan");
        sleep(Duration::from_secs(10));
        self.write_line(Command::SCAN_RESULTS, None)?;
        let ans = self.read_answer(None)?;
        debug!("stop scan");

        ans.try_into()
            .map_err(|e: Box<dyn Error>| anyhow::anyhow!(e.to_string()))
    }

    /// writes a command to the module which ends with \r\n and errors on timeout
    fn write_line(&mut self, cmd: &[u8], custom_timeout: Option<Duration>) -> Result<()> {
        let command = [cmd, Command::LINEBREAK].concat();

        self.write(&command, custom_timeout)?;

        Ok(())
    }

    /// reads all available bytes from the module
    pub fn read(&mut self, custom_timeout: Option<Duration>) -> Result<Vec<u8>> {
        match self.read_answer(custom_timeout)? {
            BgxResponse::DataWithHeader(h, _) => Err(anyhow::anyhow!(
                "Got data with header {:?} but expected passthrough payload from BGX module.",
                h
            )),
            BgxResponse::DataWithoutHeader(r) => Ok(r.to_vec()),
        }
    }

    // writes all byte to modules
    pub fn write(&mut self, payload: &[u8], custom_timeout: Option<Duration>) -> Result<()> {
        if let Some(custom_timeout) = custom_timeout {
            self.port.set_timeout(custom_timeout)?;
        } else {
            self.port.set_timeout(Command::TIMEOUT_COMMON)?;
        }

        self.port.write_all(payload)?;

        Ok(())
    }

    fn read_answer(&mut self, custom_timeout: Option<Duration>) -> Result<BgxResponse> {
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
            .collect::<Result<_, _>>()?;

        let (_, resp) = parse_response(&bytes)
            .finish_err()
            .map_err(|e| anyhow!("{e:?}"))?;

        Ok(resp)
        // do not return an error because of response code here as this is a module error but not an error in the read-answer-process
        // match h.response_code {
        //     ResponseCodes::Success => (),
        //     _ => return Err(format!(h.response_code)),
        // }
    }

    /// resets the module to factory default and applies default settings
    fn apply_default_settings(&mut self, expect_old_fw: bool) -> Result<()> {
        self.switch_to_command_mode()?;

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
            // longer timeout as the "save" command may take longer
            self.write_line(cmd, Some(Command::TIMEOUT_SETTINGS))?;
            sleep(Duration::from_millis(200)); // here we do not use a read answer as it use rad until timeout and we do not know whether the header is already activated
            info!("Successfully applied setting");
        }

        let bytes: Vec<u8> = self
            .port
            .as_mut()
            .bytes()
            .take_while(|f| f.is_ok())
            .collect::<Result<Vec<_>, std::io::Error>>()?;

        let answer = std::str::from_utf8(&bytes)?;
        trace!("Applied settings read: {}", answer);

        let count_success = answer.lines().filter(|l| l == &"Success").count();
        let expected_success = if expect_old_fw { 8 } else { 9 };

        if count_success != expected_success {
            return Err(anyhow::anyhow!(
                "Only got {} times success instead of expected {} times",
                count_success,
                expected_success
            ));
        }

        Ok(())
    }

    /// makes sure the module is not in stream mode anymore, should be ran before any other control commands should be send to the module
    fn switch_to_command_mode(&mut self) -> Result<()> {
        debug!("Check if in stream mode...");
        self.port.set_timeout(Command::TIMEOUT_COMMON)?;

        // clear buffer to make sure what come later is nothing historical
        let _ = self
            .port
            .as_mut()
            .bytes()
            .take_while(|f| f.is_ok())
            .collect::<Result<Vec<_>, std::io::Error>>()?;

        // here we write two times and then read
        // because we might have left over $$$ from an earlier command which hasn't been used as the device has not been in stream mode
        self.write_line(b"", None)?;
        self.write_line(b"", None)?;

        let read_from_port = self
            .port
            .as_mut()
            .bytes()
            .take_while(|f| f.is_ok())
            .collect::<Result<Vec<_>, std::io::Error>>()?;

        if !read_from_port.is_empty() {
            trace!("Got one or more Ready --> not in stream mode");

            // do not use common read answer method here as we can not always relying on getting a header due to a module being configured properly
            let answer = std::str::from_utf8(&read_from_port)?;
            trace!("Read from port test: {:?}", answer);

            return Ok(());
        } else {
            debug!("No answer, expect stream mode, try to leave...");
            sleep(Duration::from_millis(550));
            self.port.write_all(Command::BreakSequence)?;
            sleep(Duration::from_millis(550)); // min. 500 ms silence on UART for breakout sequence

            let _ = self
                .port
                .as_mut()
                .bytes()
                .take_while(|f| f.is_ok())
                .collect::<Result<Vec<_>, std::io::Error>>()?;

            debug!("Recheck if in stream mode...");
            self.switch_to_command_mode()?;
        }

        debug!("Stream mode left");

        Ok(())
    }

    /// connects to a device with a given mac,
    /// skips if already connected to the device and disconnects before connecting to a new device
    pub fn connect(&mut self, mac: &Mac) -> Result<()> {
        self.switch_to_command_mode()?;

        self.disconnect()?;

        self.write_line(&Command::Connect(mac), Some(Command::TIMEOUT_CONNECT))?;
        let ans = self.read_answer(Some(Command::TIMEOUT_CONNECT))?;

        match ans {
            BgxResponse::DataWithHeader(h, _) => match h.response_code {
                ResponseCodes::CommandFailed => {
                    self.disconnect()?;
                    Err(anyhow::anyhow!("Command failed as devices where still connected but now has been disconnected"))
                }
                ResponseCodes::SecurityMismatch => {
                    self.write_line(Command::ClearAllBondings, None)?;
                    if let Ok(BgxResponse::DataWithHeader(h, _)) = self.read_answer(None) {
                        if h.response_code == ResponseCodes::Success {
                            return Err(anyhow::anyhow!(
                                "Security mismatch but performed clear bonding on device"
                            ));
                        }
                    }

                    Err(anyhow::anyhow!(
                        "Security mismatch and performing clear bonding didn't worked out"
                    ))
                }
                ResponseCodes::Success => Ok(()),
                ResponseCodes::Timeout => Err(anyhow::anyhow!(
                    "Couldn't connect to device within given time."
                )),
                _ => Err(anyhow::anyhow!(
                    "Error when handling connection but no plan how to handle it: {:?}",
                    h
                )),
            },
            BgxResponse::DataWithoutHeader(_) => Err(anyhow::anyhow!(
                "Got data without header when being in connection process"
            )),
        }
    }

    pub fn disconnect(&mut self) -> Result<()> {
        self.switch_to_command_mode()?;

        // ConParams command is only available starting from BGX FW 1.2045
        self.write_line(Command::ConParams, None)?;
        let r = self.read_answer(None)?;
        match r {
            BgxResponse::DataWithHeader(h, ans) => match h.response_code {
                ResponseCodes::Success => {
                    // sample output active connection
                    /*
                        R000108\r\n
                        !  Param Value\r\n
                        #  Addr  EC1BBD1B12A1\r\n
                        #  Itvl  12\r\n
                        #  Mtu   250\r\n
                        #  Phy   1m\r\n
                        #  Tout  400\r\n
                        #  Err   023E\r\n
                    */

                    // sample output no active connection
                    /*
                        R000031\r\n
                        !  Param Value\r\n
                        #  Err   0208\r\n
                    */
                    if ans.contains("Addr") {
                        self.write_line(Command::Disconnect, None)?;
                        self.read_answer(Some(Command::TIMEOUT_DISCONNECT))?;

                        return Ok(());
                    }

                    debug!("BGX not connected, not disconnect necessary");
                    Ok(())
                }
                _ => Err(anyhow::anyhow!(
                    "Got error with header {:?} and content {:?}",
                    h,
                    ans
                )),
            },
            BgxResponse::DataWithoutHeader(e) => {
                Err(anyhow::anyhow!("Got data without header: {:?}", e))
            }
        }
    }
}
