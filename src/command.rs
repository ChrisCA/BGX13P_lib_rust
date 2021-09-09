use std::time::Duration;

pub(crate) struct Command;

impl Command {
    pub const GetVersion: &'static [u8; 3] = b"ver";
    pub fn Connect(mac: &str) -> Vec<u8> {
        format!("con {} {}", mac, Command::TIMEOUT_CONNECT_BGX_INTERN)
            .as_bytes()
            .to_vec()
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
    pub const TIMEOUT_COMMON: Duration = Duration::from_millis(20);
    // change this to automatically change the BGX scan timeout and the read answer timeout
    pub const TIMEOUT_CONNECT_BGX_INTERN: u64 = 2;
    // change this to modify the timeout for the read answer timeout
    pub const TIMEOUT_CONNECT: Duration =
        Duration::from_millis(100 + Command::TIMEOUT_CONNECT_BGX_INTERN * 1000);
    pub const TIMEOUT_DISCONNECT: Duration = Duration::from_millis(100);
}
