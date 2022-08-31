use std::{error::Error, thread::sleep, time::Duration};

use log::debug;
use simple_logger::SimpleLogger;
use BGX13P_lib_rust::detect_modules;

fn main() -> Result<(), Box<dyn Error>> {
    SimpleLogger::new().init().unwrap();
    // let macs = vec!["d0cf5e828506", /* "000d6fa7a5e8",*/ "000d6fa7a154"];

    if let Some(bgx) = detect_modules().unwrap().first_mut() {
        bgx.reach_well_known_state()?;

        let macs = bgx.scan()?.0.into_iter().map(|d| d.mac).collect::<Vec<_>>();
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
                match bgx.disconnect() {
                    Ok(_) => debug!("Disconnected from {}", m),
                    Err(e) => debug!("Couldn't disconnected from {} because of {:?}", m, e),
                }
                sleep(Duration::from_millis(100));
            }
        }
    } else {
        Err("Couldn't apply settings".into())
    }
}
