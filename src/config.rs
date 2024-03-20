use std::net::IpAddr;

use serde::Deserialize;

fn default_ip() -> IpAddr {
    IpAddr::from([0, 0, 0, 0])
}

fn default_port() -> u16 {
    50051
}

fn default_flightmngr_url() -> String {
    "grpc://flightmngr:50051".to_string()
}

fn default_priceest_url() -> String {
    "grpc://priceest:50051".to_string()
}

fn default_ticketsrvc_url() -> String {
    "grpc://ticketsrvc:50051".to_string()
}

#[derive(Deserialize, Debug)]
pub struct Options {
    #[serde(default = "default_ip")]
    pub ip: IpAddr,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_flightmngr_url")]
    pub flightmngr_url: String,
    #[serde(default = "default_priceest_url")]
    pub priceest_url: String,
    #[serde(default = "default_ticketsrvc_url")]
    pub ticketsrvc_url: String,
}
