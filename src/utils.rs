use crate::{config, utils};
use std::{convert::AsRef, net::Ipv4Addr, str::FromStr};

pub static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

pub static IP_CHECK_ADDRESS: &str = "https://ident.me";

pub static DDNS_ADDRESS: &str = "https://dy.fi/nic/update";

pub fn basic_auth<T: AsRef<str>>(user: T, password: T) -> String {
    let encoded = base64::encode(format!("{}:{}", user.as_ref(), password.as_ref()));
    format!("Basic {}", encoded)
}

pub fn get_ip(agent: &ureq::Agent, config: &config::Config) -> anyhow::Result<Ipv4Addr> {
    Ok(agent
        .get(utils::IP_CHECK_ADDRESS)
        .set("Connection", "Keep-Alive")
        .set(
            "Keep-Alive",
            &format!("timeout={}, max=1000", config.ip_checker.interval + 5),
        )
        .call()?
        .into_string()?
        .parse()?)
}

pub fn update_ddns(agent: &ureq::Agent, config: &config::Config) -> anyhow::Result<DdnsStatus> {
    let response = agent
        .get(utils::DDNS_ADDRESS)
        .set(
            "Authorization",
            &basic_auth(&config.ddns.username, &config.ddns.password),
        )
        .call()?
        .into_string()?;
    Ok(response.parse()?)
}

#[derive(Debug, PartialEq)]
pub enum DdnsStatus {
    BadAuth,
    NoHost,
    NotFqdn,
    BadIp(Ipv4Addr),
    NoChg,
    Good(Ipv4Addr),
    DnsErr,
    Abuse,
}

#[derive(Debug, thiserror::Error)]
pub enum DdnsError {
    #[error("dy.fi returned invalid IP")]
    InvalidIp(#[from] std::net::AddrParseError),
    #[error("invalid response: '{0}'")]
    InvalidResponse(String),
}

impl FromStr for DdnsStatus {
    type Err = DdnsError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "badauth" => Ok(DdnsStatus::BadAuth),
            "nohost" => Ok(DdnsStatus::NoHost),
            "notfqdn" => Ok(DdnsStatus::NotFqdn),
            "nochg" => Ok(DdnsStatus::NoChg),
            "dnserr" => Ok(DdnsStatus::DnsErr),
            "abuse" => Ok(DdnsStatus::Abuse),
            _ => {
                // Manually check badip and good since they provide ip addresses
                if input.starts_with("badip") {
                    let ip: Ipv4Addr = input[7..].parse().map_err(DdnsError::InvalidIp)?;
                    Ok(DdnsStatus::BadIp(ip))
                } else if input.starts_with("good") {
                    let ip: Ipv4Addr = input[6..].parse().map_err(DdnsError::InvalidIp)?;
                    Ok(DdnsStatus::Good(ip))
                } else {
                    Err(DdnsError::InvalidResponse(input.to_string()))
                }
            }
        }
    }
}
