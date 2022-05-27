use std::sync::mpsc;
use std::thread;
use std::time;

mod config;
mod utils;

fn main() -> anyhow::Result<()> {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let config = config::init_config()?;
    let agent = ureq::builder().user_agent(utils::APP_USER_AGENT).build();

    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || loop {
        match receiver.recv_timeout(time::Duration::from_secs(12 * 60 * 60)) {
            Err(mpsc::RecvTimeoutError::Timeout) => {
                log::info!("Timeout reached. Update ddns service with current ip");
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                log::warn!("IP checking thread disconnected, closing updater thread");
                break;
            }
            Ok(ip) => {
                log::info!("Updating ddns service with new ip: {}", ip);
            }
        }
    });

    let mut old_ip = utils::get_ip(&agent, &config)?;

    loop {
        let ip = utils::get_ip(&agent, &config)?;
        log::info!("Old ip: {}, current ip: {}", old_ip, ip);

        if old_ip != ip {
            old_ip = ip;
            sender.send(ip)?;
        }

        thread::sleep(time::Duration::from_secs(config.ip_checker.interval));
    }
}
