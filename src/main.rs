use std::thread;
use core::init;


fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("🚀 Start of FAGA Browser...");

    init();
}
