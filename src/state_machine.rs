use std::{fmt::Debug, time::Duration};

use anyhow::{Context, Result};
use log::{debug, info, warn};
use serialport::*;

use crate::{command::Command, find_module, scan_result::ScanResult};

/// State container which can contains several possible states.
struct State<S> {
    port: Box<dyn SerialPort>,
    state: S,
}

// debugging will only be used to log the states
impl<S> Debug for State<S>
where
    S: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State").field("state", &self.state).finish()
    }
}

/// The initial state in which its unknown whether the module has ever been configured
/// and whether the FW version is compatible.
/// However the serial port and the communication to the modules is established.
#[derive(Debug)]
struct Unknown;

/// The module is setup and configured for usage and the FW is compatible.
#[derive(Debug)]
struct Initialized;

/// Module is in CommandMode.
/// In this state its made sure that the module is not connected.
#[derive(Debug)]
struct CommandMode;

/// Module is in StreamMode.
/// In this state the module should always be connected.
#[derive(Debug)]
struct StreamMode;

// this area contains the logic of the state machine

impl State<Unknown> {
    /// Initializes the serial port connection to the module.
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
                    "Couldn't determine USB port: {:?} try using default /dev/ttyUSB0",
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

        Ok(State {
            port: op,
            state: Unknown,
        })
    }
}

impl State<Unknown> {
    /// Applies module settings and verifies the FW.
    pub fn init(self) -> Result<State<Initialized>> {
        debug!("Act within state: {:?}", self);

        // TODO: here apply module settings and verify FW

        Ok(State {
            port: self.port,
            state: Initialized,
        })
    }
}

impl State<Initialized> {
    /// Set module into the CommandMode
    pub fn to_command_mode(self) -> Result<State<CommandMode>> {
        debug!("Act within state: {:?}", self);

        todo!()
    }

    /// Scans for nearby devices
    pub fn scan(self) -> Result<(State<CommandMode>, Vec<ScanResult>), State<Initialized>> {
        todo!()
    }
}

impl State<CommandMode> {
    /// Connects to a device with a MAC address in format d0cf5e828506
    pub fn connect(self, mac: &str) -> Result<State<StreamMode>, State<CommandMode>> {
        debug!("Act within state: {:?}", self);

        todo!()
    }
}

impl State<StreamMode> {
    /// Receive answer from device the module is connected to
    pub fn read(
        self,
        timeout: Duration,
    ) -> Result<(State<StreamMode>, Vec<u8>), State<Initialized>> {
        debug!("Act within state: {:?}", self);

        todo!()
    }

    /// Write to the device the module is connected to
    pub fn write(
        self,
        payload: &[u8],
        timeout: Duration,
    ) -> Result<State<StreamMode>, State<Initialized>> {
        debug!("Act within state: {:?}", self);

        todo!()
    }

    /// Disconnect from the device the module is connected to
    pub fn disconnect(self) -> Result<State<CommandMode>, State<Initialized>> {
        debug!("Act within state: {:?}", self);

        todo!()
    }
}
