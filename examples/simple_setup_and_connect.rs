use anyhow::{anyhow, Result};
use std::{thread::sleep, time::Duration};

use BGX13P_lib_rust::*;

fn main() -> Result<()> {
    if let Some(mut bgx) = Bgx13p::new() {
        bgx.reach_well_known_state()?;

        bgx.connect("d0cf5e828506")?;

        sleep(Duration::from_secs(2));

        bgx.disconnect()?;

        Ok(())
    } else {
        Err(anyhow!("Couldn't apply settings"))
    }
}
