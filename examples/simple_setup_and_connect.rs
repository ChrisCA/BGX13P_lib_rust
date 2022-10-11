use anyhow::Result;
use std::{thread::sleep, time::Duration};

use BGX13P_lib_rust::bgx::detect_modules;

fn main() -> Result<()> {
    if let Some(bgx) = detect_modules().unwrap().first_mut() {
        bgx.reach_well_known_state()?;

        bgx.connect(&"d0cf5e828506".parse().unwrap())?;

        sleep(Duration::from_secs(2));

        bgx.disconnect()?;

        Ok(())
    } else {
        Err(anyhow::anyhow!("Couldn't apply settings"))
    }
}
