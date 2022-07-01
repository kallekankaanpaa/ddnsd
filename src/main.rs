use std::sync::mpsc;
use std::thread;
use std::time;

mod config;
mod utils;
mod windows;

fn maini() -> anyhow::Result<()> {
    //simple_logger::init_with_level(log::Level::Info).unwrap();

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

#[cfg(target_family = "unix")]
fn main() -> std::process::ExitCode {
    std::process::ExitCode::SUCCESS;
}

#[cfg(target_family = "windows")]
pub const SERVICE_NAME: &str = include_str!(r"..\service_name.in");

#[cfg(target_family = "windows")]
use windows::ddns_service_main;
#[cfg(target_family = "windows")]
windows_service::define_windows_service!(ffi_service_main, ddns_service_main);

#[cfg(target_family = "windows")]
fn main() -> windows_service::Result<()> {
    use windows_event_log::{EventLog, EventLogKey};
    let event_log = EventLog::new(EventLogKey::default(), SERVICE_NAME, log::Level::Info);
    event_log
        .set_message_file_location()
        .unwrap()
        .register()
        .unwrap();
    windows_service::service_dispatcher::start(SERVICE_NAME, ffi_service_main)
}
