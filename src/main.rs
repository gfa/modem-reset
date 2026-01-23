use std::error::Error;
use std::time::Duration;
use std::{net::ToSocketAddrs, thread};
use tapo::ApiClient;
use tokio::runtime::Runtime;

fn main() {
    let rt = Runtime::new().unwrap();
    let current_datetime = chrono::offset::Local::now();
    println!("starting {}", current_datetime);

    loop {
        {
            if !pinger() {
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

fn pinger() -> bool {
    // TODO: catch failures on name resolution

    let address = "1.1.1.1:0".to_socket_addrs().unwrap().next().unwrap().ip();
    let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let timeout = Duration::from_millis(100); // 0.1 seconds
    let options = ping_rs::PingOptions {
        ttl: 128,
        dont_fragment: false,
    };

    let result = ping_rs::send_ping(&address, timeout, &data, Some(&options));

    match result {
        Ok(_reply) => true,
        Err(_e) => false,
    }
}
