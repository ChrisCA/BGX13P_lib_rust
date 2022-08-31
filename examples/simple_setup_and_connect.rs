use std::{error::Error, thread::sleep, time::Duration};

use BGX13P_lib_rust::detect_modules;

fn main() -> Result<(), Box<dyn Error>> {
    if let Some(bgx) = detect_modules().unwrap().first_mut() {
        bgx.reach_well_known_state()?;

        bgx.connect("d0cf5e828506")?;

        sleep(Duration::from_secs(2));

        bgx.disconnect()?;

        Ok(())
    } else {
        Err("Couldn't apply settings".into())
    }
}
