use std::error::Error;
use std::{net::ToSocketAddrs, thread};
use tapo::ApiClient;
use tokio::runtime::Runtime;

use std::net::SocketAddr;
use std::time::Duration;

use surge_ping::{Client, Config, PingIdentifier, PingSequence, ICMP};

fn main() {
    let rt = Runtime::new().unwrap();
    println!("starting");

    loop {
        {
            if !rt.block_on(pingfunc()) {

                match rt.block_on(change_state(false)) {
                    Ok(_) => println!("off"),
                    Err(e) => eprintln!("Error controlling plug: {}", e),
                }
                thread::sleep(Duration::from_secs(20));

                match rt.block_on(change_state(true)) {
                    Ok(_) => println!("on"),
                    Err(e) => eprintln!("Error controlling plug: {}", e),
                }
                thread::sleep(Duration::from_secs(600));
            }
        }
        thread::sleep(Duration::from_secs(20));
    }
}

async fn change_state(state: bool) -> Result<(), Box<dyn Error>> {
    let device = ApiClient::new("foo", "bar").p115("192.168.94.70").await?;

    println!("turning the device: {}", state);

    if state {
        device.on().await?;
    } else {
        device.off().await?;
    }

    Ok(())
}

async fn pingfunc() -> bool {
    let hosts = "_gateway:0".to_socket_addrs();

    let host = match &hosts {
        Ok(_ip) => hosts.unwrap().next().unwrap(),
        // _gateway is magic, it can be only resolved when present
        Err(_e) => return false,
    };

    let mut config_builder = Config::builder();

    if host.is_ipv6() {
        config_builder = config_builder.kind(ICMP::V6);
    }
    let config = config_builder.build();

    let payload = vec![0; 1000]; // fixed size of 1000 bits
    let client = Client::new(&config).unwrap();

    let mut pinger = client.pinger(host.ip(), PingIdentifier(111)).await;

    if let SocketAddr::V6(addr) = host {
        pinger.scope_id(addr.scope_id());
    }
    pinger.timeout(Duration::from_secs(1));

    let mut failed_pings = 0;
    for i in 0..5 {
        let result = pinger.ping(PingSequence(i as u16), &payload).await;
        if result.is_err() {
            failed_pings += 1;
        }
        thread::sleep(Duration::from_secs(2));
    }

    if failed_pings > 2 {
        println!("failed_pings: {}", failed_pings);
        println!("host: {}", host);

        return false;
    }

    return true;
}
