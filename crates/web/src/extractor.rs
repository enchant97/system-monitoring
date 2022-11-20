use actix_web::{
    dev::Payload, error::ErrorUnauthorized, http::header::HeaderValue, Error, FromRequest,
    HttpRequest,
};
use agent_config::types::Config;
use core::future::Future;
use std::net::IpAddr;
use std::pin::Pin;

pub(crate) struct Client {
    pub authenticated: bool,
}

fn ip_allowed(client_ip: IpAddr, allowed_ips: &[IpAddr]) -> bool {
    allowed_ips
        .iter()
        .any(|current_ip| client_ip.eq(current_ip))
}

/// Checks a authorization header value ensuring it is valid
fn auth_value_allowed(value: Option<&HeaderValue>, allowed_keys: &[String]) -> Option<()> {
    // Valid value will look like: 'Bearer key'
    let value = value?;
    let value = value.to_str().ok()?;
    let value = &value.strip_prefix("Bearer")?;
    let value = value.trim().to_owned();
    // Check key value is in allowed keys
    match allowed_keys.iter().any(|current_key| value.eq(current_key)) {
        true => Some(()),
        false => None,
    }
}

fn get_client_ip(config: &Config, req: &HttpRequest) -> Option<IpAddr> {
    let ip = match config.using_proxy {
        false => req.connection_info().peer_addr()?.to_string(),
        true => req.connection_info().realip_remote_addr()?.to_string(),
    };

    ip.parse::<IpAddr>().ok()
}

impl FromRequest for Client {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Client, Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let config = req
            .app_data::<actix_web::web::Data<Config>>()
            .expect("Client app_data Config must not be None");
        let auth_config = &config.authentication;
        let authorization_value = req.headers().get("Authorization");
        // get the clients ip address
        let client_ip = get_client_ip(config, req);
        // ensures client is allowed
        let authenticated = match (auth_config.check_ip, auth_config.check_key) {
            (false, false) => Some(false), // auth is not needed as it's disabled
            (false, true) => {
                // only key authentication is required
                auth_value_allowed(authorization_value, &auth_config.allowed_keys).map(|()| true)
            }
            (true, false) => {
                // only client ip check is required
                match client_ip {
                    Some(ip) => match ip_allowed(ip, &auth_config.allowed_ip) {
                        true => Some(true),
                        false => None,
                    },
                    None => None,
                }
            }
            (true, true) => {
                // both key authentication and client ip is required
                match client_ip {
                    Some(ip) => match (
                        auth_value_allowed(authorization_value, &auth_config.allowed_keys),
                        ip_allowed(ip, &auth_config.allowed_ip),
                    ) {
                        (Some(()), true) => Some(true),
                        _ => None,
                    },
                    None => None,
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
