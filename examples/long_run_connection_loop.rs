use anyhow::{anyhow, Result};
use std::{thread::sleep, time::Duration};

use log::debug;
use simple_logger::SimpleLogger;
use BGX13P_lib_rust::*;

fn main() -> Result<()> {
    SimpleLogger::new().init().unwrap();
    // let macs = vec!["d0cf5e828506", /* "000d6fa7a5e8",*/ "000d6fa7a154"];

    if let Some(mut bgx) = Bgx13p::new() {
        bgx.reach_well_known_state()?;

        let macs = bgx.scan()?.into_iter().map(|d| d.mac).collect::<Vec<_>>();
        debug!("Found: {:#?}", macs);

        loop {
            for m in &macs {
                debug!("Try to connect to {}", m);
                match bgx.connect(m) {
                    Ok(_) => {
                        debug!("Connected to {}", m);
                    }
                    Err(e) => {
                        debug!("Couldn't connected to {} {}", m, e);
                        continue;
                    }
                }

                sleep(Duration::from_millis(100));

                debug!("Try to disconnect from {}", m);
                if bgx.disconnect().is_ok() {
                    debug!("Disconnected from {}", m);
                } else {
                    debug!("Couldn't disconnected from {}", m);
                }
                sleep(Duration::from_millis(100));
            }
        }
    } else {
        Err(anyhow!("Couldn't apply settings"))
    }
}
