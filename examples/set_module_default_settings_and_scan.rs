use anyhow::Result;
use log::debug;
use simple_logger::SimpleLogger;
use BGX13P_lib_rust::bgx::detect_modules;

fn main() -> Result<()> {
    SimpleLogger::new().init().unwrap();

    if let Some(bgx) = detect_modules().unwrap().first_mut() {
        bgx.reach_well_known_state()?;

        let res = bgx.scan()?;
        debug!("{:?}", res);
        Ok(())
    } else {
        Err(anyhow::anyhow!("Couldn't apply settings"))
    }
}
