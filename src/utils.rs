use crate::{
    config::{self, CONFIG},
    utils,
};
use once_cell::sync::OnceCell;
use reqwest::Client;
use std::{convert::AsRef, convert::From, net::Ipv4Addr};

pub const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

pub const IP_CHECK_ADDRESS: &str = "https://ident.me";

pub const DDNS_ADDRESS: &str = "https://dy.fi/nic/update";

pub static CLIENT: OnceCell<Client> = OnceCell::new();

pub fn init_http_client() {
    let client = Client::builder()
        .https_only(true)
        .user_agent(APP_USER_AGENT)
        .build()
        .unwrap();
    CLIENT.set(client).unwrap();
}

pub fn basic_auth<T: AsRef<str>>(user: T, password: T) -> String {
    let encoded = base64::encode(format!("{}:{}", user.as_ref(), password.as_ref()));
    format!("Basic {}", encoded)
}

pub async fn check_ip() -> anyhow::Result<Ipv4Addr> {
    let client = CLIENT
        .get()
        .expect("http client needs to be initalized first");
    let config = CONFIG
        .get()
        .expect("configuration needds to be initalized first");
    let response = client
        .get(utils::IP_CHECK_ADDRESS)
        .header("Connection", "Keep-Alive")
        .header(
            "Keep-Alive",
            &format!("timeout={}, max=1000", config.ip_checker.interval + 5),
        )
        .send()
        .await?;
    Ok(response.text().await?.parse()?)
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
    Ok(response.into())
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

impl<T: AsRef<str>> From<T> for DdnsStatus {
    fn from(response: T) -> Self {
        let response_ref = response.as_ref();
        match response_ref {
            "badauth" => DdnsStatus::BadAuth,
            "nohost" => DdnsStatus::NoHost,
            "notfqdn" => DdnsStatus::NotFqdn,
            "nochg" => DdnsStatus::NoChg,
            "dnserr" => DdnsStatus::DnsErr,
            "abuse" => DdnsStatus::Abuse,
            _ => {
                // Manually check badip and good since they provide ip addresses
                if response_ref.starts_with("badip") {
                    let ip: Ipv4Addr = response_ref[7..].parse().unwrap();
                    DdnsStatus::BadIp(ip)
                } else if response_ref.starts_with("good") {
                    let ip: Ipv4Addr = response_ref[6..].parse().unwrap();
                    DdnsStatus::Good(ip)
                } else {
                    panic!("Invalid return value from dy")
                }
            }
        }
    }
}
