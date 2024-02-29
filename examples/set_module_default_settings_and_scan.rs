use anyhow::Result;
use log::{debug, error, info};
use simple_logger::SimpleLogger;
use std::net::{SocketAddr, TcpListener};
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

                let res = bgx.scan()?;
                debug!("{:?}", res);
            }
            Err(e) => error!("{e}"),
        };
    }
}
