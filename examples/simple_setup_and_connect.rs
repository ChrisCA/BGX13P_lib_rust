use anyhow::Result;
use log::{error, info};
use simple_logger::SimpleLogger;
use std::{
    net::{SocketAddr, TcpListener},
    thread::sleep,
    time::Duration,
};
use BGX13P_lib_rust::bgx::Bgx13p;

fn main() -> Result<()> {
    SimpleLogger::new().init().unwrap();

    let listener = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], 56789))).unwrap();
    info!("Start listening...");
    loop {
        match listener.accept() {
            Ok((bgx, addr)) => {
                info!("Got incoming connection from: {}", addr);
                let mut bgx = Bgx13p::new(bgx).unwrap();
                bgx.reach_well_known_state()?;
                info!("Reached well known state");

                bgx.connect(&"d0cf5e828506".parse().unwrap())?;

                sleep(Duration::from_secs(2));

                bgx.disconnect()?;
            }
            Err(e) => error!("{e}"),
        };
    }
}
