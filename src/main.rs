use std::thread;
use std::time::Duration;

use net::BrowserClient;
use core::init;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("🚀 Start of FAGA Browser...");

    thread::spawn(|| {
        thread::sleep(Duration::from_secs(5));
        log::info!("Network");

        let client = BrowserClient::new();
        let url = "https://www.example.com";
        match client.fetch(url) {
            Ok(content) => log::info!("Fetched content from {}: {} bytes", url, content.len()),
            Err(e) => log::error!("Failed to fetch {}: {}", url, e),
        }

    });

    init();
}