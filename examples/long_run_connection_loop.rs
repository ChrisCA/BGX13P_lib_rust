use anyhow::Result;
use log::{debug, error, info};
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
            }
            Err(e) => error!("{e}"),
        };
    }
}
