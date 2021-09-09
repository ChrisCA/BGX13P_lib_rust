use anyhow::{anyhow, Result};

use log::debug;
use simple_logger::SimpleLogger;
use BGX13P_lib_rust::*;

fn main() -> Result<()> {
    SimpleLogger::new().init().unwrap();

    if let Ok(mut bgx) = Bgx13p::new() {
        bgx.reach_well_known_state()?;

        let res = bgx.scan()?;
        debug!("{:?}", res);
        Ok(())
    } else {
        Err(anyhow!("Couldn't apply settings"))
    }
}
