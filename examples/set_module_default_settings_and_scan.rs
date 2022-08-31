use std::error::Error;

use log::debug;
use simple_logger::SimpleLogger;
use BGX13P_lib_rust::detect_modules;

fn main() -> Result<(), Box<dyn Error>> {
    SimpleLogger::new().init().unwrap();

    if let Some(bgx) = detect_modules().unwrap().first_mut() {
        bgx.reach_well_known_state()?;

        let res = bgx.scan()?;
        debug!("{:?}", res);
        Ok(())
    } else {
        Err("Couldn't apply settings".into())
    }
}
