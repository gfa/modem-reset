use std::error::Error;
use std::{net::ToSocketAddrs, thread};
use tapo::ApiClient;
use tokio::runtime::Runtime;

use std::net::SocketAddr;
use std::time::Duration;

use surge_ping::{Client, Config, PingIdentifier, PingSequence, ICMP};

fn main() {
    let rt = Runtime::new().unwrap();
    let current_datetime = chrono::offset::Local::now();
    println!("starting {}", current_datetime);

    loop {
        {
            if !rt.block_on(pingfunc()) {
                let current_datetime = chrono::offset::Local::now();

                match rt.block_on(change_state(false)) {
                    Ok(_) => println!("{} off", current_datetime),
                    Err(e) => eprintln!("Error controlling plug: {} at {}", e, current_datetime),
                }
                thread::sleep(Duration::from_secs(20));

                let current_datetime = chrono::offset::Local::now();

                match rt.block_on(change_state(true)) {
                    Ok(_) => println!("{} on", current_datetime),
                    Err(e) => eprintln!("Error controlling plug: {} at {}", e, current_datetime),
                }
                thread::sleep(Duration::from_secs(600));
            }
        }
        thread::sleep(Duration::from_secs(20));
    }
}

async fn change_state(state: bool) -> Result<(), Box<dyn Error>> {
    let device = ApiClient::new("foo", "bar").p115("192.168.94.70").await?;

    let current_datetime = chrono::offset::Local::now();
    println!("turning the device: {} at {}", state, current_datetime);

    if state {
        device.on().await?;
    } else {
        device.off().await?;
    }

    Ok(())
}

async fn pingfunc() -> bool {
    let hosts = "_gateway:0".to_socket_addrs();

    let _host = match &hosts {
        Ok(_ip) => hosts.unwrap().next().unwrap(),
        // _gateway is magic, it can be only resolved when present
        Err(_e) => return false,
    };

    return true;

}
