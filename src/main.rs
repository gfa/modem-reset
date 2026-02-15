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
    let hosts = "84.116.46.21:0".to_socket_addrs();

    let host = match &hosts {
        Ok(_ip) => hosts.unwrap().next().unwrap(),
        // _gateway is magic, it can be only resolved when present
        Err(_e) => return false,
    };

    let current_datetime = chrono::offset::Local::now();
    println!("pinging {} {}", host, current_datetime);

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
    }

    failed_pings < 3
}
