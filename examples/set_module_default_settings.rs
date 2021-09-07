use anyhow::{anyhow, Result};

use BGX13P_lib_rust::*;

fn main() -> Result<()> {
    if let Ok(mut bgx) = Bgx13p::new() {
        bgx.reach_well_known_state()?;
        Ok(())
    } else {
        Err(anyhow!("Couldn't apply settings"))
    }
}
