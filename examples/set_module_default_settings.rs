use std::error::Error;

use simple_logger::SimpleLogger;
use BGX13P_lib_rust::bgx::detect_modules;

fn main() -> Result<(), Box<dyn Error>> {
    SimpleLogger::new().init().unwrap();

    if let Some(bgx) = detect_modules().unwrap().first_mut() {
        bgx.reach_well_known_state()?;
        Ok(())
    } else {
        Err("Couldn't apply settings".into())
    }
}
