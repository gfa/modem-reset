use std::error::Error;
use std::time::Duration;
use std::{net::ToSocketAddrs, thread};
use tapo::requests::{EnergyDataInterval, PowerDataInterval};
use tapo::ApiClient;
use tokio::runtime::Runtime;

fn main() {
    let rt = Runtime::new().unwrap();

    loop {
        {
            if !pinger() {
                match rt.block_on(change_state(false)) {
                    Ok(_) => println!("Success! Plug is off."),
                    Err(e) => eprintln!("Error controlling plug: {}", e),
                }
                thread::sleep(Duration::from_secs(2));
                match rt.block_on(change_state(true)) {
                    Ok(_) => println!("Success! Plug is on."),
                    Err(e) => eprintln!("Error controlling plug: {}", e),
                }
                thread::sleep(Duration::from_secs(300));
            }
        }
        thread::sleep(Duration::from_secs(2));
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

fn pinger() -> bool {
    // TODO: catch failures on name resolution

    let address = "_gateway:0".to_socket_addrs().unwrap().next().unwrap().ip();
    let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let timeout = Duration::from_millis(100); // 0.1 seconds
    let options = ping_rs::PingOptions {
        ttl: 2,
        dont_fragment: false,
    };

    // println!("{}", address);

    let result = ping_rs::send_ping(&address, timeout, &data, Some(&options));

    match result {
        Ok(_reply) => true,
        Err(_e) => false,
    }
}
