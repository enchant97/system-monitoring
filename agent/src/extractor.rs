use actix_web::{
    dev::Payload, error::ErrorUnauthorized, http::header::HeaderValue, Error, FromRequest,
    HttpRequest,
};
use core::future::Future;
use std::pin::Pin;

use crate::config::Config;

pub struct Client {
    pub authenticated: bool,
}

fn ip_allowed(client_ip: String, allowed_ips: &[String]) -> bool {
    allowed_ips
        .iter()
        .any(|current_ip| client_ip.eq(current_ip))
}

fn auth_value_allowed(value: Option<&HeaderValue>, allowed_keys: &[String]) -> bool {
    // Valid value will look like: 'Bearer key'
    match value {
        Some(v) => {
            // TODO remove unwrap usage
            let mut key_value = v.to_str().unwrap().to_owned();
            key_value = key_value.strip_prefix("Bearer").unwrap().trim().to_owned();
            allowed_keys
                .iter()
                .any(|current_key| key_value.eq(current_key))
        }
        None => false,
    }
}

impl FromRequest for Client {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Client, Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let config = req.app_data::<Config>().unwrap();
        let auth_config = &config.authentication;
        let authorization_value = req.headers().get("Authorization");
        // get the clients ip address
        let client_ip = match config.using_proxy {
            false => req.connection_info().peer_addr().unwrap().to_string(),
            true => req
                .connection_info()
                .realip_remote_addr()
                .unwrap()
                .to_string(),
        };
        // ensures client is allowed
        let authenticated = match (auth_config.check_ip, auth_config.check_key) {
            (false, false) => Some(false), // auth is not needed as it's disabled
            (false, true) => {
                // only key authentication is required
                match auth_value_allowed(authorization_value, &auth_config.allowed_keys) {
                    true => Some(true),
                    false => None,
                }
            }
            (true, false) => {
                // only client ip check is required
                match ip_allowed(client_ip, &auth_config.allowed_ip) {
                    true => Some(true),
                    false => None,
                }
            }
            (true, true) => {
                // both key authentication and client ip is required
                match (
                    auth_value_allowed(authorization_value, &auth_config.allowed_keys),
                    ip_allowed(client_ip, &auth_config.allowed_ip),
                ) {
                    (true, true) => Some(true),
                    _ => None,
                }
            }
        };
        Box::pin(async move {
            match authenticated {
                Some(v) => Ok(Client { authenticated: v }),
                None => Err(ErrorUnauthorized("")),
            }
        })
    }
}
