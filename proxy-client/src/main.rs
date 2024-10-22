use reqwest::blocking::Client;
use std::{error::Error, thread::sleep, time::Duration};
use tracing::info;
use tracing_subscriber;

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();
    let client = Client::new();
    
    loop {
        let response = client.get("https://www.google.com")
            .send()?
            .text()?;
        
        info!("Response from website: {}", response);

        sleep(Duration::from_secs(45));
    }
}
