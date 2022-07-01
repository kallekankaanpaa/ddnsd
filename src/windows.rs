use crate::{config, utils, SERVICE_NAME};
use std::{sync::mpsc, thread, time};

#[cfg(target_family = "windows")]
pub fn ddns_service_main(_arguments: Vec<std::ffi::OsString>) {
    if let Err(err) = run_service() {
        log::error!("{}", err);
    }
}

#[cfg(target_family = "windows")]
fn run_service() -> anyhow::Result<()> {
    use windows_service::{
        service::{
            ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
            ServiceType,
        },
        service_control_handler::{self, ServiceControlHandlerResult},
    };

    let (shutdown_sender, shutdown_receiver) = mpsc::channel();

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop => {
                shutdown_sender
                    .send(())
                    .expect("Failed to send the stop signal");
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: time::Duration::default(),
        process_id: None,
    })?;

    let config = config::init_config()?;
    let agent = ureq::builder().user_agent(utils::APP_USER_AGENT).build();

    let (ip_sender, ip_receiver) = mpsc::channel();

    thread::spawn(move || loop {
        match ip_receiver.recv_timeout(time::Duration::from_secs(12 * 60 * 60)) {
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
        match shutdown_receiver.recv_timeout(time::Duration::from_secs(config.ip_checker.interval))
        {
            Err(mpsc::RecvTimeoutError::Timeout) => {
                log::info!("Timeout reached, check current ip");
                let ip = utils::get_ip(&agent, &config)?;
                log::info!("Old ip: {}, current ip: {}", old_ip, ip);

                if old_ip != ip {
                    old_ip = ip;
                    ip_sender.send(ip)?;
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                log::warn!("Shutdown sender disconnected, shutting down");
                break;
            }
            Ok(_) => {
                log::info!("Shutdown event received");
                break;
            }
        }
    }

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: time::Duration::default(),
        process_id: None,
    })?;

    Ok(())
}
